/**
 * Live dashboard: clock, WebSocket telemetry, GPU, status badges.
 * Loaded from /static so fixes ship with ui.tar.gz without relying on cached inline HTML.
 */
(function (global) {
    'use strict';

    function readConfig() {
        const el = document.getElementById('amud-live-config');
        if (!el) return { isAdmin: false, hideTelemetry: false };
        try {
            return JSON.parse(el.textContent || '{}');
        } catch {
            return { isAdmin: false, hideTelemetry: false };
        }
    }

    const config = readConfig();
    const isAdmin = !!config.isAdmin;
    let latestAppStatuses = {};

    function setWsStatus(state) {
        const pill = document.getElementById('ws-status-pill');
        const text = document.getElementById('ws-status-text');
        if (!pill || !text) return;
        pill.classList.remove('ws-live', 'ws-connecting', 'ws-offline');
        if (state === 'live') {
            pill.classList.add('ws-live');
            text.innerText = 'Live';
        } else if (state === 'offline') {
            pill.classList.add('ws-offline');
            text.innerText = 'Offline';
        } else {
            pill.classList.add('ws-connecting');
            text.innerText = 'Reconnecting';
        }
    }

    function setDashboardText(id, text) {
        const el = document.getElementById(id);
        if (el) el.innerText = text;
    }

    function setDashboardBar(id, pct) {
        const el = document.getElementById(id);
        if (el) el.style.width = `${Math.max(0, Math.min(100, pct))}%`;
    }

    function parseRateToBps(rateText) {
        const s = String(rateText || '').trim();
        const m = s.match(/^([\d.]+)\s*([KMG]?B)\/s$/i);
        if (!m) return 0;
        const v = parseFloat(m[1]);
        const unit = (m[2] || 'B').toUpperCase();
        const mult = unit === 'GB' ? 1024 * 1024 * 1024 : unit === 'MB' ? 1024 * 1024 : unit === 'KB' ? 1024 : 1;
        return v * mult;
    }

    function formatRateFromBps(bytes) {
        if (!bytes || bytes <= 0) return '—';
        const units = ['B/s', 'KB/s', 'MB/s', 'GB/s'];
        let value = bytes;
        let idx = 0;
        while (value >= 1024 && idx < units.length - 1) {
            value /= 1024;
            idx++;
        }
        return `${value.toFixed(value < 10 && idx > 0 ? 1 : 0)} ${units[idx]}`;
    }

    function normalizeToken(value) {
        return String(value || '').toLowerCase().replace(/[^a-z0-9]/g, '');
    }

    function findContainerByNames(containers, names) {
        const tokens = names
            .flatMap(n => String(n || '').toLowerCase().split(/[^a-z0-9]+/))
            .map(normalizeToken)
            .filter(Boolean);
        return containers.find(lxc => {
            const cname = normalizeToken(lxc.name);
            return tokens.some(name =>
                cname === name ||
                name.includes(cname) ||
                cname.includes(name)
            );
        });
    }

    function containerNamesForCard(card) {
        const aliases = card.getAttribute('data-container-aliases') || '';
        const appName = card.getAttribute('data-app-name') || '';
        return [appName, ...aliases.split(/\s+/).filter(Boolean)];
    }

    function guestAvailabilityStatus(status) {
        const normalized = (status || '').toLowerCase();
        if (['running', 'online'].includes(normalized)) {
            return 'online';
        }
        return 'offline';
    }

    function styleStatusBadge(badge, status, latencyMs = null) {
        const normalized = (status || '').toLowerCase();
        badge.innerText = (status || 'UNKNOWN').toUpperCase();

        const isHealthy = ['running', 'online'].includes(normalized);
        const isWarn = ['not configured', 'checking', 'unknown'].includes(normalized);
        badge.classList.remove('status-online', 'status-offline', 'status-checking', 'status-unknown', 'ms');
        if (isHealthy) {
            badge.classList.add('status-online');
            if (isAdmin && latencyMs !== null) {
                badge.title = `Online — ${latencyMs} ms`;
            }
        } else if (isWarn) {
            badge.classList.add(normalized === 'checking' ? 'status-checking' : 'status-unknown');
        } else {
            badge.classList.add('status-offline');
        }
        badge.dataset.lastStatus = normalized || 'unknown';
    }

    function updateHostTelemetry(sys) {
        if (!sys) return;
        setDashboardText('val-pve-cpu', `${sys.cpu_usage ?? 0}%`);
        setDashboardBar('bar-pve-cpu', sys.cpu_usage ?? 0);
        setDashboardText('val-pve-mem', `${sys.ram_usage ?? 0}%`);
        setDashboardBar('bar-pve-mem', sys.ram_usage ?? 0);
        setDashboardText('val-cpu-model', sys.cpu_model ? String(sys.cpu_model).slice(0, 22) : 'Host CPU');
        setDashboardText('val-cpu-temp', (sys.cpu_temp && sys.cpu_temp > 0) ? `${Number(sys.cpu_temp).toFixed(0)}°C` : '—');
        setDashboardText('val-ram-used', `${(sys.ram_used_gb ?? 0).toFixed(1)} GB Used`);
        setDashboardText('val-ram-total', `${(sys.ram_total_gb ?? 0).toFixed(1)} GB Total`);

        const freeGb = (sys.disk_total_gb ?? 0) - (sys.disk_used_gb ?? 0);
        setDashboardText('val-nas-free', `${freeGb.toFixed(1)} GB Free`);
        setDashboardBar('bar-nas-usage', sys.disk_usage ?? 0);
        setDashboardText('val-nas-used', `${(sys.disk_used_gb ?? 0).toFixed(1)} GB Used`);
        setDashboardText('val-disk-total', `${(sys.disk_total_gb ?? 0).toFixed(1)} GB Total`);

        const gpuName = (sys.gpu_name || '').trim();
        const hasGpu = gpuName.length > 0 && (sys.gpu_usage ?? -1) >= 0;
        const gpuCard = document.getElementById('gpu-card');
        const telemetryRow = document.getElementById('telemetry-section');
        if (gpuCard) gpuCard.style.display = hasGpu ? '' : 'none';
        if (telemetryRow) telemetryRow.classList.toggle('has-gpu', hasGpu);
        if (hasGpu) {
            setDashboardText('val-gpu-usage', `${sys.gpu_usage ?? 0}%`);
            setDashboardBar('bar-gpu-usage', sys.gpu_usage ?? 0);
            setDashboardText('val-gpu-name', gpuName.slice(0, 18));
            const used = (sys.gpu_mem_used_mb ?? 0) / 1024;
            const total = (sys.gpu_mem_total_mb ?? 0) / 1024;
            setDashboardText('val-gpu-vram', total > 0 ? `VRAM ${used.toFixed(1)}/${total.toFixed(1)} GB` : 'VRAM —');
        }
    }

    function updateStreamStatusBadges(containers) {
        document.querySelectorAll('[data-stream-app]').forEach(badge => {
            const names = badge.getAttribute('data-stream-app')
                .toLowerCase()
                .split(/\s+/)
                .filter(Boolean);
            const match = findContainerByNames(containers, names);
            if (!match) return;
            styleStatusBadge(badge, match.status);
        });
    }

    function updateAppUrlStatuses(statuses) {
        latestAppStatuses = statuses || {};
        document.querySelectorAll('.app-card').forEach(card => {
            const appName = card.getAttribute('data-app-name');
            if (!appName) return;

            const status = latestAppStatuses[appName] || latestAppStatuses[appName.toLowerCase()];
            const badge = card.querySelector('.status-badge');
            if (!badge || badge.dataset.containerManaged === 'true') return;
            if (!status) {
                styleStatusBadge(badge, 'UNKNOWN');
                badge.title = 'No URL health status has been received for this app';
                return;
            }

            badge.title = isAdmin ? 'URL health check' : 'Public availability check';
            styleStatusBadge(badge, status.status, status.latency_ms);
        });
    }

    function updateMediaStream(service, stream) {
        const normalized = service === 'plex' ? 'plex' : 'jellyfin';
        const track = document.getElementById(`${normalized}-track`);
        const timer = document.getElementById(`${normalized}-timer`);
        const progress = document.getElementById(`${normalized}-progress`);
        const badge = document.querySelector(`[data-stream-service="${normalized}"]`)
            || document.querySelector(`[data-stream-service="emby"]`);

        if (track) track.innerText = stream.title;
        if (timer) timer.innerText = stream.active ? `${stream.current_time} / ${stream.total_time}` : '-';
        if (progress) progress.style.width = `${stream.progress_percent || 0}%`;
        if (badge && stream.status) {
            styleStatusBadge(badge, stream.status);
        }
    }

    function onWsMessage(event) {
        try {
            const data = JSON.parse(event.data);

            if (data.system) {
                const sys = data.system;
                updateHostTelemetry(sys);

                const containers = (sys.lxc_containers && sys.lxc_containers.length > 0) ? sys.lxc_containers : [];
                if (containers.length > 0) {
                    updateStreamStatusBadges(containers);
                }
                document.querySelectorAll('.app-card').forEach(card => {
                    const match = findContainerByNames(containers, containerNamesForCard(card));
                    const isHostAgentApp = card.getAttribute('data-host-agent-app') === 'true';

                    if (match) {
                        const ctrlContainer = card.querySelector('.container-controls');
                        if (ctrlContainer) {
                            const isDocker = match.vmid < 0;
                            const provider = isDocker ? 'docker' : 'lxc';
                            const containerId = isDocker ? match.name : match.vmid;

                            ctrlContainer.setAttribute('data-id', containerId);
                            ctrlContainer.setAttribute('data-provider', provider);

                            if (!ctrlContainer.classList.contains('loading')) {
                                ctrlContainer.style.display = 'inline-flex';

                                const isRunning = match.status === 'running';
                                const startBtn = ctrlContainer.querySelector('.btn-ctrl.start');
                                const stopBtn = ctrlContainer.querySelector('.btn-ctrl.stop');
                                const restartBtn = ctrlContainer.querySelector('.btn-ctrl.restart');

                                if (startBtn) startBtn.style.display = isRunning ? 'none' : 'inline-flex';
                                if (stopBtn) stopBtn.style.display = isRunning ? 'inline-flex' : 'none';
                                if (restartBtn) restartBtn.style.display = isRunning ? 'inline-flex' : 'none';
                            }
                        }

                        const badgeContainer = card.querySelector('.app-card-badges');
                        if (badgeContainer) {
                            const existingBadge = badgeContainer.querySelector('.status-badge');
                            if (existingBadge && !existingBadge.classList.contains('ms')) {
                                existingBadge.dataset.containerManaged = 'true';
                                existingBadge.title = isAdmin
                                    ? 'Container runtime status'
                                    : 'Service availability';
                                const badgeStatus = isAdmin
                                    ? match.status
                                    : guestAvailabilityStatus(match.status);
                                styleStatusBadge(existingBadge, badgeStatus);
                            }
                        }

                        const metricsGrid = card.querySelector('[data-lxc-metrics]');
                        if (typeof global.shouldUpdateLxcMetrics === 'function' && global.shouldUpdateLxcMetrics(metricsGrid)) {
                            const cpuPct = match.cpu ? `${(match.cpu * 100).toFixed(1)}%` : '0%';
                            const ramPct = match.mem && match.maxmem
                                ? `${((match.mem / match.maxmem) * 100).toFixed(1)}%`
                                : '0%';
                            global.setMetricsGrid(metricsGrid, [
                                {
                                    value: cpuPct,
                                    label: 'CPU',
                                    valueStyle: match.cpu > 0.8 ? 'color: #ef4444' : '',
                                },
                                { value: ramPct, label: 'RAM' },
                            ]);
                        }
                    }
                    if (!match && isHostAgentApp) {
                        const badgeContainer = card.querySelector('.app-card-badges');
                        if (badgeContainer) {
                            const existingBadge = badgeContainer.querySelector('.status-badge');
                            if (existingBadge && !existingBadge.classList.contains('ms')) {
                                existingBadge.dataset.containerManaged = 'true';
                                existingBadge.title = isAdmin
                                    ? 'Proxmox host agent status'
                                    : 'Service availability';
                                const badgeStatus = isAdmin
                                    ? (data.agent_connected ? 'running' : 'offline')
                                    : guestAvailabilityStatus(data.agent_connected ? 'running' : 'offline');
                                styleStatusBadge(existingBadge, badgeStatus);
                            }
                        }
                        const metricsGrid = card.querySelector('[data-lxc-metrics]');
                        if (typeof global.shouldUpdateLxcMetrics === 'function' && global.shouldUpdateLxcMetrics(metricsGrid)) {
                            global.setMetricsGrid(metricsGrid, [
                                { value: `${sys.cpu_usage ?? 0}%`, label: 'CPU' },
                                { value: `${sys.ram_usage ?? 0}%`, label: 'RAM' },
                            ]);
                        }
                    }
                });
            }

            if (data.app_statuses) {
                updateAppUrlStatuses(data.app_statuses);
            }

            if (data.network) {
                const net = data.network;
                const tx = parseRateToBps(net.internal_tx) + parseRateToBps(net.external_tx);
                const rx = parseRateToBps(net.internal_rx) + parseRateToBps(net.external_rx);
                const total = tx + rx;
                setDashboardText('val-net-up', `↑ ${formatRateFromBps(tx)}`);
                setDashboardText('val-net-down', `↓ ${formatRateFromBps(rx)}`);
                setDashboardText('val-net-total', formatRateFromBps(total));
                setDashboardBar('bar-net-total', Math.min(100, total / (125 * 1024 * 1024) * 100));
            }

            if (typeof data.agent_connected === 'boolean') {
                document.querySelectorAll('.telemetry-card').forEach(card => {
                    card.classList.toggle('agent-offline', !data.agent_connected);
                });
            }

            if (data.streams) {
                if (data.streams.plex) {
                    updateMediaStream('plex', data.streams.plex);
                }
                const jellyfinStream = data.streams.jellyfin || data.streams.emby;
                if (jellyfinStream) {
                    updateMediaStream('jellyfin', jellyfinStream);
                }
            }

            if (data.smart_home) {
                const sh = data.smart_home;
                const haLights = document.getElementById('ha-lights');
                if (haLights) haLights.innerText = sh.lights_on;
                const haSwitches = document.getElementById('ha-switches');
                if (haSwitches) haSwitches.innerText = sh.switches_on;
                const haTemp = document.getElementById('ha-temp');
                if (haTemp) haTemp.innerText = sh.avg_temp !== null ? sh.avg_temp + '°' : '—';
            }
        } catch (err) {
            console.error('Failed to parse websocket message:', err);
        }
    }

    function updateClock() {
        const now = new Date();
        let hours = now.getHours();
        const minutes = String(now.getMinutes()).padStart(2, '0');
        const ampm = hours >= 12 ? 'PM' : 'AM';
        hours = hours % 12;
        hours = hours ? hours : 12;
        setDashboardText('live-time', `${hours}:${minutes}`);
        setDashboardText('live-ampm', ampm);

        const dateOptions = { weekday: 'long', year: 'numeric', month: 'long', day: 'numeric' };
        const rawDate = now.toLocaleDateString('en-US', dateOptions);
        const parts = rawDate.split(', ');
        if (parts.length >= 3) {
            setDashboardText('live-date', `${parts[0]} · ${parts[1]}, ${parts[2]}`);
        } else {
            setDashboardText('live-date', rawDate.replace(/,/g, ' ·'));
        }

        const rawHours = now.getHours();
        let greeting = 'Hello';
        if (rawHours >= 5 && rawHours < 12) {
            greeting = 'Good morning';
        } else if (rawHours >= 12 && rawHours < 18) {
            greeting = 'Good afternoon';
        } else if (rawHours >= 18 && rawHours < 22) {
            greeting = 'Good evening';
        } else {
            greeting = 'Working late';
        }
        const greetingTextEl = document.getElementById('greeting-text');
        if (greetingTextEl) {
            greetingTextEl.textContent = greeting;
        }
    }

    function connectWebSocket() {
        const protocol = global.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const socketUrl = `${protocol}//${global.location.host}/ws`;
        const ws = new WebSocket(socketUrl);

        ws.onopen = () => setWsStatus('live');
        ws.onmessage = onWsMessage;
        ws.onerror = () => setWsStatus('offline');
        ws.onclose = () => {
            setWsStatus('offline');
            console.warn('WebSocket closed. Attempting reload in 5s.');
            setTimeout(() => { global.location.reload(); }, 5000);
        };
    }

    function init() {
        if (config.hideTelemetry) {
            const telemetrySection = document.getElementById('telemetry-section');
            if (telemetrySection) telemetrySection.style.display = 'none';
        }

        updateClock();
        setInterval(updateClock, 1000);
        connectWebSocket();

        setTimeout(() => {
            let unknownCount = 0;
            document.querySelectorAll('.status-badge').forEach(badge => {
                if (!badge.dataset.lastStatus) {
                    styleStatusBadge(badge, 'UNKNOWN');
                    badge.title = 'No live status received yet';
                    unknownCount += 1;
                }
            });
            const hint = document.getElementById('health-status-hint');
            if (hint && isAdmin && unknownCount > 0) {
                hint.style.display = 'block';
            }
        }, 20000);
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }

    global.amudLiveDashboard = { styleStatusBadge, updateAppUrlStatuses, latestAppStatuses: () => latestAppStatuses };
})(window);
