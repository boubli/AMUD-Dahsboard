const CACHE_NAME = 'amud-dashboard-v13';
const ASSETS_TO_CACHE = [
  '/static/style.css',
  '/static/AMUD-logo.png',
  '/static/manifest.json',
  '/static/vendor/alpine.min.js',
  '/static/vendor/lucide.min.js',
  '/static/admin.js',
  '/static/dashboard-live.js',
  '/static/theme-scheduler.js',
  '/static/app-search.js',
  '/static/shortcuts.js',
  '/static/embed-tabs.js'
];

globalThis.addEventListener('install', event => {
  event.waitUntil(
    caches.open(CACHE_NAME)
      .then(cache => cache.addAll(ASSETS_TO_CACHE))
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

  // Never cache HTML documents — avoids stale broken inline scripts after upgrades.
  if (request.mode === 'navigate') {
    event.respondWith(fetch(request));
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
