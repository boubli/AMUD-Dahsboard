/**
 * AMUD Theme Engine — local icon packs, fonts, wallpaper (manifest v5).
 */
(function (global) {
    'use strict';

    var FROZEN_THEMES = { default: true, 'luxury-gold': true };
    var CHROME_SELECTOR = [
        '.topbar [data-lucide]',
        '.telemetry-bar-container [data-lucide]',
        '.greeting-widget [data-lucide]',
        '.clock-widget [data-lucide]',
        '.weather-widget [data-lucide]',
        '.category-tabs [data-lucide]',
        '.ws-status-pill [data-lucide]',
        '.settings-sidebar [data-lucide]',
        '.settings-page-title [data-lucide]',
        '.nav-item [data-lucide]',
        '.save-bar [data-lucide]',
        '[data-theme-icon]'
    ].join(', ');

    var manifestCache = null;

    function themeId() {
        return document.body?.getAttribute('data-theme-id') || 'default';
    }

    function resolveAssetUrl(manifest, path) {
        if (!path) return '';
        if (path.indexOf('http://') === 0 || path.indexOf('https://') === 0) return path;
        if (path.indexOf('/') === 0) return path;
        var base = manifest?.assetBase || '';
        if (!base) return '/' + path.replace(/^\//, '');
        return base.replace(/\/$/, '') + '/' + path.replace(/^\//, '');
    }

    function injectLayoutCss(layoutUrl) {
        if (!layoutUrl) return;
        var id = 'amud-theme-layout';
        var existing = document.getElementById(id);
        if (existing && existing.getAttribute('data-href') === layoutUrl) return;
        if (existing) existing.remove();
        var link = document.createElement('link');
        link.id = id;
        link.rel = 'stylesheet';
        link.href = layoutUrl;
        link.setAttribute('data-href', layoutUrl);
        document.head.appendChild(link);
    }

    function applyUiProfile(entry) {
        if (!entry || !entry.uiProfile) return;
        document.body.setAttribute('data-ui-profile', entry.uiProfile);
    }

    function loadManifest() {
        if (manifestCache) return Promise.resolve(manifestCache);
        return fetch('/static/themes/manifest.json', { cache: 'no-store' })
            .then(function (res) {
                if (!res.ok) throw new Error('manifest');
                return res.json();
            })
            .then(function (data) {
                manifestCache = data;
                return data;
            });
    }

    function themeEntry(manifest, id) {
        return (manifest.themes || []).find(function (t) { return t.id === id; });
    }

    function cacheKey(suffix) {
        return 'amud-theme-' + themeId() + '-' + suffix;
    }

    function injectFont(fontUrl) {
        if (!fontUrl) return;
        var id = 'amud-theme-font';
        var existing = document.getElementById(id);
        if (existing && existing.getAttribute('data-href') === fontUrl) return;
        if (existing) existing.remove();
        var link = document.createElement('link');
        link.id = id;
        link.rel = 'stylesheet';
        link.href = fontUrl;
        link.setAttribute('data-href', fontUrl);
        document.head.appendChild(link);
    }

    function applyWallpaperUrl(url) {
        if (!url) return;
        document.documentElement.style.setProperty('--brand-bg-image', "url('" + url.replace(/'/g, "\\'") + "')");
    }

    function fetchSvg(url) {
        var key = cacheKey('svg:' + url);
        try {
            var cached = sessionStorage.getItem(key);
            if (cached) return Promise.resolve(cached);
        } catch (e) { /* ignore */ }
        return fetch(url, { cache: 'force-cache' })
            .then(function (res) {
                if (!res.ok) throw new Error('svg');
                return res.text();
            })
            .then(function (text) {
                try { sessionStorage.setItem(key, text); } catch (e) { /* ignore */ }
                return text;
            });
    }

    function replaceLucideNode(el, svgText) {
        var wrap = document.createElement('span');
        wrap.className = 'amud-theme-icon';
        wrap.innerHTML = svgText;
        var svg = wrap.querySelector('svg');
        if (svg) {
            var size = el.getAttribute('data-icon-size') || el.style.width || '1.25rem';
            if (typeof size === 'string' && size) {
                svg.style.width = size;
                svg.style.height = size;
            }
            svg.setAttribute('aria-hidden', 'true');
        }
        el.replaceWith(wrap);
    }

    function loadIconPack(manifest, id) {
        if (FROZEN_THEMES[id]) return Promise.resolve(null);
        var entry = themeEntry(manifest, id);
        if (!entry || !entry.iconPack) return Promise.resolve(null);
        var packUrl = resolveAssetUrl(manifest, entry.iconPack);
        var key = cacheKey('pack:' + packUrl);
        try {
            var cached = sessionStorage.getItem(key);
            if (cached) return Promise.resolve(JSON.parse(cached));
        } catch (e) { /* ignore */ }
        return fetch(packUrl, { cache: 'force-cache' })
            .then(function (res) {
                if (!res.ok) throw new Error('pack');
                return res.json();
            })
            .then(function (pack) {
                try { sessionStorage.setItem(key, JSON.stringify(pack)); } catch (e) { /* ignore */ }
                return pack;
            });
    }

    function swapChromeIcons(manifest, pack) {
        if (!pack || !pack.icons) return Promise.resolve();
        var base = resolveAssetUrl(manifest, pack.base || '');
        var nodes = document.querySelectorAll(CHROME_SELECTOR);
        var tasks = [];
        nodes.forEach(function (el) {
            if (el.closest('.app-card')) return;
            var name = el.getAttribute('data-lucide') || el.getAttribute('data-theme-icon');
            if (!name || !pack.icons[name]) return;
            var file = pack.icons[name];
            var url = file.indexOf('http') === 0 ? file : base.replace(/\/$/, '') + '/' + file.replace(/^\//, '');
            tasks.push(
                fetchSvg(url).then(function (svg) {
                    replaceLucideNode(el, svg);
                }).catch(function () { /* keep lucide */ })
            );
        });
        return Promise.all(tasks);
    }

    function initThemeEngine() {
        var id = themeId();
        document.body.classList.add('theme-' + id);
        return loadManifest().then(function (manifest) {
            var entry = themeEntry(manifest, id);
            applyUiProfile(entry);
            if (entry?.layoutCss) injectLayoutCss(entry.layoutCss);
            if (entry?.fontUrl) injectFont(entry.fontUrl);
            if (entry?.wallpaper && entry.usesWallpaper !== false) {
                var wp = resolveAssetUrl(manifest, entry.wallpaper);
                if (wp) applyWallpaperUrl(wp);
            }
            return loadIconPack(manifest, id).then(function (pack) {
                return swapChromeIcons(manifest, pack);
            });
        }).catch(function () { /* offline / default */ });
    }

    global.amudThemeEngine = {
        themeId: themeId,
        resolveAssetUrl: resolveAssetUrl,
        loadManifest: loadManifest,
        applyUiProfile: applyUiProfile,
        injectLayoutCss: injectLayoutCss,
        init: initThemeEngine
    };

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', function () {
            initThemeEngine().then(function () {
                if (typeof lucide !== 'undefined' && lucide.createIcons) {
                    lucide.createIcons();
                }
            });
        });
    } else {
        initThemeEngine().then(function () {
            if (typeof lucide !== 'undefined' && lucide.createIcons) {
                lucide.createIcons();
            }
        });
    }
})(typeof window !== 'undefined' ? window : globalThis);
