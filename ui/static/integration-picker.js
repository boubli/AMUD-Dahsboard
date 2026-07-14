(function () {
    'use strict';

    let manifestIndex = {};
    const NONE_ICON = '/static/fallback.svg';

    function alpineRoot() {
        const body = document.body;
        return body?._x_dataStack?.[0] ?? null;
    }

    function setAlpineIntegration(field, value) {
        const root = alpineRoot();
        if (!root) return;
        if (field === 'newApp') {
            root.newApp.integration_type = value;
            if (typeof window.amudOnIntegrationTypeChange === 'function') {
                window.amudOnIntegrationTypeChange(root.newApp, value);
            } else if (typeof window.amudApplyIntegrationSpan === 'function') {
                window.amudApplyIntegrationSpan(root.newApp);
            }
        } else if (field === 'editApp') {
            root.editApp.integration_type = value;
            if (typeof window.amudOnIntegrationTypeChange === 'function') {
                window.amudOnIntegrationTypeChange(root.editApp, value);
            } else if (typeof window.amudApplyIntegrationSpan === 'function') {
                window.amudApplyIntegrationSpan(root.editApp);
            }
        }
    }

    function buildManifestIndex(manifest) {
        manifestIndex = { '': { label: 'None', icon: NONE_ICON, health_only: false } };
        if (!manifest?.groups) return;
        manifest.groups.forEach(function (g) {
            (g.integrations || []).forEach(function (item) {
                if (!item.id) return;
                manifestIndex[item.id] = {
                    label: item.label,
                    icon: item.icon || NONE_ICON,
                    health_only: !!item.health_only,
                    card_metrics: item.card_metrics || [],
                };
            });
        });
    }

    function metaFor(id) {
        return manifestIndex[id] || { label: id || 'None', icon: NONE_ICON, health_only: false };
    }

    function updateTrigger(picker, id) {
        const meta = metaFor(id);
        const trigger = picker.querySelector('.integration-picker-trigger');
        const img = picker.querySelector('.integration-picker-trigger-icon');
        const label = picker.querySelector('.integration-picker-trigger-label');
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
        const hidden = picker.querySelector('input[type="hidden"]');
        if (hidden) {
            hidden.value = id;
            hidden.dispatchEvent(new Event('input', { bubbles: true }));
        }
        const field = picker.dataset.alpineField;
        if (field) setAlpineIntegration(field, id);
    }

    function closeAllPanels(except) {
        document.querySelectorAll('.integration-picker-panel').forEach(function (panel) {
            if (except && panel === except) return;
            panel.hidden = true;
            const picker = panel.closest('.integration-picker');
            if (picker) {
                const trigger = picker.querySelector('.integration-picker-trigger');
                if (trigger) trigger.setAttribute('aria-expanded', 'false');
            }
        });
    }

    function matchesFilter(item, groupName, q) {
        if (!q) return true;
        const needle = q.toLowerCase();
        return (
            item.label.toLowerCase().includes(needle) ||
            item.id.toLowerCase().includes(needle) ||
            groupName.toLowerCase().includes(needle)
        );
    }

    function createNoneButton(picker) {
        const noneBtn = document.createElement('button');
        noneBtn.type = 'button';
        noneBtn.className = 'integration-picker-option';
        noneBtn.dataset.id = '';
        noneBtn.innerHTML =
            '<img src="' + NONE_ICON + '" alt="" class="integration-picker-option-icon">' +
            '<span class="integration-picker-option-label">None</span>';
        noneBtn.addEventListener('click', function () {
            selectOption(picker, '');
        });
        return noneBtn;
    }

    function createIntegrationButton(picker, item) {
        const btn = document.createElement('button');
        btn.type = 'button';
        btn.className = 'integration-picker-option';
        btn.dataset.id = item.id;
        const icon = item.icon || NONE_ICON;
        const suffix = item.health_only ? ' (health)' : '';
        btn.innerHTML =
            '<img src="' + icon + '" alt="" class="integration-picker-option-icon">' +
            '<span class="integration-picker-option-label">' + item.label + suffix + '</span>';
        btn.addEventListener('click', function () {
            selectOption(picker, item.id);
        });
        return btn;
    }

    function renderList(picker, filter) {
        const list = picker.querySelector('.integration-picker-list');
        if (!list || !window.INTEGRATION_MANIFEST) return;
        const q = (filter || '').trim().toLowerCase();
        list.replaceChildren();

        if (!q || 'none'.includes(q)) {
            list.appendChild(createNoneButton(picker));
        }

        window.INTEGRATION_MANIFEST.groups.forEach(function (g) {
            const groupItems = (g.integrations || []).filter(function (item) {
                if (!item.id) return false;
                return matchesFilter(item, g.group, q);
            });
            if (!groupItems.length) return;

            const heading = document.createElement('div');
            heading.className = 'integration-picker-group-label';
            heading.textContent = g.group;
            list.appendChild(heading);

            groupItems.forEach(function (item) {
                list.appendChild(createIntegrationButton(picker, item));
            });
        });
    }

    function selectOption(picker, id) {
        updateTrigger(picker, id);
        const panel = picker.querySelector('.integration-picker-panel');
        if (panel) panel.hidden = true;
        const search = picker.querySelector('.integration-picker-search');
        if (search) search.value = '';
        renderList(picker, '');
    }

    function initPicker(picker) {
        if (!picker || picker.dataset.amudPickerInit === '1') return;
        picker.dataset.amudPickerInit = '1';

        const trigger = picker.querySelector('.integration-picker-trigger');
        const panel = picker.querySelector('.integration-picker-panel');
        const search = picker.querySelector('.integration-picker-search');

        if (trigger && panel) {
            trigger.addEventListener('click', function (e) {
                e.stopPropagation();
                const open = panel.hidden;
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

        const hidden = picker.querySelector('input[type="hidden"]');
        const initial = hidden ? hidden.value : '';
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

        const healthOnly = new Set();
        if (manifest?.groups) {
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
        const picker = document.getElementById(pickerId);
        if (picker) updateTrigger(picker, value || '');
    };
})();
