(function () {
    function isTypingTarget(el) {
        if (!el) return false;
        const tag = el.tagName;
        return tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT' || el.isContentEditable;
    }

    function showOverlay() {
        let overlay = document.getElementById('shortcuts-overlay');
        if (!overlay) {
            overlay = document.createElement('div');
            overlay.id = 'shortcuts-overlay';
            overlay.className = 'shortcuts-overlay';
            overlay.innerHTML =
                '<div class="shortcuts-panel glass-panel">' +
                '<h3>Keyboard shortcuts</h3>' +
                '<ul>' +
                '<li><kbd>Ctrl</kbd> + <kbd>K</kbd> — Focus app search</li>' +
                '<li><kbd>/</kbd> — Focus app search</li>' +
                '<li><kbd>Esc</kbd> — Clear search / close overlay</li>' +
                '<li><kbd>1</kbd>–<kbd>9</kbd> — Switch category tab</li>' +
                '<li><kbd>?</kbd> — Show this help</li>' +
                '</ul>' +
                '<button type="button" class="btn btn-secondary" id="shortcuts-close">Close</button>' +
                '</div>';
            document.body.appendChild(overlay);
            overlay.addEventListener('click', function (e) {
                if (e.target === overlay) hideOverlay();
            });
            document.getElementById('shortcuts-close')?.addEventListener('click', hideOverlay);
        }
        overlay.style.display = 'flex';
    }

    function hideOverlay() {
        const overlay = document.getElementById('shortcuts-overlay');
        if (overlay) overlay.style.display = 'none';
    }

    function switchCategoryByIndex(index) {
        const tabs = [...document.querySelectorAll('#category-tabs-container .filter-tab, #category-tabs-container .feed-filter-tab')];
        if (index < 0 || index >= tabs.length) return;
        tabs[index].click();
    }

    function handleQuestionKey(e, typing) {
        if (e.key !== '?' || typing) return false;
        e.preventDefault();
        showOverlay();
        return true;
    }

    function handleEscapeKey() {
        hideOverlay();
        if (typeof globalThis.amudClearAppSearch === 'function') {
            globalThis.amudClearAppSearch();
        }
    }

    function handleSearchFocusKey(e, typing) {
        const mod = e.ctrlKey || e.metaKey;
        if (!((mod && e.key.toLowerCase() === 'k') || (e.key === '/' && !typing))) return false;
        e.preventDefault();
        if (typeof globalThis.amudFocusAppSearch === 'function') {
            globalThis.amudFocusAppSearch();
        }
        return true;
    }

    function handleCategoryShortcut(e, typing, mod) {
        if (typing || e.key < '1' || e.key > '9' || mod || e.altKey) return;
        const input = document.getElementById('search-input');
        if (input?.value.trim()) return;
        switchCategoryByIndex(Number.parseInt(e.key, 10) - 1);
    }

    document.addEventListener('keydown', function (e) {
        const typing = isTypingTarget(document.activeElement);
        const mod = e.ctrlKey || e.metaKey;

        if (handleQuestionKey(e, typing)) return;

        if (e.key === 'Escape') {
            handleEscapeKey();
            return;
        }

        if (typing && !(mod && e.key.toLowerCase() === 'k')) return;

        if (handleSearchFocusKey(e, typing)) return;

        handleCategoryShortcut(e, typing, mod);
    });
})();
