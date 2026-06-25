/**
 * AMUD Drag & Drop — card reorder (admin only, handle-only, all-categories filter)
 */
(function() {
    'use strict';

    document.addEventListener('DOMContentLoaded', function() {
        if (typeof isAdmin === 'undefined' || !isAdmin) return;

        const grid = document.querySelector('main.bento-grid');
        if (!grid) return;

        let draggedCard = null;
        let orderSnapshot = null;
        let persistInFlight = false;
        let dropCommitted = false;
        let pointerDrag = null;

        function getSortableCards() {
            return Array.from(
                grid.querySelectorAll('.app-card:not(.filter-empty-msg):not(.app-card--empty)')
            ).filter(function(card) {
                return card.style.display !== 'none';
            });
        }

        function snapshotOrder() {
            return getSortableCards().map(function(card) {
                return {
                    id: card.getAttribute('data-app-id'),
                    node: card,
                    next: card.nextElementSibling,
                };
            });
        }

        function restoreOrder(snapshot) {
            if (!snapshot || !snapshot.length) return;
            snapshot.forEach(function(item) {
                if (!item.node || !item.node.parentNode) return;
                if (item.next && item.next.parentNode === grid) {
                    grid.insertBefore(item.node, item.next);
                } else {
                    grid.appendChild(item.node);
                }
            });
        }

        function reorderBlockedMessage() {
            if (window.activeCategoryFilter && window.activeCategoryFilter !== 'all') {
                return 'Switch to the All category before reordering cards.';
            }
            return null;
        }

        function clearDragHighlights() {
            getSortableCards().forEach(function(c) {
                c.classList.remove('drag-over');
            });
        }

        function cleanup() {
            if (draggedCard) {
                draggedCard.classList.remove('dragging');
            }
            clearDragHighlights();
            grid.classList.remove('reordering');
            draggedCard = null;
            pointerDrag = null;
        }

        function insertRelativeToTarget(targetCard, clientY) {
            if (!draggedCard || !targetCard || targetCard === draggedCard) return;
            if (targetCard.classList.contains('filter-empty-msg')) return;

            const rect = targetCard.getBoundingClientRect();
            const insertAfter = clientY > rect.top + rect.height / 2;

            if (insertAfter) {
                grid.insertBefore(draggedCard, targetCard.nextElementSibling);
            } else {
                grid.insertBefore(draggedCard, targetCard);
            }
        }

        function cardFromPoint(clientX, clientY) {
            const cards = getSortableCards();
            for (let i = 0; i < cards.length; i++) {
                const card = cards[i];
                if (card === draggedCard) continue;
                const rect = card.getBoundingClientRect();
                if (
                    clientX >= rect.left &&
                    clientX <= rect.right &&
                    clientY >= rect.top &&
                    clientY <= rect.bottom
                ) {
                    return card;
                }
            }
            return null;
        }

        function injectDragHandles() {
            const blocked = reorderBlockedMessage();
            const cards = grid.querySelectorAll('.app-card:not(.filter-empty-msg):not(.app-card--empty)');

            cards.forEach(function(card) {
                let handle = card.querySelector('.drag-handle');
                if (!handle) {
                    handle = document.createElement('div');
                    handle.className = 'drag-handle';
                    handle.setAttribute('title', 'Drag to reorder');
                    handle.setAttribute('aria-label', 'Drag to reorder');
                    card.insertBefore(handle, card.firstChild);
                }

                card.removeAttribute('draggable');
                handle.removeAttribute('draggable');

                if (blocked) {
                    handle.setAttribute('draggable', 'false');
                    handle.style.opacity = '0.35';
                    handle.style.cursor = 'not-allowed';
                    handle.style.pointerEvents = 'auto';
                } else {
                    handle.setAttribute('draggable', 'true');
                    handle.style.opacity = '';
                    handle.style.cursor = 'grab';
                    handle.style.pointerEvents = '';
                }
            });
        }

        injectDragHandles();

        const observer = new MutationObserver(function() {
            injectDragHandles();
        });
        observer.observe(grid, { childList: true });

        document.addEventListener('amud:category-filter', function() {
            injectDragHandles();
        });

        grid.addEventListener('dragstart', function(e) {
            const handle = e.target.closest('.drag-handle');
            if (!handle) {
                e.preventDefault();
                return;
            }

            const blocked = reorderBlockedMessage();
            if (blocked) {
                e.preventDefault();
                if (typeof amudShowToast === 'function') {
                    amudShowToast(blocked, 'warning');
                }
                return;
            }

            const card = handle.closest('.app-card');
            if (!card || card.classList.contains('filter-empty-msg')) {
                e.preventDefault();
                return;
            }

            draggedCard = card;
            orderSnapshot = snapshotOrder();
            dropCommitted = false;

            if (e.dataTransfer) {
                e.dataTransfer.effectAllowed = 'move';
                e.dataTransfer.setData('text/plain', card.getAttribute('data-app-id') || '');
            }

            requestAnimationFrame(function() {
                card.classList.add('dragging');
                grid.classList.add('reordering');
            });
        });

        grid.addEventListener('dragover', function(e) {
            e.preventDefault();
            if (!draggedCard) return;
            if (e.dataTransfer) {
                e.dataTransfer.dropEffect = 'move';
            }

            clearDragHighlights();

            const overCard = cardFromPoint(e.clientX, e.clientY);
            if (overCard && overCard !== draggedCard) {
                overCard.classList.add('drag-over');
                insertRelativeToTarget(overCard, e.clientY);
            }
        });

        grid.addEventListener('dragleave', function(e) {
            const card = e.target.closest('.app-card');
            if (card) {
                card.classList.remove('drag-over');
            }
        });

        grid.addEventListener('drop', function(e) {
            e.preventDefault();
            if (!draggedCard) return;

            const overCard = cardFromPoint(e.clientX, e.clientY);
            if (overCard && overCard !== draggedCard) {
                insertRelativeToTarget(overCard, e.clientY);
            } else if (!overCard) {
                grid.appendChild(draggedCard);
            }

            dropCommitted = true;
            persistOrder(orderSnapshot);
            cleanup();
        });

        grid.addEventListener('dragend', function() {
            if (!dropCommitted && orderSnapshot) {
                restoreOrder(orderSnapshot);
            }
            dropCommitted = false;
            orderSnapshot = null;
            cleanup();
        });

        function beginPointerDrag(handle, card, clientX, clientY) {
            const blocked = reorderBlockedMessage();
            if (blocked) {
                if (typeof amudShowToast === 'function') {
                    amudShowToast(blocked, 'warning');
                }
                return;
            }

            draggedCard = card;
            orderSnapshot = snapshotOrder();
            dropCommitted = false;
            pointerDrag = { handle: handle, active: true };
            card.classList.add('dragging');
            grid.classList.add('reordering');
            clearDragHighlights();

            const overCard = cardFromPoint(clientX, clientY);
            if (overCard && overCard !== card) {
                overCard.classList.add('drag-over');
            }
        }

        function finishPointerDrag(clientX, clientY) {
            if (!pointerDrag || !pointerDrag.active || !draggedCard) return;

            const overCard = cardFromPoint(clientX, clientY);
            if (overCard && overCard !== draggedCard) {
                insertRelativeToTarget(overCard, clientY);
            } else if (!overCard) {
                grid.appendChild(draggedCard);
            }

            dropCommitted = true;
            persistOrder(orderSnapshot);
            dropCommitted = false;
            orderSnapshot = null;
            cleanup();
        }

        grid.addEventListener('pointerdown', function(e) {
            const handle = e.target.closest('.drag-handle');
            if (!handle || e.pointerType === 'mouse') return;

            const card = handle.closest('.app-card');
            if (!card) return;

            e.preventDefault();
            handle.setPointerCapture(e.pointerId);
            beginPointerDrag(handle, card, e.clientX, e.clientY);
        });

        grid.addEventListener('pointermove', function(e) {
            if (!pointerDrag || !pointerDrag.active || !draggedCard) return;
            e.preventDefault();

            clearDragHighlights();
            const overCard = cardFromPoint(e.clientX, e.clientY);
            if (overCard && overCard !== draggedCard) {
                overCard.classList.add('drag-over');
                insertRelativeToTarget(overCard, e.clientY);
            }
        });

        grid.addEventListener('pointerup', function(e) {
            if (!pointerDrag || !pointerDrag.active) return;
            if (pointerDrag.handle && pointerDrag.handle.hasPointerCapture(e.pointerId)) {
                pointerDrag.handle.releasePointerCapture(e.pointerId);
            }
            finishPointerDrag(e.clientX, e.clientY);
        });

        grid.addEventListener('pointercancel', function() {
            if (pointerDrag && pointerDrag.active && orderSnapshot) {
                restoreOrder(orderSnapshot);
            }
            dropCommitted = false;
            orderSnapshot = null;
            cleanup();
        });

        function persistOrder(rollbackSnapshot) {
            if (persistInFlight) return;

            const cards = getSortableCards();
            const ids = cards.map(function(card) {
                return parseInt(card.getAttribute('data-app-id'), 10);
            }).filter(function(id) {
                return !isNaN(id);
            });

            if (ids.length === 0) return;

            const previousIds = (rollbackSnapshot || []).map(function(item) {
                return parseInt(item.id, 10);
            }).filter(function(id) {
                return !isNaN(id);
            });

            if (previousIds.length === ids.length && previousIds.every(function(id, index) {
                return id === ids[index];
            })) {
                return;
            }

            let csrfToken = '';
            try {
                csrfToken = amudCsrfToken();
            } catch (err) {
                const meta = document.querySelector('meta[name="csrf-token"]');
                csrfToken = meta ? meta.getAttribute('content') : '';
            }

            persistInFlight = true;

            fetch('/apps/reorder', {
                method: 'POST',
                headers: Object.assign(
                    { 'Content-Type': 'application/json' },
                    typeof amudCsrfHeaders === 'function' ? amudCsrfHeaders() : {}
                ),
                body: JSON.stringify({ ids: ids, csrf_token: csrfToken })
            })
            .then(function(res) {
                return res.text().then(function(text) {
                    let data = {};
                    try {
                        data = text ? JSON.parse(text) : {};
                    } catch (parseErr) {
                        data = { success: false, error: text || 'Invalid server response' };
                    }
                    return { ok: res.ok, data: data };
                });
            })
            .then(function(result) {
                if (!result.ok || !result.data.success) {
                    restoreOrder(rollbackSnapshot);
                    const message = (result.data && result.data.error) || 'Failed to save card order.';
                    if (typeof amudShowToast === 'function') {
                        amudShowToast(message, 'error');
                    }
                    return;
                }
                orderSnapshot = snapshotOrder();
            })
            .catch(function() {
                restoreOrder(rollbackSnapshot);
                if (typeof amudShowToast === 'function') {
                    amudShowToast('Network error while saving card order.', 'error');
                }
            })
            .finally(function() {
                persistInFlight = false;
            });
        }
    });
})();
