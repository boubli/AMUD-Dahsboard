(function () {
    document.addEventListener('click', function (e) {
        const link = e.target.closest('a[data-embed-mode="tab"]');
        if (!link) return;
        e.preventDefault();
        const panel = document.getElementById('embed-panel');
        const frame = document.getElementById('embed-panel-frame');
        if (!panel || !frame) return;
        frame.src = link.getAttribute('href') || '';
        panel.style.display = 'block';
        panel.scrollIntoView({ behavior: 'smooth', block: 'start' });
    });

    document.addEventListener('keydown', function (e) {
        if (e.key !== 'Escape') return;
        const panel = document.getElementById('embed-panel');
        const frame = document.getElementById('embed-panel-frame');
        if (!panel || panel.style.display === 'none') return;
        panel.style.display = 'none';
        if (frame) frame.src = 'about:blank';
    });
})();
