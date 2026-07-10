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

    function formatBytesShort(bytes) {
        const n = Number(bytes);
        if (!Number.isFinite(n) || n <= 0) return '—';
        if (n >= 1_000_000_000_000) {
            return `${(n / 1_000_000_000_000).toFixed(1)} TB`;
        }
        if (n >= 1_000_000_000) {
            return `${(n / 1_000_000_000).toFixed(1)} GB`;
        }
        if (n >= 1_000_000) {
            return `${(n / 1_000_000).toFixed(0)} MB`;
        }
        return `${(n / 1_000).toFixed(0)} KB`;
    }

    function bitUnitMultiplier(unit) {
        const u = (unit || '').toUpperCase();
        if (u === 'G') return 1_000_000_000;
        if (u === 'M') return 1_000_000;
        return 1_000;
    }

    function byteUnitMultiplier(unit) {
        const u = (unit || 'B').toUpperCase();
        if (u === 'GB') return 1024 * 1024 * 1024;
        if (u === 'MB') return 1024 * 1024;
        if (u === 'KB') return 1024;
        return 1;
    }

    function parseRateToBps(rateText) {
        const s = String(rateText || '').trim();
        const bitRe = /^([\d.]+)\s*([kMG]?)bit\/s$/i;
        const bitMatch = bitRe.exec(s);
        if (bitMatch) {
            const v = Number.parseFloat(bitMatch[1]);
            if (!Number.isFinite(v)) return 0;
            return bitUnitMultiplier(bitMatch[2]) * v / 8;
        }
        const byteRe = /^([\d.]+)\s*([KMG]?B)\/s$/i;
        const m = byteRe.exec(s);
        if (!m) return 0;
        const v = Number.parseFloat(m[1]);
        if (!Number.isFinite(v)) return 0;
        return v * byteUnitMultiplier(m[2]);
    }

    // Input is bytes per second (agent sends bit/s strings; we convert to B/s first).
    function formatRateFromBps(bytes, allowZero) {
        if (!bytes || bytes <= 0) {
            return allowZero ? '0 B/s' : '—';
        }
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
        const diskHint = document.getElementById('val-disk-hint');
        if (diskHint) {
            const hints = [];
            if (isAdmin && sys.disk_mapping_fallback) {
                hints.push('(auto-detect)');
                diskHint.title = 'Configured disk mounts were not all visible inside the agent; showing auto-detected storage.';
            } else if (isAdmin && sys.telemetry_scope === 'container') {
                hints.push('(container scope)');
                diskHint.title = 'Agent sees container resources, not the host. Use host network and bind-mount disk paths on Unraid/Docker.';
            }
            diskHint.textContent = hints.join(' ');
            if (!hints.length) diskHint.removeAttribute('title');
        }

        updateDiskVolumes(sys);

        const gpuName = (sys.gpu_name || '').trim();
        const hasGpu = gpuName.length > 0 && (sys.gpu_usage ?? -1) >= 0;
        const gpuCard = document.getElementById('gpu-card');
        if (gpuCard) gpuCard.style.display = hasGpu ? '' : 'none';
        if (hasGpu) {
            setDashboardText('val-gpu-usage', `${sys.gpu_usage ?? 0}%`);
            setDashboardBar('bar-gpu-usage', sys.gpu_usage ?? 0);
            setDashboardText('val-gpu-name', gpuName.slice(0, 18));
            const used = (sys.gpu_mem_used_mb ?? 0) / 1024;
            const total = (sys.gpu_mem_total_mb ?? 0) / 1024;
            setDashboardText('val-gpu-vram', total > 0 ? `VRAM ${used.toFixed(1)}/${total.toFixed(1)} GB` : 'VRAM —');
        }
    }

    function updateDiskVolumes(sys) {
        const container = document.getElementById('disk-volumes-container');
        const aggregateCard = document.getElementById('disk-aggregate-card');
        if (!container) return;

        const volumes = Array.isArray(sys.disk_volumes) ? sys.disk_volumes : [];
        const showPerMount = volumes.length > 1 && !sys.disk_mapping_fallback;

        if (aggregateCard) {
            aggregateCard.style.display = showPerMount ? 'none' : '';
        }

        const keepIds = new Set();
        volumes.forEach((vol, index) => {
            if (!showPerMount) return;
            const mountKey = String(vol.mount || vol.label || index)
                .replace(/[^a-zA-Z0-9_-]/g, '_');
            const cardId = `disk-volume-${mountKey}`;
            keepIds.add(cardId);

            let card = document.getElementById(cardId);
            if (!card) {
                card = document.createElement('div');
                card.id = cardId;
                card.className = 'glass-panel telemetry-card disk-volume-card';
                card.innerHTML = `
                    <div class="telemetry-header">
                        <span class="telemetry-title"></span>
                        <span class="telemetry-value disk-volume-used">0 GB Used</span>
                    </div>
                    <div class="telemetry-bar-container">
                        <div class="telemetry-bar-fill disk-volume-bar" style="width: 0%;"></div>
                    </div>
                    <div class="telemetry-subinfo">
                        <span class="disk-volume-free">0 GB Free</span>
                        <span class="disk-volume-total">0 GB Total</span>
                    </div>`;
                container.appendChild(card);
            }

            const label = vol.label || vol.mount || 'Disk';
            const titleEl = card.querySelector('.telemetry-title');
            if (titleEl) titleEl.textContent = label;

            const usedGb = vol.used_gb ?? 0;
            const totalGb = vol.total_gb ?? 0;
            const freeGb = totalGb - usedGb;
            const usage = vol.usage ?? 0;

            const usedEl = card.querySelector('.disk-volume-used');
            if (usedEl) usedEl.textContent = `${usedGb.toFixed(1)} GB Used`;
            const freeEl = card.querySelector('.disk-volume-free');
            if (freeEl) freeEl.textContent = `${freeGb.toFixed(1)} GB Free`;
            const totalEl = card.querySelector('.disk-volume-total');
            if (totalEl) totalEl.textContent = `${totalGb.toFixed(1)} GB Total`;
            const barEl = card.querySelector('.disk-volume-bar');
            if (barEl) barEl.style.width = `${Math.min(100, Math.max(0, usage))}%`;
        });

        container.querySelectorAll('.disk-volume-card').forEach(card => {
            if (!keepIds.has(card.id)) card.remove();
        });
        container.style.display = showPerMount ? '' : 'none';
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

                document.querySelectorAll('.app-card').forEach(card => {
                    const nodeTag = card.getAttribute('data-node-tag') || 'Local';
                    const nodeTel = (data.nodes && data.nodes[nodeTag]) ? data.nodes[nodeTag] : sys;
                    const containers = (nodeTel.lxc_containers && nodeTel.lxc_containers.length > 0)
                        ? nodeTel.lxc_containers
                        : ((sys.lxc_containers && sys.lxc_containers.length > 0) ? sys.lxc_containers : []);
                    if (containers.length > 0 && nodeTag === (sys.node_tag || 'Local')) {
                        updateStreamStatusBadges(containers);
                    }
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

                        const cpuPct = match.cpu ? `${(match.cpu * 100).toFixed(1)}%` : '0%';
                        const ramDisplay = match.mem != null && match.mem > 0
                            ? formatBytesShort(match.mem)
                            : '—';
                        updateCardContainerMetrics(card, cpuPct, ramDisplay, match.cpu > 0.8);
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
                                let badgeStatus;
                                if (isAdmin) {
                                    badgeStatus = data.agent_connected ? 'running' : 'offline';
                                } else {
                                    badgeStatus = guestAvailabilityStatus(
                                        data.agent_connected ? 'running' : 'offline'
                                    );
                                }
                                styleStatusBadge(existingBadge, badgeStatus);
                            }
                        }
                        updateCardContainerMetrics(
                            card,
                            `${sys.cpu_usage ?? 0}%`,
                            `${(sys.ram_used_gb ?? 0).toFixed(1)} GB`,
                            (sys.cpu_usage ?? 0) > 80
                        );
                    }
                });
            }

            if (data.app_statuses) {
                updateAppUrlStatuses(data.app_statuses);
                try {
                    const slim = Object.fromEntries(
                        Object.entries(data.app_statuses).map(([k, v]) => [k, { s: v.status }])
                    );
                    localStorage.setItem('amud-offline-status', JSON.stringify(slim));
                } catch (_) { /* quota */ }
            }

            if (data.network) {
                const net = data.network;
                const tx = parseRateToBps(net.internal_tx) + parseRateToBps(net.external_tx);
                const rx = parseRateToBps(net.internal_rx) + parseRateToBps(net.external_rx);
                const total = tx + rx;
                setDashboardText('val-net-up', `↑ ${formatRateFromBps(tx, true)}`);
                setDashboardText('val-net-down', `↓ ${formatRateFromBps(rx, true)}`);
                setDashboardText('val-net-total', formatRateFromBps(total, true));
                setDashboardBar('bar-net-total', Math.min(100, total / (125 * 1024 * 1024) * 100));
                const netHint = document.getElementById('val-net-hint');
                if (netHint) {
                    const hints = [];
                    if (isAdmin && data.system && data.system.network_mapping_fallback) {
                        hints.push('(auto-detect)');
                        netHint.title = 'Configured network interfaces were not all visible inside the agent; showing auto-detected bandwidth.';
                    } else if (isAdmin && data.system && data.system.telemetry_scope === 'container') {
                        hints.push('(container scope)');
                        netHint.title = 'Agent sees container NICs, not the host. Use host network on Unraid/Docker for br0/bond0 mapping.';
                    }
                    netHint.textContent = hints.join(' ');
                    if (!hints.length) netHint.removeAttribute('title');
                }
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

    function updateCardContainerMetrics(card, cpuDisplay, ramDisplay, cpuHot) {
        const cpuEl = card.querySelector('[data-lxc-cpu] .metric-value');
        const ramEl = card.querySelector('[data-lxc-ram] .metric-value');
        if (cpuEl && ramEl) {
            cpuEl.textContent = cpuDisplay;
            ramEl.textContent = ramDisplay;
            if (cpuHot) cpuEl.style.color = '#ef4444';
            else cpuEl.style.removeProperty('color');
            return;
        }
        const metricsGrid = card.querySelector('[data-lxc-metrics]');
        if (typeof global.shouldUpdateLxcMetrics === 'function' && global.shouldUpdateLxcMetrics(metricsGrid)) {
            global.setMetricsGrid(metricsGrid, [
                { value: cpuDisplay, label: 'CPU', valueStyle: cpuHot ? 'color: #ef4444' : '' },
                { value: ramDisplay, label: 'RAM' },
            ]);
        }
    }

    function refreshIntegrationCardsBatch(cards) {
        const ids = cards
            .map(card => card.getAttribute('data-integration-refresh'))
            .filter(Boolean);
        if (!ids.length) return;
        fetch('/api/apps/integrations/batch', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ ids: ids.map(id => parseInt(id, 10)) }),
        })
            .then(r => {
                if (r.status === 503) return null;
                return r.ok ? r.json() : null;
            })
            .then(batch => {
                if (!batch) return;
                cards.forEach(card => {
                    const appId = card.getAttribute('data-integration-refresh');
                    const d = batch[appId];
                    if (!d || !d.type) return;
                    if (typeof Alpine !== 'undefined' && typeof Alpine.$data === 'function') {
                        const data = Alpine.$data(card);
                        if (data) data.integrationData = d;
                    }
                });
            })
            .catch(() => {});
    }

    function reportViewport(ids) {
        if (!ids.length) return;
        fetch('/api/activity/viewport', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ ids }),
        }).catch(() => {});
    }

    function reportPresence(active) {
        fetch('/api/activity/presence', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ active }),
        }).catch(() => {});
    }

    function initIntegrationRefresh() {
        const cards = Array.from(document.querySelectorAll('[data-integration-refresh]'));
        if (!cards.length) return;

        const visible = new Set();
        const observer = new IntersectionObserver((entries) => {
            entries.forEach(entry => {
                const id = entry.target.getAttribute('data-integration-refresh');
                if (!id) return;
                if (entry.isIntersecting) visible.add(id);
                else visible.delete(id);
            });
            const visibleIds = Array.from(visible).map(id => parseInt(id, 10)).slice(0, 50);
            reportViewport(visibleIds);
        }, { root: null, threshold: 0.1 });

        cards.forEach(card => observer.observe(card));

        let batchTimer = null;
        const tick = () => {
            const visibleCards = cards.filter(card => {
                const id = card.getAttribute('data-integration-refresh');
                return id && visible.has(id);
            });
            if (!visibleCards.length) return;
            if (batchTimer) clearTimeout(batchTimer);
            batchTimer = setTimeout(() => refreshIntegrationCardsBatch(visibleCards), 300);
        };
        setInterval(tick, 30000);
        tick();
    }

    function initActivityPresence() {
        reportPresence(true);
        document.addEventListener('visibilitychange', () => {
            reportPresence(!document.hidden);
        });
        window.addEventListener('beforeunload', () => reportPresence(false));
    }

    function init() {
        if (config.hideTelemetry) {
            const telemetrySection = document.getElementById('telemetry-section');
            if (telemetrySection) telemetrySection.style.display = 'none';
        }

        updateClock();
        setInterval(updateClock, 1000);
        connectWebSocket();
        initActivityPresence();
        initIntegrationRefresh();

        // Make app cards clickable across their whole surface, while preserving
        // drag handles, control buttons, and other interactive elements.
        document.addEventListener('click', function (event) {
            const card = event.target.closest('.app-card');
            if (!card) return;

            // Ignore clicks that originate from interactive controls or links.
            const interactive = event.target.closest('button, form, input, select, textarea, label, a');
            if (interactive) return;

            const link = card.querySelector('.app-card-open');
            if (!link) return;

            // Delegate to the existing anchor so embed-tabs.js and normal
            // navigation behavior continue to work as-is.
            link.click();
        });

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
