(function () {
    function parseConfig() {
        var el = document.getElementById('amud-theme-config');
        if (!el) return null;
        try {
            return JSON.parse(el.textContent || '{}');
        } catch (_e) {
            return null;
        }
    }

    function parseMinutes(str) {
        var parts = (str || '').split(':');
        if (parts.length !== 2) return null;
        var h = parseInt(parts[0], 10);
        var m = parseInt(parts[1], 10);
        if (Number.isNaN(h) || Number.isNaN(m) || h < 0 || h > 23 || m < 0 || m > 59) {
            return null;
        }
        return h * 60 + m;
    }

    function isDarkManual(now, lightAt, darkAt) {
        var nowMin = now.getHours() * 60 + now.getMinutes();
        var lightMin = parseMinutes(lightAt);
        var darkMin = parseMinutes(darkAt);
        if (lightMin === null || darkMin === null) return false;
        return nowMin >= darkMin || nowMin < lightMin;
    }

    function applyTheme(mode) {
        document.documentElement.setAttribute('data-theme', mode === 'light' ? 'light' : 'dark');
    }

    var sunCache = { date: '', sunrise: 0, sunset: 0 };

    function fetchSunTimes(lat, lon) {
        var today = new Date().toISOString().slice(0, 10);
        if (sunCache.date === today && sunCache.sunrise) {
            return Promise.resolve(sunCache);
        }
        var url =
            'https://api.open-meteo.com/v1/forecast?latitude=' +
            encodeURIComponent(lat) +
            '&longitude=' +
            encodeURIComponent(lon) +
            '&daily=sunrise,sunset&timezone=auto&forecast_days=1';
        return fetch(url).then(function (res) {
            if (!res.ok) throw new Error('sun fetch failed');
            return res.json();
        }).then(function (data) {
            var rise = new Date(data.daily.sunrise[0]).getTime();
            var set = new Date(data.daily.sunset[0]).getTime();
            sunCache = { date: today, sunrise: rise, sunset: set };
            return sunCache;
        });
    }

    function effectiveTheme(cfg, now) {
        var base = cfg.baseMode === 'light' ? 'light' : 'dark';
        if (!cfg.scheduler || cfg.scheduler === 'off') {
            return base;
        }
        if (cfg.scheduler === 'manual') {
            return isDarkManual(now, cfg.lightAt, cfg.darkAt) ? 'dark' : 'light';
        }
        if (cfg.scheduler === 'sunrise_sunset') {
            var lat = parseFloat(cfg.lat);
            var lon = parseFloat(cfg.lon);
            if (!Number.isFinite(lat) || !Number.isFinite(lon)) {
                return base;
            }
            return fetchSunTimes(lat, lon).then(function (sun) {
                var t = now.getTime();
                return t < sun.sunrise || t >= sun.sunset ? 'dark' : 'light';
            });
        }
        return base;
    }

    function tick() {
        var cfg = parseConfig();
        if (!cfg) return;
        var now = new Date();
        var result = effectiveTheme(cfg, now);
        if (result && typeof result.then === 'function') {
            result.then(applyTheme).catch(function () {
                applyTheme(cfg.baseMode === 'light' ? 'light' : 'dark');
            });
            return;
        }
        applyTheme(result);
    }

    tick();
    setInterval(tick, 60000);
    window.amudApplyScheduledTheme = tick;
    window.amudEffectiveThemeMode = function (cfg) {
        var now = new Date();
        var result = effectiveTheme(cfg || parseConfig() || {}, now);
        if (result && typeof result.then === 'function') {
            return result;
        }
        return Promise.resolve(result);
    };
})();
