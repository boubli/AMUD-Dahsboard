(function () {
  const STORAGE_KEY = 'amud-pwa-install-dismissed';
  let deferredPrompt = null;

  function isStandalone() {
    return globalThis.matchMedia('(display-mode: standalone)').matches
      || globalThis.navigator.standalone === true;
  }

  function shouldShowBanner() {
    if (isStandalone()) return false;
    if (globalThis.localStorage.getItem(STORAGE_KEY) === '1') return false;
    return Boolean(deferredPrompt);
  }

  function removeBanner() {
    const banner = document.getElementById('amud-pwa-install-banner');
    if (banner) banner.remove();
  }

  function dismissBanner() {
    globalThis.localStorage.setItem(STORAGE_KEY, '1');
    removeBanner();
  }

  function renderBanner() {
    if (!shouldShowBanner() || document.getElementById('amud-pwa-install-banner')) return;

    const banner = document.createElement('div');
    banner.id = 'amud-pwa-install-banner';
    banner.className = 'pwa-install-banner glass-panel';
    banner.setAttribute('role', 'region');
    banner.setAttribute('aria-label', 'Install AMUD app');
    banner.innerHTML = `
      <div class="pwa-install-banner__copy">
        <strong>Install AMUD</strong>
        <span>Add the dashboard to your home screen for quick access.</span>
      </div>
      <div class="pwa-install-banner__actions">
        <button type="button" class="glass-panel topbar-action pwa-install-banner__install">Install</button>
        <button type="button" class="glass-panel topbar-action pwa-install-banner__dismiss" aria-label="Dismiss install prompt">Not now</button>
      </div>
    `;

    banner.querySelector('.pwa-install-banner__dismiss')?.addEventListener('click', dismissBanner);
    banner.querySelector('.pwa-install-banner__install')?.addEventListener('click', async () => {
      if (!deferredPrompt) return;
      deferredPrompt.prompt();
      await deferredPrompt.userChoice.catch((error) => {
        console.debug('AMUD PWA install prompt closed:', error);
      });
      deferredPrompt = null;
      dismissBanner();
    });

    document.body.prepend(banner);
  }

  globalThis.addEventListener('beforeinstallprompt', (event) => {
    event.preventDefault();
    deferredPrompt = event;
    renderBanner();
  });

  globalThis.addEventListener('appinstalled', () => {
    deferredPrompt = null;
    dismissBanner();
  });

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', renderBanner);
  } else {
    renderBanner();
  }
})();
