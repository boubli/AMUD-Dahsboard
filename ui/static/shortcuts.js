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

    document.addEventListener('keydown', function (e) {
        const typing = isTypingTarget(document.activeElement);
        const mod = e.ctrlKey || e.metaKey;

        if (e.key === '?' && !typing) {
            e.preventDefault();
            showOverlay();
            return;
        }

        if (e.key === 'Escape') {
            hideOverlay();
            if (typeof window.amudClearAppSearch === 'function') {
                window.amudClearAppSearch();
            }
            return;
        }

        if (typing && !(mod && e.key.toLowerCase() === 'k')) return;

        if ((mod && e.key.toLowerCase() === 'k') || (e.key === '/' && !typing)) {
            e.preventDefault();
            if (typeof window.amudFocusAppSearch === 'function') {
                window.amudFocusAppSearch();
            }
            return;
        }

        if (!typing && e.key >= '1' && e.key <= '9' && !mod && !e.altKey) {
            const input = document.getElementById('search-input');
            if (input && input.value.trim()) return;
            switchCategoryByIndex(Number.parseInt(e.key, 10) - 1);
        }
    });
})();
