#!/usr/bin/env node
/** Minimal tests for per-mount disk card visibility rules */

function assert(cond, msg) {
    if (!cond) {
        console.error('FAIL:', msg);
        process.exit(1);
    }
}

function shouldShowPerMount(sys) {
    const volumes = Array.isArray(sys.disk_volumes) ? sys.disk_volumes : [];
    return volumes.length > 1 && !sys.disk_mapping_fallback;
}

assert(
    shouldShowPerMount({
        disk_mapping_fallback: false,
        disk_volumes: [{ mount: '/mnt/user' }, { mount: '/mnt/cache' }],
    }),
    'two mounts without fallback'
);

assert(
    !shouldShowPerMount({
        disk_mapping_fallback: true,
        disk_volumes: [{ mount: '/mnt/user' }, { mount: '/mnt/cache' }],
    }),
    'fallback uses aggregate only'
);

assert(
    !shouldShowPerMount({
        disk_mapping_fallback: false,
        disk_volumes: [{ mount: '/mnt/user' }],
    }),
    'single mount uses aggregate'
);

console.log('disk volume render tests: ok');
