/**
 * Settings theme gallery — manifest v5 local assets.
 */
(function (global) {
    'use strict';

    function initThemePicker(options) {
        var opts = options || {};
        var grid = document.getElementById('theme-picker-grid');
        var categoriesEl = document.getElementById('theme-picker-categories');
        var searchInput = document.getElementById('theme-picker-search');
        var statusEl = document.getElementById('theme-picker-status');
        var downloadBtn = document.getElementById('btn-download-bundled-theme');
        var cssTextarea = document.getElementById('setting-custom-css');
        var bgInput = document.getElementById('setting-custom-bg-url');
        var activeThemeInput = document.getElementById('setting-active-theme-id');
        var accentInput = document.getElementById('setting-accent-color');
        var themeModeSelect = document.getElementById('theme-mode-select');
        var lightWarning = document.getElementById('theme-light-mode-warning');
        var defaultAccent = opts.defaultAccent || '#cf6427';
        var defaultWallpaper = opts.defaultWallpaper || '/static/wallpaper.png';
        var syncAccentFromCss = opts.syncAccentPickerFromCss;
        if (!grid || !cssTextarea) return;

        var manifest = { themes: [], categories: [] };
        var activeCategory = 'all';
        var selectedThemeId = activeThemeInput?.value || 'default';

        function selectedTheme() {
            return manifest.themes.find(function (t) { return t.id === selectedThemeId; });
        }

        function fetchThemeCss(file) {
            if (!file) return Promise.resolve('');
            return fetch('/static/themes/' + file, { cache: 'no-store' }).then(function (res) {
                if (!res.ok) throw new Error('Theme file not found');
                return res.text();
            });
        }

        function resolveThemeAsset(path) {
            if (!path) return '';
            if (path.indexOf('http://') === 0 || path.indexOf('https://') === 0) return path;
            if (path.indexOf('/') === 0) return path;
            var base = (manifest.assetBase || '/static/themes').replace(/\/$/, '');
            return base + '/' + path.replace(/^\//, '');
        }

        function applyPreviewThemeMeta(theme) {
            var scene = document.getElementById('preview-scene');
            if (!scene || !theme) return;
            scene.setAttribute('data-theme-id', theme.id || 'default');
            if (theme.uiProfile) {
                scene.setAttribute('data-ui-profile', theme.uiProfile);
            } else {
                scene.removeAttribute('data-ui-profile');
            }
            if (theme.layoutCss) {
                global.amudThemeEngine?.injectLayoutCss(theme.layoutCss);
            }
        }

        function updateLightModeWarning() {
            if (!lightWarning || !themeModeSelect) return;
            var hasCss = (cssTextarea.value || '').trim().length > 0;
            var isLight = themeModeSelect.value === 'light';
            lightWarning.style.display = (hasCss && isLight) ? 'block' : 'none';
        }

        function markActiveCard() {
            grid.querySelectorAll('.theme-picker-card').forEach(function (card) {
                card.classList.toggle('is-active', card.getAttribute('data-theme-id') === selectedThemeId);
            });
        }

        function markUnsaved() {
            if (global.settingsState && typeof global.settingsState === 'function') {
                try { global.settingsState().unsavedChanges = true; } catch (e) { /* ignore */ }
            }
            var form = document.getElementById('mainSettingsForm');
            if (form?._x_dataStack?.[0]) {
                form._x_dataStack[0].unsavedChanges = true;
            }
        }

        function finishPreview(theme) {
            cssTextarea.dispatchEvent(new Event('input', { bubbles: true }));
            if (typeof global.amudRefreshLivePreview === 'function') {
                global.amudRefreshLivePreview();
            }
            markActiveCard();
            updateLightModeWarning();
            if (statusEl && theme) {
                statusEl.textContent = 'Previewing "' + theme.name + '". Click Save Changes to apply dashboard-wide.';
            }
            markUnsaved();
        }

        function applyDefaultTheme(theme) {
            cssTextarea.value = '';
            if (accentInput) accentInput.value = defaultAccent;
            if (bgInput && theme.wallpaper) {
                bgInput.value = resolveThemeAsset(theme.wallpaper) || defaultWallpaper;
            }
            finishPreview(theme);
        }

        function applyBundledTheme(theme) {
            fetchThemeCss(theme.file).then(function (css) {
                cssTextarea.value = css;
                if (typeof syncAccentFromCss === 'function') {
                    syncAccentFromCss(css, accentInput);
                }
                if (bgInput && theme.usesWallpaper && theme.wallpaper) {
                    bgInput.value = resolveThemeAsset(theme.wallpaper);
                }
                finishPreview(theme);
            }).catch(function () {
                if (statusEl) statusEl.textContent = 'Failed to load theme — update AMUD to the latest ui.tar.gz.';
            });
        }

        function applyThemePreview(theme) {
            if (!theme) return;
            selectedThemeId = theme.id;
            if (activeThemeInput) activeThemeInput.value = theme.id;
            applyPreviewThemeMeta(theme);
            if (theme.id === 'default' || !theme.file) {
                applyDefaultTheme(theme);
                return;
            }
            applyBundledTheme(theme);
        }

        function filteredThemes() {
            var q = (searchInput?.value || '').trim().toLowerCase();
            return manifest.themes.filter(function (theme) {
                if (activeCategory !== 'all' && theme.category !== activeCategory) return false;
                if (!q) return true;
                var hay = (theme.name + ' ' + (theme.tags || []).join(' ') + ' ' + theme.id).toLowerCase();
                return hay.indexOf(q) >= 0;
            });
        }

        function buildThemeCard(theme) {
            var card = document.createElement('button');
            card.type = 'button';
            card.className = 'theme-picker-card' + (theme.id === selectedThemeId ? ' is-active' : '');
            card.setAttribute('data-theme-id', theme.id);
            card.setAttribute('aria-pressed', theme.id === selectedThemeId ? 'true' : 'false');
            var preview = document.createElement('div');
            preview.className = 'theme-picker-preview';
            preview.style.backgroundImage = "url('" + resolveThemeAsset(theme.preview || '') + "')";
            var body = document.createElement('div');
            body.className = 'theme-picker-card-body';
            var title = document.createElement('span');
            title.className = 'theme-picker-card-title';
            title.textContent = theme.name;
            var tags = document.createElement('span');
            tags.className = 'theme-picker-card-tags';
            tags.textContent = (theme.tags || []).slice(0, 2).join(' · ');
            body.appendChild(title);
            body.appendChild(tags);
            card.appendChild(preview);
            card.appendChild(body);
            card.addEventListener('click', function () { applyThemePreview(theme); });
            return card;
        }

        function renderGrid() {
            var themes = filteredThemes();
            grid.replaceChildren();
            themes.forEach(function (theme) {
                grid.appendChild(buildThemeCard(theme));
            });
            if (statusEl && !themes.length) {
                statusEl.textContent = 'No themes match your search.';
            }
        }

        var categoryLabels = {
            all: 'All',
            default: 'Default',
            classic: 'Classic',
            advanced: 'Advanced',
            nature: 'Nature',
            terminal: 'Terminal',
            feminine: 'Feminine',
            variety: 'Variety'
        };

        function renderCategories() {
            if (!categoriesEl) return;
            var cats = ['all'].concat(manifest.categories || []);
            categoriesEl.replaceChildren();
            cats.forEach(function (cat) {
                var btn = document.createElement('button');
                btn.type = 'button';
                btn.className = 'theme-picker-category' + (cat === activeCategory ? ' is-active' : '');
                btn.textContent = categoryLabels[cat] || cat;
                btn.addEventListener('click', function () {
                    activeCategory = cat;
                    categoriesEl.querySelectorAll('.theme-picker-category').forEach(function (b) {
                        b.classList.toggle('is-active', b === btn);
                    });
                    renderGrid();
                });
                categoriesEl.appendChild(btn);
            });
        }

        function bindDownload() {
            if (!downloadBtn) return;
            downloadBtn.addEventListener('click', function () {
                var theme = selectedTheme();
                if (!theme || !theme.file) {
                    if (statusEl) statusEl.textContent = 'Select a non-default theme to download CSS.';
                    return;
                }
                fetchThemeCss(theme.file).then(function (css) {
                    var blob = new Blob([css], { type: 'text/css' });
                    var url = URL.createObjectURL(blob);
                    var anchor = document.createElement('a');
                    anchor.href = url;
                    anchor.download = theme.file;
                    anchor.click();
                    URL.revokeObjectURL(url);
                    if (statusEl) statusEl.textContent = 'Downloaded ' + theme.file + '.';
                }).catch(function () {
                    if (statusEl) statusEl.textContent = 'Download failed.';
                });
            });
        }

        function loadManifest() {
            fetch('/static/themes/manifest.json', { cache: 'no-store' })
                .then(function (res) { return res.ok ? res.json() : { themes: [], categories: [] }; })
                .then(function (data) {
                    manifest = data && Array.isArray(data.themes) ? data : { themes: [], categories: [] };
                    if (!manifest.categories || !manifest.categories.length) {
                        manifest.categories = ['default', 'classic', 'advanced', 'nature', 'terminal', 'feminine', 'variety'];
                    }
                    renderCategories();
                    renderGrid();
                    markActiveCard();
                    updateLightModeWarning();
                    if (statusEl && manifest.themes.length) {
                        statusEl.textContent = manifest.themes.length + ' themes available offline. Click one to preview.';
                    }
                })
                .catch(function () {
                    if (statusEl) statusEl.textContent = 'Theme gallery unavailable — reinstall ui.tar.gz from a recent release.';
                });
        }

        bindDownload();
        if (searchInput) searchInput.addEventListener('input', renderGrid);
        if (themeModeSelect) themeModeSelect.addEventListener('change', updateLightModeWarning);
        cssTextarea.addEventListener('input', updateLightModeWarning);
        loadManifest();
    }

    global.amudInitThemePicker = initThemePicker;
})(typeof window !== 'undefined' ? window : globalThis);
