(function () {
    'use strict';

    var manifestIndex = {};
    var NONE_ICON = '/static/fallback.svg';

    function alpineRoot() {
        var body = document.body;
        if (!body || !body._x_dataStack || !body._x_dataStack.length) return null;
        return body._x_dataStack[0];
    }

    function setAlpineIntegration(field, value) {
        var root = alpineRoot();
        if (!root) return;
        if (field === 'newApp') {
            root.newApp.integration_type = value;
            if (typeof window.amudApplyIntegrationSpan === 'function') {
                window.amudApplyIntegrationSpan(root.newApp);
            }
        } else if (field === 'editApp') {
            root.editApp.integration_type = value;
            if (typeof window.amudApplyIntegrationSpan === 'function') {
                window.amudApplyIntegrationSpan(root.editApp);
            }
        }
    }

    function buildManifestIndex(manifest) {
        manifestIndex = { '': { label: 'None', icon: NONE_ICON, health_only: false } };
        if (!manifest || !manifest.groups) return;
        manifest.groups.forEach(function (g) {
            (g.integrations || []).forEach(function (item) {
                if (!item.id) return;
                manifestIndex[item.id] = {
                    label: item.label,
                    icon: item.icon || NONE_ICON,
                    health_only: !!item.health_only,
                };
            });
        });
    }

    function metaFor(id) {
        return manifestIndex[id] || { label: id || 'None', icon: NONE_ICON, health_only: false };
    }

    function updateTrigger(picker, id) {
        var meta = metaFor(id);
        var trigger = picker.querySelector('.integration-picker-trigger');
        var img = picker.querySelector('.integration-picker-trigger-icon');
        var label = picker.querySelector('.integration-picker-trigger-label');
        if (img) {
            img.src = meta.icon;
            img.alt = meta.label;
        }
        if (label) {
            label.textContent = meta.label + (meta.health_only ? ' (health)' : '');
        }
        if (trigger) {
            trigger.setAttribute('aria-expanded', 'false');
        }
        var hidden = picker.querySelector('input[type="hidden"]');
        if (hidden) {
            hidden.value = id;
            hidden.dispatchEvent(new Event('input', { bubbles: true }));
        }
        var field = picker.getAttribute('data-alpine-field');
        if (field) setAlpineIntegration(field, id);
    }

    function closeAllPanels(except) {
        document.querySelectorAll('.integration-picker-panel').forEach(function (panel) {
            if (except && panel === except) return;
            panel.hidden = true;
            var picker = panel.closest('.integration-picker');
            if (picker) {
                var trigger = picker.querySelector('.integration-picker-trigger');
                if (trigger) trigger.setAttribute('aria-expanded', 'false');
            }
        });
    }

    function renderList(picker, filter) {
        var list = picker.querySelector('.integration-picker-list');
        if (!list || !window.INTEGRATION_MANIFEST) return;
        var q = (filter || '').trim().toLowerCase();
        list.replaceChildren();

        var noneBtn = document.createElement('button');
        noneBtn.type = 'button';
        noneBtn.className = 'integration-picker-option';
        noneBtn.dataset.id = '';
        noneBtn.innerHTML =
            '<img src="' + NONE_ICON + '" alt="" class="integration-picker-option-icon">' +
            '<span class="integration-picker-option-label">None</span>';
        noneBtn.addEventListener('click', function () {
            selectOption(picker, '');
        });
        if (!q || 'none'.indexOf(q) !== -1) list.appendChild(noneBtn);

        window.INTEGRATION_MANIFEST.groups.forEach(function (g) {
            var groupItems = (g.integrations || []).filter(function (item) {
                if (!item.id) return false;
                if (!q) return true;
                return (
                    item.label.toLowerCase().indexOf(q) !== -1 ||
                    item.id.toLowerCase().indexOf(q) !== -1 ||
                    g.group.toLowerCase().indexOf(q) !== -1
                );
            });
            if (!groupItems.length) return;

            var heading = document.createElement('div');
            heading.className = 'integration-picker-group-label';
            heading.textContent = g.group;
            list.appendChild(heading);

            groupItems.forEach(function (item) {
                var btn = document.createElement('button');
                btn.type = 'button';
                btn.className = 'integration-picker-option';
                btn.dataset.id = item.id;
                var icon = item.icon || NONE_ICON;
                var suffix = item.health_only ? ' (health)' : '';
                btn.innerHTML =
                    '<img src="' + icon + '" alt="" class="integration-picker-option-icon">' +
                    '<span class="integration-picker-option-label">' + item.label + suffix + '</span>';
                btn.addEventListener('click', function () {
                    selectOption(picker, item.id);
                });
                list.appendChild(btn);
            });
        });
    }

    function selectOption(picker, id) {
        updateTrigger(picker, id);
        var panel = picker.querySelector('.integration-picker-panel');
        if (panel) panel.hidden = true;
        var search = picker.querySelector('.integration-picker-search');
        if (search) search.value = '';
        renderList(picker, '');
    }

    function initPicker(picker) {
        if (!picker || picker.dataset.amudPickerInit === '1') return;
        picker.dataset.amudPickerInit = '1';

        var trigger = picker.querySelector('.integration-picker-trigger');
        var panel = picker.querySelector('.integration-picker-panel');
        var search = picker.querySelector('.integration-picker-search');

        if (trigger && panel) {
            trigger.addEventListener('click', function (e) {
                e.stopPropagation();
                var open = panel.hidden;
                closeAllPanels(open ? panel : null);
                panel.hidden = !open;
                trigger.setAttribute('aria-expanded', open ? 'true' : 'false');
                if (open && search) {
                    search.focus();
                    renderList(picker, search.value);
                }
            });
        }

        if (search) {
            search.addEventListener('input', function () {
                renderList(picker, search.value);
            });
            search.addEventListener('click', function (e) {
                e.stopPropagation();
            });
        }

        var hidden = picker.querySelector('input[type="hidden"]');
        var initial = hidden ? hidden.value : '';
        updateTrigger(picker, initial);
        renderList(picker, '');
    }

    function initAllPickers() {
        document.querySelectorAll('.integration-picker').forEach(initPicker);
        if (typeof lucide !== 'undefined') lucide.createIcons();
    }

    document.addEventListener('click', function () {
        closeAllPanels(null);
    });

    document.addEventListener('keydown', function (e) {
        if (e.key === 'Escape') closeAllPanels(null);
    });

    window.amudInitIntegrationPickers = function (manifest) {
        window.INTEGRATION_MANIFEST = manifest;
        buildManifestIndex(manifest);

        var healthOnly = new Set();
        if (manifest && manifest.groups) {
            manifest.groups.forEach(function (g) {
                (g.integrations || []).forEach(function (item) {
                    if (item.id && item.health_only) healthOnly.add(item.id);
                });
            });
        }
        window.HEALTH_ONLY_INTEGRATIONS = healthOnly;

        initAllPickers();
    };

    window.amudRefreshIntegrationPicker = function (pickerId, value) {
        var picker = document.getElementById(pickerId);
        if (picker) updateTrigger(picker, value || '');
    };
})();
