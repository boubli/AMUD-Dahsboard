(function () {
    const MODE_KEY = 'amud_search_mode';

    function getMode() {
        try {
            return localStorage.getItem(MODE_KEY) || 'apps';
        } catch (_e) {
            return 'apps';
        }
    }

    function setMode(mode) {
        try {
            localStorage.setItem(MODE_KEY, mode);
        } catch (_e) { /* ignore */ }
        refreshModeUi();
    }

    function refreshModeUi() {
        const mode = getMode();
        const engineSelect = document.getElementById('search-engine');
        const divider = document.querySelector('.search-input-divider');
        const input = document.getElementById('search-input');
        const modeApps = document.getElementById('search-mode-apps');
        const modeWeb = document.getElementById('search-mode-web');
        if (engineSelect) {
            engineSelect.style.display = mode === 'web' ? '' : 'none';
        }
        if (divider) {
            divider.style.display = mode === 'web' ? '' : 'none';
        }
        if (input) {
            input.placeholder = mode === 'web' ? 'Search the web...' : 'Search apps... (Ctrl+K)';
            input.setAttribute('aria-label', mode === 'web' ? 'Search the web' : 'Search apps');
        }
        if (modeApps) modeApps.classList.toggle('active', mode === 'apps');
        if (modeWeb) modeWeb.classList.toggle('active', mode === 'web');
    }

    function applyAppFilter(query) {
        const q = (query || '').trim().toLowerCase();
        const cardSelector = '.app-card:not(.filter-empty-msg), .feed-card:not(.filter-empty-msg)';
        const cards = document.querySelectorAll(cardSelector);
        let visible = 0;
        cards.forEach(function (card) {
            if (!q) {
                const cat = window.activeCategoryFilter || 'all';
                if (cat === 'all') {
                    card.style.display = 'flex';
                    visible += 1;
                } else {
                    const show = card.getAttribute('data-category') === cat;
                    card.style.display = show ? 'flex' : 'none';
                    if (show) visible += 1;
                }
                return;
            }
            const name = (card.getAttribute('data-app-name') || '').toLowerCase();
            const title = (card.querySelector('.app-card-title, .feed-card-title')?.textContent || '').toLowerCase();
            const match = name.includes(q) || title.includes(q);
            card.style.display = match ? 'flex' : 'none';
            if (match) visible += 1;
        });

        const grid = document.querySelector('.bento-grid, .feeds-grid');
        if (!grid) return;
        let emptyMsg = grid.querySelector('.filter-empty-msg');
        if (q && visible === 0) {
            if (!emptyMsg) {
                emptyMsg = document.createElement('div');
                emptyMsg.className = 'glass-panel filter-empty-msg ' + (grid.classList.contains('feeds-grid') ? 'feed-card' : 'app-card');
                grid.appendChild(emptyMsg);
            }
            emptyMsg.textContent = 'No apps match "' + query.trim() + '"';
            emptyMsg.style.display = 'flex';
        } else if (emptyMsg && q) {
            emptyMsg.style.display = 'none';
        }
    }

    function openWebSearch(query) {
        const engine = document.getElementById('search-engine')?.value || 'google';
        let url = 'https://www.google.com/search?q=' + encodeURIComponent(query);
        if (engine === 'duckduckgo') {
            url = 'https://duckduckgo.com/?q=' + encodeURIComponent(query);
        } else if (engine === 'bing') {
            url = 'https://www.bing.com/search?q=' + encodeURIComponent(query);
        }
        window.open(url, '_blank');
    }

    function init() {
        const input = document.getElementById('search-input');
        const modeApps = document.getElementById('search-mode-apps');
        const modeWeb = document.getElementById('search-mode-web');
        if (!input) return;

        refreshModeUi();
        if (modeApps) modeApps.addEventListener('click', function () { setMode('apps'); });
        if (modeWeb) modeWeb.addEventListener('click', function () { setMode('web'); });

        input.addEventListener('input', function () {
            if (getMode() === 'apps') {
                applyAppFilter(input.value);
            }
        });

        input.addEventListener('keydown', function (e) {
            if (e.key === 'Escape') {
                input.value = '';
                applyAppFilter('');
                input.blur();
                return;
            }
            if (e.key !== 'Enter') return;
            const query = input.value.trim();
            if (!query) return;
            if (getMode() === 'web') {
                openWebSearch(query);
                input.value = '';
            }
        });

        document.addEventListener('amud:category-filter', function () {
            if (getMode() === 'apps' && !input.value.trim()) {
                applyAppFilter('');
            }
        });

        window.amudFocusAppSearch = function () {
            if (getMode() !== 'apps') setMode('apps');
            input.focus();
            input.select();
        };
        window.amudClearAppSearch = function () {
            input.value = '';
            applyAppFilter('');
        };
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }
})();
