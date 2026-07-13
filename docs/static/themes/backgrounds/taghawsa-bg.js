/**
 * AMUD Taghawsa theme — WebGL animated background (CodePen Scheme 5).
 * Requires /static/vendor/three.min.js loaded first.
 */
(function (global) {
    'use strict';

    var THEME_ID = 'taghawsa';
    var CONTAINER_ID = 'amud-webgl-bg';
    var MOBILE_MAX = 768;

    var instance = null;

    function prefersReducedMotion() {
        try {
            return global.matchMedia('(prefers-reduced-motion: reduce)').matches;
        } catch (e) {
            return false;
        }
    }

    function isMobileViewport() {
        return global.innerWidth <= MOBILE_MAX;
    }

    function shouldSkipWebGl() {
        if (prefersReducedMotion() || isMobileViewport()) return true;
        if (document.body.classList.contains('settings-page')) return true;
        if (document.documentElement.getAttribute('data-theme') === 'light') return true;
        return false;
    }

    function TouchTexture() {
        this.size = 64;
        this.width = this.height = this.size;
        this.maxAge = 64;
        this.radius = 0.25 * this.size;
        this.speed = 1 / this.maxAge;
        this.trail = [];
        this.last = null;
        this.canvas = document.createElement('canvas');
        this.canvas.width = this.width;
        this.canvas.height = this.height;
        this.ctx = this.canvas.getContext('2d');
        this.ctx.fillStyle = 'black';
        this.ctx.fillRect(0, 0, this.canvas.width, this.canvas.height);
        this.texture = new THREE.Texture(this.canvas);
    }

    TouchTexture.prototype.update = function () {
        this.ctx.fillStyle = 'black';
        this.ctx.fillRect(0, 0, this.canvas.width, this.canvas.height);
        var speed = this.speed;
        for (var i = this.trail.length - 1; i >= 0; i--) {
            var point = this.trail[i];
            var f = point.force * speed * (1 - point.age / this.maxAge);
            point.x += point.vx * f;
            point.y += point.vy * f;
            point.age++;
            if (point.age > this.maxAge) {
                this.trail.splice(i, 1);
            } else {
                this.drawPoint(point);
            }
        }
        this.texture.needsUpdate = true;
    };

    TouchTexture.prototype.addTouch = function (point) {
        var force = 0;
        var vx = 0;
        var vy = 0;
        var last = this.last;
        if (last) {
            var dx = point.x - last.x;
            var dy = point.y - last.y;
            if (dx === 0 && dy === 0) return;
            var dd = dx * dx + dy * dy;
            var d = Math.sqrt(dd);
            vx = dx / d;
            vy = dy / d;
            force = Math.min(dd * 20000, 2.0);
        }
        this.last = { x: point.x, y: point.y };
        this.trail.push({ x: point.x, y: point.y, age: 0, force: force, vx: vx, vy: vy });
    };

    TouchTexture.prototype.drawPoint = function (point) {
        var pos = {
            x: point.x * this.width,
            y: (1 - point.y) * this.height
        };
        var intensity = 1;
        if (point.age < this.maxAge * 0.3) {
            intensity = Math.sin((point.age / (this.maxAge * 0.3)) * (Math.PI / 2));
        } else {
            var t = 1 - (point.age - this.maxAge * 0.3) / (this.maxAge * 0.7);
            intensity = -t * (t - 2);
        }
        intensity *= point.force;
        var radius = this.radius;
        var color = ((point.vx + 1) / 2) * 255 + ', ' +
            ((point.vy + 1) / 2) * 255 + ', ' +
            (intensity * 255);
        var offset = this.size * 5;
        this.ctx.shadowOffsetX = offset;
        this.ctx.shadowOffsetY = offset;
        this.ctx.shadowBlur = radius;
        this.ctx.shadowColor = 'rgba(' + color + ',' + (0.2 * intensity) + ')';
        this.ctx.beginPath();
        this.ctx.fillStyle = 'rgba(255,0,0,1)';
        this.ctx.arc(pos.x - offset, pos.y - offset, radius, 0, Math.PI * 2);
        this.ctx.fill();
    };

    var FRAGMENT_SHADER = [
        'uniform float uTime;',
        'uniform vec2 uResolution;',
        'uniform vec3 uColor1;',
        'uniform vec3 uColor2;',
        'uniform vec3 uColor3;',
        'uniform vec3 uColor4;',
        'uniform vec3 uColor5;',
        'uniform vec3 uColor6;',
        'uniform float uSpeed;',
        'uniform float uIntensity;',
        'uniform sampler2D uTouchTexture;',
        'uniform float uGrainIntensity;',
        'uniform vec3 uDarkNavy;',
        'uniform float uGradientSize;',
        'uniform float uGradientCount;',
        'uniform float uColor1Weight;',
        'uniform float uColor2Weight;',
        'varying vec2 vUv;',
        'float grain(vec2 uv, float time) {',
        '  vec2 grainUv = uv * uResolution * 0.5;',
        '  float grainValue = fract(sin(dot(grainUv + time, vec2(12.9898, 78.233))) * 43758.5453);',
        '  return grainValue * 2.0 - 1.0;',
        '}',
        'vec3 getGradientColor(vec2 uv, float time) {',
        '  float gradientRadius = uGradientSize;',
        '  vec2 center1 = vec2(0.5 + sin(time * uSpeed * 0.4) * 0.4, 0.5 + cos(time * uSpeed * 0.5) * 0.4);',
        '  vec2 center2 = vec2(0.5 + cos(time * uSpeed * 0.6) * 0.5, 0.5 + sin(time * uSpeed * 0.45) * 0.5);',
        '  vec2 center3 = vec2(0.5 + sin(time * uSpeed * 0.35) * 0.45, 0.5 + cos(time * uSpeed * 0.55) * 0.45);',
        '  vec2 center4 = vec2(0.5 + cos(time * uSpeed * 0.5) * 0.4, 0.5 + sin(time * uSpeed * 0.4) * 0.4);',
        '  vec2 center5 = vec2(0.5 + sin(time * uSpeed * 0.7) * 0.35, 0.5 + cos(time * uSpeed * 0.6) * 0.35);',
        '  vec2 center6 = vec2(0.5 + cos(time * uSpeed * 0.45) * 0.5, 0.5 + sin(time * uSpeed * 0.65) * 0.5);',
        '  vec2 center7 = vec2(0.5 + sin(time * uSpeed * 0.55) * 0.38, 0.5 + cos(time * uSpeed * 0.48) * 0.42);',
        '  vec2 center8 = vec2(0.5 + cos(time * uSpeed * 0.65) * 0.36, 0.5 + sin(time * uSpeed * 0.52) * 0.44);',
        '  vec2 center9 = vec2(0.5 + sin(time * uSpeed * 0.42) * 0.41, 0.5 + cos(time * uSpeed * 0.58) * 0.39);',
        '  vec2 center10 = vec2(0.5 + cos(time * uSpeed * 0.48) * 0.37, 0.5 + sin(time * uSpeed * 0.62) * 0.43);',
        '  vec2 center11 = vec2(0.5 + sin(time * uSpeed * 0.68) * 0.33, 0.5 + cos(time * uSpeed * 0.44) * 0.46);',
        '  vec2 center12 = vec2(0.5 + cos(time * uSpeed * 0.38) * 0.39, 0.5 + sin(time * uSpeed * 0.56) * 0.41);',
        '  float dist1 = length(uv - center1);',
        '  float dist2 = length(uv - center2);',
        '  float dist3 = length(uv - center3);',
        '  float dist4 = length(uv - center4);',
        '  float dist5 = length(uv - center5);',
        '  float dist6 = length(uv - center6);',
        '  float dist7 = length(uv - center7);',
        '  float dist8 = length(uv - center8);',
        '  float dist9 = length(uv - center9);',
        '  float dist10 = length(uv - center10);',
        '  float dist11 = length(uv - center11);',
        '  float dist12 = length(uv - center12);',
        '  float influence1 = 1.0 - smoothstep(0.0, gradientRadius, dist1);',
        '  float influence2 = 1.0 - smoothstep(0.0, gradientRadius, dist2);',
        '  float influence3 = 1.0 - smoothstep(0.0, gradientRadius, dist3);',
        '  float influence4 = 1.0 - smoothstep(0.0, gradientRadius, dist4);',
        '  float influence5 = 1.0 - smoothstep(0.0, gradientRadius, dist5);',
        '  float influence6 = 1.0 - smoothstep(0.0, gradientRadius, dist6);',
        '  float influence7 = 1.0 - smoothstep(0.0, gradientRadius, dist7);',
        '  float influence8 = 1.0 - smoothstep(0.0, gradientRadius, dist8);',
        '  float influence9 = 1.0 - smoothstep(0.0, gradientRadius, dist9);',
        '  float influence10 = 1.0 - smoothstep(0.0, gradientRadius, dist10);',
        '  float influence11 = 1.0 - smoothstep(0.0, gradientRadius, dist11);',
        '  float influence12 = 1.0 - smoothstep(0.0, gradientRadius, dist12);',
        '  vec2 rotatedUv1 = uv - 0.5;',
        '  float angle1 = time * uSpeed * 0.15;',
        '  rotatedUv1 = vec2(rotatedUv1.x * cos(angle1) - rotatedUv1.y * sin(angle1), rotatedUv1.x * sin(angle1) + rotatedUv1.y * cos(angle1));',
        '  rotatedUv1 += 0.5;',
        '  vec2 rotatedUv2 = uv - 0.5;',
        '  float angle2 = -time * uSpeed * 0.12;',
        '  rotatedUv2 = vec2(rotatedUv2.x * cos(angle2) - rotatedUv2.y * sin(angle2), rotatedUv2.x * sin(angle2) + rotatedUv2.y * cos(angle2));',
        '  rotatedUv2 += 0.5;',
        '  float radialInfluence1 = 1.0 - smoothstep(0.0, 0.8, length(rotatedUv1 - 0.5));',
        '  float radialInfluence2 = 1.0 - smoothstep(0.0, 0.8, length(rotatedUv2 - 0.5));',
        '  vec3 color = vec3(0.0);',
        '  color += uColor1 * influence1 * (0.55 + 0.45 * sin(time * uSpeed)) * uColor1Weight;',
        '  color += uColor2 * influence2 * (0.55 + 0.45 * cos(time * uSpeed * 1.2)) * uColor2Weight;',
        '  color += uColor3 * influence3 * (0.55 + 0.45 * sin(time * uSpeed * 0.8)) * uColor1Weight;',
        '  color += uColor4 * influence4 * (0.55 + 0.45 * cos(time * uSpeed * 1.3)) * uColor2Weight;',
        '  color += uColor5 * influence5 * (0.55 + 0.45 * sin(time * uSpeed * 1.1)) * uColor1Weight;',
        '  color += uColor6 * influence6 * (0.55 + 0.45 * cos(time * uSpeed * 0.9)) * uColor2Weight;',
        '  if (uGradientCount > 6.0) {',
        '    color += uColor1 * influence7 * (0.55 + 0.45 * sin(time * uSpeed * 1.4)) * uColor1Weight;',
        '    color += uColor2 * influence8 * (0.55 + 0.45 * cos(time * uSpeed * 1.5)) * uColor2Weight;',
        '    color += uColor3 * influence9 * (0.55 + 0.45 * sin(time * uSpeed * 1.6)) * uColor1Weight;',
        '    color += uColor4 * influence10 * (0.55 + 0.45 * cos(time * uSpeed * 1.7)) * uColor2Weight;',
        '  }',
        '  if (uGradientCount > 10.0) {',
        '    color += uColor5 * influence11 * (0.55 + 0.45 * sin(time * uSpeed * 1.8)) * uColor1Weight;',
        '    color += uColor6 * influence12 * (0.55 + 0.45 * cos(time * uSpeed * 1.9)) * uColor2Weight;',
        '  }',
        '  color += mix(uColor1, uColor3, radialInfluence1) * 0.45 * uColor1Weight;',
        '  color += mix(uColor2, uColor4, radialInfluence2) * 0.4 * uColor2Weight;',
        '  color = clamp(color, vec3(0.0), vec3(1.0)) * uIntensity;',
        '  float luminance = dot(color, vec3(0.299, 0.587, 0.114));',
        '  color = mix(vec3(luminance), color, 1.35);',
        '  color = pow(color, vec3(0.92));',
        '  float brightness1 = length(color);',
        '  color = mix(uDarkNavy, color, max(brightness1 * 1.2, 0.15));',
        '  float maxBrightness = 1.0;',
        '  float brightness = length(color);',
        '  if (brightness > maxBrightness) color = color * (maxBrightness / brightness);',
        '  return color;',
        '}',
        'void main() {',
        '  vec2 uv = vUv;',
        '  vec4 touchTex = texture2D(uTouchTexture, uv);',
        '  float vx = -(touchTex.r * 2.0 - 1.0);',
        '  float vy = -(touchTex.g * 2.0 - 1.0);',
        '  float intensity = touchTex.b;',
        '  uv.x += vx * 0.8 * intensity;',
        '  uv.y += vy * 0.8 * intensity;',
        '  vec2 center = vec2(0.5);',
        '  float dist = length(uv - center);',
        '  float ripple = sin(dist * 20.0 - uTime * 3.0) * 0.04 * intensity;',
        '  float wave = sin(dist * 15.0 - uTime * 2.0) * 0.03 * intensity;',
        '  uv += vec2(ripple + wave);',
        '  vec3 color = getGradientColor(uv, uTime);',
        '  color += grain(uv, uTime) * uGrainIntensity;',
        '  float timeShift = uTime * 0.5;',
        '  color.r += sin(timeShift) * 0.02;',
        '  color.g += cos(timeShift * 1.4) * 0.02;',
        '  color.b += sin(timeShift * 1.2) * 0.02;',
        '  float brightness2 = length(color);',
        '  color = mix(uDarkNavy, color, max(brightness2 * 1.2, 0.15));',
        '  color = clamp(color, vec3(0.0), vec3(1.0));',
        '  float brightness = length(color);',
        '  if (brightness > 1.0) color = color * (1.0 / brightness);',
        '  gl_FragColor = vec4(color, 1.0);',
        '}'
    ].join('\n');

    function TaghawsaApp(container) {
        this.container = container;
        this.running = false;
        this.rafId = null;
        this.paused = false;

        this.renderer = new THREE.WebGLRenderer({
            antialias: true,
            powerPreference: 'high-performance',
            alpha: false,
            stencil: false,
            depth: false
        });
        this.renderer.setPixelRatio(Math.min(global.devicePixelRatio || 1, 2));
        this.renderer.domElement.style.display = 'block';
        this.renderer.domElement.style.width = '100%';
        this.renderer.domElement.style.height = '100%';
        container.appendChild(this.renderer.domElement);

        this.camera = new THREE.PerspectiveCamera(45, 1, 0.1, 10000);
        this.camera.position.z = 50;
        this.scene = new THREE.Scene();
        this.scene.background = new THREE.Color(0x0a0e27);
        this.clock = new THREE.Clock();

        this.touchTexture = new TouchTexture();
        this.uniforms = {
            uTime: { value: 0 },
            uResolution: { value: new THREE.Vector2(global.innerWidth, global.innerHeight) },
            uColor1: { value: new THREE.Vector3(0.945, 0.353, 0.133) },
            uColor2: { value: new THREE.Vector3(0.0, 0.259, 0.22) },
            uColor3: { value: new THREE.Vector3(0.945, 0.353, 0.133) },
            uColor4: { value: new THREE.Vector3(0.0, 0.0, 0.0) },
            uColor5: { value: new THREE.Vector3(0.945, 0.353, 0.133) },
            uColor6: { value: new THREE.Vector3(0.0, 0.0, 0.0) },
            uSpeed: { value: 1.5 },
            uIntensity: { value: 1.8 },
            uTouchTexture: { value: this.touchTexture.texture },
            uGrainIntensity: { value: 0.08 },
            uDarkNavy: { value: new THREE.Vector3(0.039, 0.055, 0.153) },
            uGradientSize: { value: 0.45 },
            uGradientCount: { value: 12.0 },
            uColor1Weight: { value: 0.5 },
            uColor2Weight: { value: 1.8 }
        };

        this.mesh = null;
        this.onMouseMove = this.onMouseMove.bind(this);
        this.onTouchMove = this.onTouchMove.bind(this);
        this.onResize = this.onResize.bind(this);
        this.onVisibility = this.onVisibility.bind(this);
        this._tick = this.tick.bind(this);

        this.initMesh();
        this.onResize();
    }

    TaghawsaApp.prototype.getViewSize = function () {
        var fovInRadians = (this.camera.fov * Math.PI) / 180;
        var height = Math.abs(this.camera.position.z * Math.tan(fovInRadians / 2) * 2);
        return { width: height * this.camera.aspect, height: height };
    };

    TaghawsaApp.prototype.initMesh = function () {
        var viewSize = this.getViewSize();
        var geometry = new THREE.PlaneGeometry(viewSize.width, viewSize.height, 1, 1);
        var material = new THREE.ShaderMaterial({
            uniforms: this.uniforms,
            vertexShader: [
                'varying vec2 vUv;',
                'void main() {',
                '  gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);',
                '  vUv = uv;',
                '}'
            ].join('\n'),
            fragmentShader: FRAGMENT_SHADER
        });
        this.mesh = new THREE.Mesh(geometry, material);
        this.mesh.position.z = 0;
        this.scene.add(this.mesh);
    };

    TaghawsaApp.prototype.onMouseMove = function (ev) {
        this.touchTexture.addTouch({
            x: ev.clientX / global.innerWidth,
            y: 1 - ev.clientY / global.innerHeight
        });
    };

    TaghawsaApp.prototype.onTouchMove = function (ev) {
        if (!ev.touches || !ev.touches.length) return;
        var touch = ev.touches[0];
        this.onMouseMove({ clientX: touch.clientX, clientY: touch.clientY });
    };

    TaghawsaApp.prototype.onResize = function () {
        var w = global.innerWidth;
        var h = global.innerHeight;
        this.camera.aspect = w / h;
        this.camera.updateProjectionMatrix();
        this.renderer.setSize(w, h);
        if (this.mesh) {
            var viewSize = this.getViewSize();
            this.mesh.geometry.dispose();
            this.mesh.geometry = new THREE.PlaneGeometry(viewSize.width, viewSize.height, 1, 1);
        }
        if (this.uniforms.uResolution) {
            this.uniforms.uResolution.value.set(w, h);
        }
    };

    TaghawsaApp.prototype.onVisibility = function () {
        this.paused = document.hidden;
        if (!this.paused && this.running) {
            this.clock.getDelta();
            this.tick();
        }
    };

    TaghawsaApp.prototype.tick = function () {
        if (!this.running) return;
        this.rafId = global.requestAnimationFrame(this._tick);
        if (this.paused) return;
        var delta = Math.min(this.clock.getDelta(), 0.1);
        this.uniforms.uTime.value += delta;
        this.touchTexture.update();
        this.renderer.render(this.scene, this.camera);
    };

    TaghawsaApp.prototype.start = function () {
        this.running = true;
        this.paused = false;
        this.clock.getDelta();
        global.addEventListener('mousemove', this.onMouseMove);
        global.addEventListener('touchmove', this.onTouchMove, { passive: true });
        global.addEventListener('resize', this.onResize);
        document.addEventListener('visibilitychange', this.onVisibility);
        this.tick();
    };

    TaghawsaApp.prototype.stop = function () {
        this.running = false;
        if (this.rafId !== null) {
            global.cancelAnimationFrame(this.rafId);
            this.rafId = null;
        }
        global.removeEventListener('mousemove', this.onMouseMove);
        global.removeEventListener('touchmove', this.onTouchMove);
        global.removeEventListener('resize', this.onResize);
        document.removeEventListener('visibilitychange', this.onVisibility);
    };

    TaghawsaApp.prototype.dispose = function () {
        this.stop();
        if (this.mesh) {
            if (this.mesh.geometry) this.mesh.geometry.dispose();
            if (this.mesh.material) this.mesh.material.dispose();
            this.scene.remove(this.mesh);
            this.mesh = null;
        }
        if (this.touchTexture && this.touchTexture.texture) {
            this.touchTexture.texture.dispose();
        }
        if (this.renderer) {
            this.renderer.dispose();
            if (this.renderer.domElement && this.renderer.domElement.parentNode) {
                this.renderer.domElement.parentNode.removeChild(this.renderer.domElement);
            }
        }
    };

    function init() {
        if (instance) return;
        if (typeof THREE === 'undefined') return;
        if (shouldSkipWebGl()) return;

        var container = document.getElementById(CONTAINER_ID);
        if (!container) {
            container = document.createElement('div');
            container.id = CONTAINER_ID;
            container.setAttribute('aria-hidden', 'true');
            container.style.cssText = 'position:fixed;inset:0;z-index:0;pointer-events:none;overflow:hidden;';
            document.body.insertBefore(container, document.body.firstChild);
        }

        document.body.classList.add('has-webgl-bg');
        instance = new TaghawsaApp(container);
        instance.start();
    }

    function destroy() {
        document.body.classList.remove('has-webgl-bg');
        if (instance) {
            instance.dispose();
            instance = null;
        }
        var container = document.getElementById(CONTAINER_ID);
        if (container) container.remove();
    }

    global.amudThemeBackground = {
        id: THEME_ID,
        init: init,
        destroy: destroy
    };
})(typeof window !== 'undefined' ? window : globalThis);
