const CACHE_NAME = 'amud-dashboard-v38';
const ASSETS_TO_CACHE = [
  '/static/style.css',
  '/static/theme-guards.css',
  '/static/theme-engine.js',
  '/static/theme-picker.js',
  '/static/AMUD-logo.png',
  '/static/pwa-icon-192.png',
  '/static/pwa-icon-512.png',
  '/static/offline.html',
  '/static/pwa-install.js',
  '/static/themes/manifest.json',
  '/static/themes/_shared.css',
  '/static/vendor/alpine.min.js',
  '/static/vendor/lucide.min.js',
  '/static/vendor/three.min.js',
  '/static/themes/backgrounds/taghawsa-bg.js',
  '/static/admin.js',
  '/static/dashboard-live.js',
  '/static/theme-scheduler.js',
  '/static/app-search.js',
  '/static/shortcuts.js',
  '/static/embed-tabs.js',
  '/static/drag.js'
];

function parseManifestJson(res) {
  if (!res.ok) return null;
  return res.json();
}

function cacheThemePreviews() {
  return fetch('/static/themes/manifest.json', { cache: 'no-store' })
    .then(parseManifestJson)
    .then(function (manifest) {
      if (!manifest?.themes) return [];
      return manifest.themes
        .map(function (t) { return t.preview; })
        .filter(function (p) { return p?.startsWith('/'); });
    })
    .catch(function () { return []; });
}

globalThis.addEventListener('install', event => {
  event.waitUntil(
    cacheThemePreviews()
      .then(function (previews) {
        return caches.open(CACHE_NAME).then(function (cache) {
          return cache.addAll(ASSETS_TO_CACHE.concat(previews.slice(0, 40)));
        });
      })
      .then(() => globalThis.skipWaiting())
  );
});

globalThis.addEventListener('activate', event => {
  event.waitUntil(
    caches.keys().then(keys => {
      return Promise.all(
        keys
          .filter(key => key !== CACHE_NAME)
          .map(key => caches.delete(key))
      );
    }).then(() => globalThis.clients.claim())
  );
});

globalThis.addEventListener('fetch', event => {
  const request = event.request;
  const url = new URL(request.url);

  if (url.pathname.startsWith('/ws') || url.pathname.startsWith('/api/')) {
    return;
  }

  if (request.mode === 'navigate') {
    event.respondWith(
      fetch(request).catch(() =>
        caches.match('/static/offline.html').then((cached) => cached || Response.error())
      )
    );
    return;
  }

  if (url.pathname.startsWith('/static/')) {
    event.respondWith(
      caches.match(request).then(cached => {
        return cached || fetch(request).then(response => {
          const copy = response.clone();
          caches.open(CACHE_NAME).then(cache => cache.put(request, copy));
          return response;
        });
      })
    );
  }
});
