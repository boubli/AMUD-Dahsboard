/** Integration setup wizard helpers (v1.8.0). */
(function () {
  async function testIntegration(integrationType, url, apiKey) {
    const form = new FormData();
    form.append('integration_type', integrationType || '');
    form.append('url', url || '');
    form.append('api_key', apiKey || '');
    const res = await fetch('/api/integrations/test', {
      method: 'POST',
      headers: typeof amudCsrfHeaders === 'function' ? amudCsrfHeaders() : {},
      body: form,
    });
    const data = await res.json().catch(() => ({}));
    return { ok: res.ok, data };
  }

  async function loadCustomApiTemplates() {
    const res = await fetch('/api/integrations/custom-api/templates');
    if (!res.ok) return [];
    const data = await res.json().catch(() => ({}));
    return data.templates || [];
  }

  window.amudIntegrationWizard = { testIntegration, loadCustomApiTemplates };
})();
