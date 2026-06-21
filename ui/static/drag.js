/**
 * AMUD Drag & Drop — Lightweight card reorder engine
 * Only active when admin is logged in (controlled by isAdmin global)
 * Uses native HTML5 Drag & Drop API
 */
(function() {
    'use strict';

    // Wait for DOM and check admin status
    document.addEventListener('DOMContentLoaded', function() {
        // isAdmin is set as a global in index.html template
        if (typeof isAdmin === 'undefined' || !isAdmin) return;

        const grid = document.querySelector('.bento-grid');
        if (!grid) return;

        let draggedCard = null;
        let draggedIndex = -1;

        // Inject drag handles into all app cards
        function injectDragHandles() {
            const cards = grid.querySelectorAll('.app-card:not(.filter-empty-msg):not(.app-card--empty)');
            cards.forEach(function(card) {
                if (card.querySelector('.drag-handle')) return;
                const handle = document.createElement('div');
                handle.className = 'drag-handle';
                handle.setAttribute('title', 'Drag to reorder');
                handle.setAttribute('aria-label', 'Drag handle');
                card.insertBefore(handle, card.firstChild);
                card.setAttribute('draggable', 'true');
            });
        }

        injectDragHandles();

        // Observe for dynamically added cards
        const observer = new MutationObserver(function() {
            injectDragHandles();
        });
        observer.observe(grid, { childList: true });

        // Get all sortable cards (excluding empty/filter messages)
        function getSortableCards() {
            return Array.from(grid.querySelectorAll('.app-card:not(.filter-empty-msg):not(.app-card--empty)'));
        }

        // Drag start
        grid.addEventListener('dragstart', function(e) {
            const card = e.target.closest('.app-card');
            if (!card || card.classList.contains('filter-empty-msg')) return;

            draggedCard = card;
            draggedIndex = getSortableCards().indexOf(card);
            
            // Set drag image with slight offset
            if (e.dataTransfer) {
                e.dataTransfer.effectAllowed = 'move';
                e.dataTransfer.setData('text/plain', draggedIndex.toString());
            }

            // Add dragging class after a small delay to not affect drag image
            requestAnimationFrame(function() {
                card.classList.add('dragging');
                grid.classList.add('reordering');
            });
        });

        // Drag over — determine drop position
        grid.addEventListener('dragover', function(e) {
            e.preventDefault();
            if (!draggedCard) return;
            e.dataTransfer.dropEffect = 'move';

            const cards = getSortableCards();
            const overCard = e.target.closest('.app-card');
            
            // Clear all drag-over states
            cards.forEach(function(c) { c.classList.remove('drag-over'); });

            if (overCard && overCard !== draggedCard && !overCard.classList.contains('filter-empty-msg')) {
                overCard.classList.add('drag-over');
            }
        });

        // Drag leave
        grid.addEventListener('dragleave', function(e) {
            const card = e.target.closest('.app-card');
            if (card) {
                card.classList.remove('drag-over');
            }
        });

        // Drop — perform the reorder
        grid.addEventListener('drop', function(e) {
            e.preventDefault();
            if (!draggedCard) return;

            const overCard = e.target.closest('.app-card');
            if (!overCard || overCard === draggedCard || overCard.classList.contains('filter-empty-msg')) {
                cleanup();
                return;
            }

            const cards = getSortableCards();
            const fromIndex = cards.indexOf(draggedCard);
            const toIndex = cards.indexOf(overCard);

            if (fromIndex === -1 || toIndex === -1) {
                cleanup();
                return;
            }

            // Perform DOM reorder
            if (fromIndex < toIndex) {
                overCard.parentNode.insertBefore(draggedCard, overCard.nextSibling);
            } else {
                overCard.parentNode.insertBefore(draggedCard, overCard);
            }

            cleanup();
            persistOrder();
        });

        // Drag end — cleanup if drop didn't fire
        grid.addEventListener('dragend', function() {
            cleanup();
        });

        function cleanup() {
            if (draggedCard) {
                draggedCard.classList.remove('dragging');
            }
            getSortableCards().forEach(function(c) {
                c.classList.remove('drag-over');
            });
            grid.classList.remove('reordering');
            draggedCard = null;
            draggedIndex = -1;
        }

        // Persist new order to backend
        function persistOrder() {
            const cards = getSortableCards();
            const ids = cards.map(function(card) {
                return parseInt(card.getAttribute('data-app-id'), 10);
            }).filter(function(id) {
                return !isNaN(id);
            });

            if (ids.length === 0) return;

            var csrfToken = '';
            try {
                csrfToken = amudCsrfToken();
            } catch(e) {
                var meta = document.querySelector('meta[name="csrf-token"]');
                csrfToken = meta ? meta.getAttribute('content') : '';
            }

            fetch('/apps/reorder', {
                method: 'POST',
                headers: Object.assign(
                    { 'Content-Type': 'application/json' },
                    typeof amudCsrfHeaders === 'function' ? amudCsrfHeaders() : {}
                ),
                body: JSON.stringify({ ids: ids, csrf_token: csrfToken })
            })
            .then(function(res) { return res.json(); })
            .then(function(data) {
                if (!data.success) {
                    console.error('Reorder failed:', data.error);
                }
            })
            .catch(function(err) {
                console.error('Reorder network error:', err);
            });
        }
    });
})();
