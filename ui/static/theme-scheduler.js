(function () {
    function parseConfig() {
        const el = document.getElementById('amud-theme-config');
        if (!el) return null;
        try {
            return JSON.parse(el.textContent || '{}');
        } catch (err) {
            console.warn('theme config parse failed:', err);
            return null;
        }
    }

    function parseMinutes(str) {
        const parts = (str || '').split(':');
        if (parts.length !== 2) return null;
        const h = Number.parseInt(parts[0], 10);
        const m = Number.parseInt(parts[1], 10);
        if (Number.isNaN(h) || Number.isNaN(m) || h < 0 || h > 23 || m < 0 || m > 59) {
            return null;
        }
        return h * 60 + m;
    }

    function isDarkManual(now, lightAt, darkAt) {
        const nowMin = now.getHours() * 60 + now.getMinutes();
        const lightMin = parseMinutes(lightAt);
        const darkMin = parseMinutes(darkAt);
        if (lightMin === null || darkMin === null) return false;
        return nowMin >= darkMin || nowMin < lightMin;
    }

    function applyTheme(mode) {
        document.documentElement.dataset.theme = mode === 'light' ? 'light' : 'dark';
    }

    let sunCache = { date: '', sunrise: 0, sunset: 0 };

    function fetchSunTimes(lat, lon) {
        const today = new Date().toISOString().slice(0, 10);
        if (sunCache.date === today && sunCache.sunrise) {
            return Promise.resolve(sunCache);
        }
        const url =
            'https://api.open-meteo.com/v1/forecast?latitude=' +
            encodeURIComponent(lat) +
            '&longitude=' +
            encodeURIComponent(lon) +
            '&daily=sunrise,sunset&timezone=auto&forecast_days=1';
        return fetch(url).then(function (res) {
            if (!res.ok) throw new Error('sun fetch failed');
            return res.json();
        }).then(function (data) {
            const rise = new Date(data.daily.sunrise[0]).getTime();
            const set = new Date(data.daily.sunset[0]).getTime();
            sunCache = { date: today, sunrise: rise, sunset: set };
            return sunCache;
        });
    }

    function effectiveTheme(cfg, now) {
        const base = cfg.baseMode === 'light' ? 'light' : 'dark';
        if (!cfg.scheduler || cfg.scheduler === 'off') {
            return Promise.resolve(base);
        }
        if (cfg.scheduler === 'manual') {
            return Promise.resolve(isDarkManual(now, cfg.lightAt, cfg.darkAt) ? 'dark' : 'light');
        }
        if (cfg.scheduler === 'sunrise_sunset') {
            const lat = Number.parseFloat(cfg.lat);
            const lon = Number.parseFloat(cfg.lon);
            if (!Number.isFinite(lat) || !Number.isFinite(lon)) {
                return Promise.resolve(base);
            }
            return fetchSunTimes(lat, lon).then(function (sun) {
                const t = now.getTime();
                return t < sun.sunrise || t >= sun.sunset ? 'dark' : 'light';
            });
        }
        return Promise.resolve(base);
    }

    function tick() {
        const cfg = parseConfig();
        if (!cfg) return;
        const now = new Date();
        effectiveTheme(cfg, now)
            .then(applyTheme)
            .catch(function (err) {
                console.warn('scheduled theme apply failed:', err);
                applyTheme(cfg.baseMode === 'light' ? 'light' : 'dark');
            });
    }

    tick();
    setInterval(tick, 60000);
    globalThis.amudApplyScheduledTheme = tick;
    globalThis.amudEffectiveThemeMode = function (cfg) {
        const now = new Date();
        return effectiveTheme(cfg || parseConfig() || {}, now);
    };
})();
