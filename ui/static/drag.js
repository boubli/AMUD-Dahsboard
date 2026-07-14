/**
 * AMUD Drag & Drop — grid-aware card reorder (admin only, handle-only)
 */
(function() {
    'use strict';

    document.addEventListener('DOMContentLoaded', function() {
        if (typeof isAdmin === 'undefined' || !isAdmin) return;

        const grid = document.querySelector('main.bento-grid');
        if (!grid) return;

        const hintBanner = document.getElementById('reorder-hint-banner');

        let draggedCard = null;
        let orderSnapshot = null;
        let persistInFlight = false;
        let pointerDrag = null;
        let allAppIds = [];
        let insertionMarker = null;
        let pendingInsertion = null;

        function parseAppIdsFromDom() {
            return Array.from(grid.querySelectorAll('.app-card[data-app-id]'))
                .map(function(card) { return parseInt(card.getAttribute('data-app-id'), 10); })
                .filter(function(id) { return !isNaN(id); });
        }

        allAppIds = parseAppIdsFromDom();

        function getSortableCards() {
            return Array.from(
                grid.querySelectorAll('.app-card:not(.filter-empty-msg):not(.app-card--empty)')
            ).filter(function(card) {
                return card.style.display !== 'none';
            });
        }

        function getVisualOrderCards() {
            return getSortableCards()
                .filter(function(card) { return card !== draggedCard; })
                .sort(function(a, b) {
                    const ra = a.getBoundingClientRect();
                    const rb = b.getBoundingClientRect();
                    const rowDiff = ra.top - rb.top;
                    if (Math.abs(rowDiff) > 12) return rowDiff;
                    return ra.left - rb.left;
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
            const searchInput = document.getElementById('search-input');
            if (searchInput && searchInput.value.trim()) {
                return 'Clear the search filter before reordering cards.';
            }
            if (window.activeCategoryFilter && window.activeCategoryFilter !== 'all') {
                return 'Switch to the All category before reordering cards.';
            }
            return null;
        }

        function updateReorderHint() {
            if (!hintBanner) return;
            const blocked = reorderBlockedMessage();
            if (blocked) {
                hintBanner.classList.add('is-visible');
            } else {
                hintBanner.classList.remove('is-visible');
            }
        }

        function clearDragHighlights() {
            getSortableCards().forEach(function(c) {
                c.classList.remove('drag-over');
            });
        }

        function ensureMarker() {
            if (!insertionMarker) {
                insertionMarker = document.createElement('div');
                insertionMarker.className = 'drop-insertion-marker';
                insertionMarker.setAttribute('aria-hidden', 'true');
                grid.appendChild(insertionMarker);
            }
            return insertionMarker;
        }

        function hideMarker() {
            if (insertionMarker) {
                insertionMarker.style.display = 'none';
            }
            pendingInsertion = null;
        }

        function sameRow(rectA, rectB) {
            return Math.abs(rectA.top - rectB.top) < Math.min(rectA.height, rectB.height) * 0.45;
        }

        function computeInsertion(clientX, clientY) {
            const cards = getVisualOrderCards();
            const gridRect = grid.getBoundingClientRect();

            if (!cards.length) {
                return { refNode: null, highlightCard: null };
            }

            let overCard = null;
            for (let i = 0; i < cards.length; i++) {
                const rect = cards[i].getBoundingClientRect();
                if (
                    clientX >= rect.left &&
                    clientX <= rect.right &&
                    clientY >= rect.top &&
                    clientY <= rect.bottom
                ) {
                    overCard = { card: cards[i], index: i, rect: rect };
                    break;
                }
            }

            if (overCard) {
                const rect = overCard.rect;
                const midX = rect.left + rect.width / 2;
                const midY = rect.top + rect.height / 2;
                let insertIndex = overCard.index;

                if (clientY > midY) {
                    insertIndex = overCard.index + 1;
                } else if (clientX > midX) {
                    const next = cards[overCard.index + 1];
                    if (next && sameRow(rect, next.getBoundingClientRect())) {
                        insertIndex = overCard.index + 1;
                    }
                }

                return {
                    refNode: cards[insertIndex] || null,
                    highlightCard: overCard.card,
                };
            }

            let bestIndex = cards.length;
            let bestDist = Infinity;

            for (let i = 0; i <= cards.length; i++) {
                let markerY;
                if (i === 0) {
                    markerY = cards[0].getBoundingClientRect().top;
                } else if (i === cards.length) {
                    markerY = cards[cards.length - 1].getBoundingClientRect().bottom;
                } else {
                    const prev = cards[i - 1].getBoundingClientRect();
                    const next = cards[i].getBoundingClientRect();
                    markerY = (prev.bottom + next.top) / 2;
                }

                const dist = Math.abs(clientY - markerY) + Math.abs(clientX - gridRect.left - gridRect.width / 2) * 0.15;
                if (dist < bestDist) {
                    bestDist = dist;
                    bestIndex = i;
                }
            }

            return {
                refNode: cards[bestIndex] || null,
                highlightCard: null,
            };
        }

        function updateMarker(clientX, clientY) {
            pendingInsertion = computeInsertion(clientX, clientY);
            const marker = ensureMarker();
            const cards = getVisualOrderCards();
            const gridRect = grid.getBoundingClientRect();
            const refNode = pendingInsertion.refNode;

            clearDragHighlights();
            if (pendingInsertion.highlightCard) {
                pendingInsertion.highlightCard.classList.add('drag-over');
            }

            if (refNode) {
                const rect = refNode.getBoundingClientRect();
                marker.style.display = 'block';
                marker.style.left = (rect.left - gridRect.left) + 'px';
                marker.style.top = (rect.top - gridRect.top - 4) + 'px';
                marker.style.width = rect.width + 'px';
                return;
            }

            if (cards.length) {
                const last = cards[cards.length - 1].getBoundingClientRect();
                marker.style.display = 'block';
                marker.style.left = '0px';
                marker.style.top = (last.bottom - gridRect.top + 6) + 'px';
                marker.style.width = '100%';
                return;
            }

            marker.style.display = 'none';
        }

        function applyInsertion(insertion) {
            if (!draggedCard || !insertion) return;
            if (insertion.refNode) {
                grid.insertBefore(draggedCard, insertion.refNode);
            } else {
                grid.appendChild(draggedCard);
            }
        }

        function cleanup(restore) {
            hideMarker();
            if (draggedCard) {
                draggedCard.classList.remove('dragging');
            }
            clearDragHighlights();
            grid.classList.remove('reordering');
            if (restore && orderSnapshot) {
                restoreOrder(orderSnapshot);
            }
            draggedCard = null;
            pointerDrag = null;
            orderSnapshot = null;
        }

        function buildFullOrderIds() {
            const visibleOrder = getSortableCards().map(function(card) {
                return parseInt(card.getAttribute('data-app-id'), 10);
            }).filter(function(id) { return !isNaN(id); });

            const hiddenSet = new Set(allAppIds.filter(function(id) {
                return visibleOrder.indexOf(id) === -1;
            }));
            const hiddenIds = allAppIds.filter(function(id) { return hiddenSet.has(id); });
            return visibleOrder.concat(hiddenIds);
        }

        function injectDragHandles() {
            const blocked = reorderBlockedMessage();
            updateReorderHint();
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
                    handle.style.opacity = '0.35';
                    handle.style.cursor = 'not-allowed';
                    handle.style.pointerEvents = 'none';
                } else {
                    handle.style.opacity = '';
                    handle.style.cursor = 'grab';
                    handle.style.pointerEvents = '';
                }
            });
        }

        injectDragHandles();

        const observer = new MutationObserver(function() {
            allAppIds = parseAppIdsFromDom();
            injectDragHandles();
        });
        observer.observe(grid, { childList: true });

        document.addEventListener('amud:category-filter', injectDragHandles);
        document.addEventListener('input', function(e) {
            if (e.target && e.target.id === 'search-input') injectDragHandles();
        });

        function beginDrag(handle, card) {
            const blocked = reorderBlockedMessage();
            if (blocked) {
                if (typeof amudShowToast === 'function') amudShowToast(blocked, 'warning');
                return false;
            }

            draggedCard = card;
            orderSnapshot = snapshotOrder();
            card.classList.add('dragging');
            grid.classList.add('reordering');
            clearDragHighlights();
            return true;
        }

        function finishDrag(clientX, clientY) {
            if (!draggedCard) return;

            const insertion = computeInsertion(clientX, clientY);
            const rollbackSnapshot = orderSnapshot;
            applyInsertion(insertion);
            hideMarker();
            clearDragHighlights();
            draggedCard.classList.remove('dragging');
            grid.classList.remove('reordering');

            draggedCard = null;
            pointerDrag = null;
            orderSnapshot = null;

            persistOrder(rollbackSnapshot);
        }

        function cancelDrag() {
            cleanup(true);
        }

        grid.addEventListener('pointerdown', function(e) {
            const handle = e.target.closest('.drag-handle');
            if (!handle) return;

            const card = handle.closest('.app-card');
            if (!card || reorderBlockedMessage()) return;

            e.preventDefault();
            handle.setPointerCapture(e.pointerId);
            pointerDrag = { handle: handle, active: true, pointerId: e.pointerId };
            beginDrag(handle, card);
        });

        grid.addEventListener('pointermove', function(e) {
            if (!pointerDrag || !pointerDrag.active || !draggedCard) return;
            e.preventDefault();
            updateMarker(e.clientX, e.clientY);
        });

        grid.addEventListener('pointerup', function(e) {
            if (!pointerDrag || !pointerDrag.active) return;
            if (pointerDrag.handle && pointerDrag.handle.hasPointerCapture(e.pointerId)) {
                pointerDrag.handle.releasePointerCapture(e.pointerId);
            }
            pointerDrag.active = false;
            finishDrag(e.clientX, e.clientY);
        });

        grid.addEventListener('pointercancel', function() {
            if (pointerDrag && pointerDrag.active) cancelDrag();
        });

        document.addEventListener('pointerup', function(e) {
            if (pointerDrag && pointerDrag.active && draggedCard) {
                const handle = pointerDrag.handle;
                if (handle && handle.hasPointerCapture(pointerDrag.pointerId)) {
                    handle.releasePointerCapture(pointerDrag.pointerId);
                }
                pointerDrag.active = false;
                finishDrag(e.clientX, e.clientY);
            }
        });

        function persistOrder(rollbackSnapshot) {
            if (persistInFlight) return;

            const ids = buildFullOrderIds();
            if (ids.length === 0) return;

            const previousIds = (rollbackSnapshot || []).map(function(item) {
                return parseInt(item.id, 10);
            }).filter(function(id) { return !isNaN(id); });

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
                    try { data = text ? JSON.parse(text) : {}; }
                    catch (parseErr) { data = { success: false, error: text || 'Invalid server response' }; }
                    return { ok: res.ok, data: data };
                });
            })
            .then(function(result) {
                if (!result.ok || !result.data.success) {
                    restoreOrder(rollbackSnapshot);
                    const message = (result.data && result.data.error) || 'Failed to save card order.';
                    if (typeof amudShowToast === 'function') amudShowToast(message, 'error');
                    return;
                }
                allAppIds = ids.slice();
                if (typeof amudShowToast === 'function') amudShowToast('Card order saved.', 'success');
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
