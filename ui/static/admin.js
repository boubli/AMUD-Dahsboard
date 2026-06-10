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
    global.amudUploadImage = amudUploadImage;
    global.handleFileUpload = handleFileUpload;
})(typeof window !== 'undefined' ? window : globalThis);
