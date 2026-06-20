(function (global) {
    function amudCsrfToken() {
        return document.querySelector('meta[name="csrf-token"]')?.getAttribute('content') || '';
    }

    function amudCsrfHeaders(extra) {
        return Object.assign({ 'X-CSRF-Token': amudCsrfToken() }, extra || {});
    }

    function escapeHtml(value) {
        const div = document.createElement('div');
        div.textContent = value == null ? '' : String(value);
        return div.innerHTML;
    }

    function appendTableCell(tr, text, options) {
        const td = document.createElement('td');
        const opts = options || {};
        if (opts.style) td.setAttribute('style', opts.style);
        if (opts.className) td.className = opts.className;
        if (opts.code) {
            const code = document.createElement('code');
            code.style.fontSize = '0.75rem';
            code.textContent = text == null ? '' : String(text);
            td.appendChild(code);
        } else if (opts.strong) {
            const strong = document.createElement('strong');
            strong.textContent = text == null ? '' : String(text);
            td.appendChild(strong);
        } else {
            td.textContent = text == null ? '' : String(text);
        }
        tr.appendChild(td);
        return td;
    }

    function setTableMessage(tbody, colspan, message, options) {
        const opts = options || {};
        tbody.replaceChildren();
        const tr = document.createElement('tr');
        const td = document.createElement('td');
        td.colSpan = colspan;
        td.textContent = message;
        if (opts.style) td.setAttribute('style', opts.style);
        if (opts.className) td.className = opts.className;
        tr.appendChild(td);
        tbody.appendChild(tr);
    }

    function setButtonLoading(button, loading, sizeRem) {
        if (!button) return;
        const size = sizeRem || '0.75';
        if (loading) {
            if (!button._amudOrigNodes) {
                button._amudOrigNodes = Array.from(button.childNodes).map((node) => node.cloneNode(true));
            }
            button.replaceChildren();
            const icon = document.createElement('i');
            icon.dataset.lucide = 'loader-2';
            icon.className = 'animate-spin';
            icon.style.width = `${size}rem`;
            icon.style.height = `${size}rem`;
            button.appendChild(icon);
            if (typeof lucide !== 'undefined') lucide.createIcons();
            return;
        }
        if (!button._amudOrigNodes) return;
        button.replaceChildren(...button._amudOrigNodes.map((node) => node.cloneNode(true)));
        delete button._amudOrigNodes;
        if (typeof lucide !== 'undefined') lucide.createIcons();
    }

    function createMetricBlock(value, label, options) {
        const opts = options || {};
        const block = document.createElement('div');
        block.className = 'metric-block';
        const valueEl = document.createElement('span');
        valueEl.className = 'metric-value';
        if (opts.valueStyle) valueEl.setAttribute('style', opts.valueStyle);
        valueEl.textContent = value == null ? '' : String(value);
        const labelEl = document.createElement('span');
        labelEl.className = 'metric-label';
        labelEl.textContent = label == null ? '' : String(label);
        block.appendChild(valueEl);
        block.appendChild(labelEl);
        return block;
    }

    function setMetricsGrid(grid, blocks) {
        if (!grid) return;
        grid.replaceChildren();
        blocks.forEach((block) => {
            grid.appendChild(createMetricBlock(block.value, block.label, block));
        });
        grid.dataset.amudLiveMetrics = '1';
    }

    function shouldUpdateLxcMetrics(grid) {
        if (!grid) return false;
        if (grid.closest('.integration-widget')) return false;
        if (grid.id === 'ha-metrics-grid') return false;
        return true;
    }

    function setFilterEmptyMessage(container) {
        if (!container) return;
        container.replaceChildren();
        const line1 = document.createElement('p');
        line1.style.fontWeight = '600';
        line1.style.color = 'var(--text-secondary)';
        line1.textContent = 'No apps in this category';
        const line2 = document.createElement('p');
        line2.style.fontSize = '0.8rem';
        line2.style.color = 'var(--text-muted)';
        line2.style.marginTop = '0.5rem';
        line2.textContent = 'Switch tabs or add services under this category.';
        container.appendChild(line1);
        container.appendChild(line2);
    }

    function amudUploadImage(file, onSuccess, onError) {
        if (!file) return;
        if (file.size > 5 * 1024 * 1024) {
            const err = new Error('File size exceeds 5MB limit!');
            if (onError) onError(err);
            else alert(err.message);
            return;
        }
        const formData = new FormData();
        formData.append('image', file);
        fetch('/admin/upload', {
            method: 'POST',
            headers: amudCsrfHeaders(),
            body: formData
        })
            .then(res => {
                if (!res.ok) return res.text().then(text => { throw new Error(text); });
                return res.json();
            })
            .then(data => {
                if (onSuccess) onSuccess(data.url);
            })
            .catch(err => {
                if (onError) onError(err);
                else alert('Upload failed: ' + err.message);
            });
    }

    function handleFileUpload(event, type) {
        const file = event.target.files[0];
        amudUploadImage(file, function (url) {
            if (type === 'logo') {
                const el = document.querySelector('input[name="app_logo"]');
                if (el) { el.value = url; el.dispatchEvent(new Event('input')); }
            } else if (type === 'bg') {
                const el = document.querySelector('input[name="custom_bg_url"]');
                if (el) { el.value = url; el.dispatchEvent(new Event('input')); }
            } else if (type === 'appIcon' || type === 'editAppIcon') {
                const el = document.querySelector('body');
                let alpineData = null;
                if (typeof Alpine !== 'undefined') {
                    alpineData = Alpine.$data(el);
                }
                if (alpineData) {
                    if (type === 'appIcon') alpineData.appIconUrl = url;
                    else alpineData.editApp.icon = url;
                }
            }
        });
    }

    global.amudCsrfToken = amudCsrfToken;
    global.amudCsrfHeaders = amudCsrfHeaders;
    global.escapeHtml = escapeHtml;
    global.appendTableCell = appendTableCell;
    global.setTableMessage = setTableMessage;
    global.setButtonLoading = setButtonLoading;
    global.createMetricBlock = createMetricBlock;
    global.setMetricsGrid = setMetricsGrid;
    global.shouldUpdateLxcMetrics = shouldUpdateLxcMetrics;
    global.setFilterEmptyMessage = setFilterEmptyMessage;
    global.amudUploadImage = amudUploadImage;
    global.handleFileUpload = handleFileUpload;
})(globalThis.window ?? globalThis);
