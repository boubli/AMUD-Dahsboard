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
                const alpineData = typeof Alpine !== 'undefined' ? Alpine.$data(el) : null;
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
    global.amudUploadImage = amudUploadImage;
    global.handleFileUpload = handleFileUpload;
})(typeof window !== 'undefined' ? window : globalThis);
