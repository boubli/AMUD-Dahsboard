/**
 * Searchable logo gallery for Add/Edit App icon fields.
 * Expects #add-app-icon / #edit-app-icon inputs and mounts pickers beside them.
 */
(function (global) {
  'use strict';

  var logosCache = null;
  var logosPromise = null;

  function loadLogos() {
    if (logosCache) return Promise.resolve(logosCache);
    if (logosPromise) return logosPromise;
    logosPromise = fetch('/api/logos', { credentials: 'same-origin', cache: 'no-store' })
      .then(function (res) { return res.ok ? res.json() : { logos: [] }; })
      .then(function (data) {
        logosCache = Array.isArray(data.logos) ? data.logos : [];
        return logosCache;
      })
      .catch(function () {
        logosCache = [];
        return logosCache;
      });
    return logosPromise;
  }

  function mountPicker(input) {
    if (!input || input.dataset.logoPickerMounted === '1') return;
    input.dataset.logoPickerMounted = '1';

    var wrap = document.createElement('div');
    wrap.className = 'amud-logo-picker';
    wrap.style.cssText = 'margin-top:0.65rem;';

    var search = document.createElement('input');
    search.type = 'search';
    search.className = 'form-control';
    search.placeholder = 'Search icons…';
    search.style.marginBottom = '0.5rem';
    wrap.appendChild(search);

    var preview = document.createElement('div');
    preview.style.cssText = 'display:flex;align-items:center;gap:0.5rem;margin-bottom:0.5rem;min-height:1.75rem;';
    var previewImg = document.createElement('img');
    previewImg.alt = '';
    previewImg.style.cssText = 'width:1.5rem;height:1.5rem;object-fit:contain;display:none;';
    var previewLabel = document.createElement('span');
    previewLabel.style.cssText = 'font-size:0.78rem;color:var(--text-muted);';
    preview.appendChild(previewImg);
    preview.appendChild(previewLabel);
    wrap.appendChild(preview);

    var rail = document.createElement('div');
    rail.className = 'amud-logo-picker-rail';
    rail.style.cssText = 'display:flex;gap:0.45rem;overflow-x:auto;padding:0.35rem 0;scroll-snap-type:x mandatory;';
    wrap.appendChild(rail);

    function updatePreview(value) {
      var v = (value || '').trim();
      if (!v) {
        previewImg.style.display = 'none';
        previewLabel.textContent = '';
        return;
      }
      var url = v.startsWith('http') || v.startsWith('/') ? v : '/static/logos/' + v + '.svg';
      previewImg.src = url;
      previewImg.style.display = 'block';
      previewLabel.textContent = v;
      previewImg.onerror = function () {
        previewImg.style.display = 'none';
        previewLabel.textContent = v + ' (custom / missing preview)';
      };
    }

    function render(filter) {
      rail.replaceChildren();
      var q = (filter || '').toLowerCase().trim();
      var list = (logosCache || []).filter(function (item) {
        return !q || (item.id || '').toLowerCase().indexOf(q) !== -1;
      }).slice(0, 120);
      list.forEach(function (item) {
        var btn = document.createElement('button');
        btn.type = 'button';
        btn.title = item.id;
        btn.style.cssText = 'flex:0 0 auto;width:2.4rem;height:2.4rem;padding:0.3rem;border-radius:8px;border:1px solid var(--border-card);background:rgba(255,255,255,0.04);cursor:pointer;scroll-snap-align:start;';
        var img = document.createElement('img');
        img.src = item.url;
        img.alt = item.id;
        img.style.cssText = 'width:100%;height:100%;object-fit:contain;';
        btn.appendChild(img);
        btn.addEventListener('click', function () {
          input.value = item.id;
          input.dispatchEvent(new Event('input', { bubbles: true }));
          if (input.__x_model) {
            // Alpine x-model may need Alpine set — fall back to property
          }
          try {
            var root = input.closest('[x-data]');
            if (root && global.Alpine) {
              var data = Alpine.$data(root);
              if (data && 'appIconUrl' in data) data.appIconUrl = item.id;
              if (data && data.editApp && 'icon' in data.editApp) data.editApp.icon = item.id;
            }
          } catch (e) {}
          updatePreview(item.id);
        });
        rail.appendChild(btn);
      });
      if (!list.length) {
        var empty = document.createElement('span');
        empty.style.cssText = 'font-size:0.78rem;color:var(--text-muted);padding:0.35rem;';
        empty.textContent = 'No matching icons';
        rail.appendChild(empty);
      }
    }

    search.addEventListener('input', function () { render(search.value); });
    input.addEventListener('input', function () { updatePreview(input.value); });

    var parent = input.closest('.form-group') || input.parentElement;
    parent.appendChild(wrap);

    loadLogos().then(function () {
      updatePreview(input.value);
      render('');
    });
  }

  function initLogoPickers() {
    ['add-app-icon', 'edit-app-icon'].forEach(function (id) {
      var el = document.getElementById(id);
      if (el) mountPicker(el);
    });
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initLogoPickers);
  } else {
    initLogoPickers();
  }

  global.amudInitLogoPickers = initLogoPickers;
})(typeof window !== 'undefined' ? window : globalThis);
