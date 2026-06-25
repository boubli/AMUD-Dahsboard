#!/usr/bin/env node
/** Lightweight parser tests mirroring ui/static/dashboard-live.js */

function parseRateToBps(rateText) {
    const s = String(rateText || '').trim();
    const bitMatch = s.match(/^([\d.]+)\s*([kMG]?)bit\/s$/i);
    if (bitMatch) {
        const v = parseFloat(bitMatch[1]);
        if (!Number.isFinite(v)) return 0;
        const unit = (bitMatch[2] || '').toUpperCase();
        const bitsPerSec = unit === 'G' ? v * 1_000_000_000 : unit === 'M' ? v * 1_000_000 : v * 1_000;
        return bitsPerSec / 8;
    }
    const m = s.match(/^([\d.]+)\s*([KMG]?B)\/s$/i);
    if (!m) return 0;
    const v = parseFloat(m[1]);
    if (!Number.isFinite(v)) return 0;
    const unit = (m[2] || 'B').toUpperCase();
    const mult = unit === 'GB' ? 1024 * 1024 * 1024 : unit === 'MB' ? 1024 * 1024 : unit === 'KB' ? 1024 : 1;
    return v * mult;
}

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

function assert(cond, msg) {
    if (!cond) {
        console.error('FAIL:', msg);
        process.exit(1);
    }
}

assert(Math.abs(parseRateToBps('512 kbit/s') - 64_000) < 0.01, '512 kbit/s');
assert(Math.abs(parseRateToBps('1.2 Mbit/s') - 150_000) < 0.01, '1.2 Mbit/s');
assert(parseRateToBps('100 KB/s') === 100 * 1024, '100 KB/s');
assert(Math.abs(parseRateToBps('2 Gbit/s') - 250_000_000) < 1, '2 Gbit/s');
assert(parseRateToBps('not-a-rate') === 0, 'garbage input');
assert(formatRateFromBps(0, true) === '0 B/s', 'zero with allowZero');
assert(formatRateFromBps(0, false) === '—', 'zero without allowZero');

console.log('dashboard rate parser tests: ok');
