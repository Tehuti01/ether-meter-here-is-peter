/* ═══════════════════════════════════════════════════
   AETHER-NET  ·  main.js  ·  Premium Interactive Layer
   ═══════════════════════════════════════════════════ */

// ─── Cursor glow ───
const glow = document.createElement('div');
glow.id = 'cursor-glow';
document.body.appendChild(glow);
let mx = -999, my = -999;
document.addEventListener('mousemove', e => {
  mx = e.clientX; my = e.clientY;
  glow.style.left = mx + 'px';
  glow.style.top  = my + 'px';
});

// ─── Mobile hamburger ───
const nav = document.getElementById('nav');
const hamburger = document.getElementById('hamburger');
const mobileMenu = document.getElementById('mobile-menu');
if (hamburger && mobileMenu) {
  hamburger.addEventListener('click', () => {
    const open = mobileMenu.classList.toggle('open');
    hamburger.classList.toggle('active', open);
    hamburger.setAttribute('aria-expanded', open);
  });
}

// ─── Nav scroll effect ───
let lastScroll = 0;
window.addEventListener('scroll', () => {
  const y = window.scrollY;
  nav.classList.toggle('scrolled', y > 60);
  nav.classList.toggle('nav-hidden', y > lastScroll + 80 && y > 300);
  nav.classList.toggle('nav-hidden', false); // always keep visible for now
  lastScroll = y;
  updateActiveNav();
}, { passive: true });

function updateActiveNav() {
  const sections = document.querySelectorAll('section[id]');
  const scrollY = window.scrollY + 120;
  sections.forEach(s => {
    const top = s.offsetTop, h = s.offsetHeight, id = s.getAttribute('id');
    const link = document.querySelector(`.nav-links a[href="#${id}"]`);
    if (link) link.classList.toggle('active', scrollY >= top && scrollY < top + h);
  });
}

// ─── Staggered scroll reveal ───
const revealObserver = new IntersectionObserver((entries) => {
  entries.forEach((entry, i) => {
    if (entry.isIntersecting) {
      const delay = entry.target.dataset.delay || 0;
      setTimeout(() => entry.target.classList.add('visible'), +delay);
      revealObserver.unobserve(entry.target);
    }
  });
}, { threshold: 0.1, rootMargin: '0px 0px -40px 0px' });

document.querySelectorAll('.reveal').forEach((el, i) => {
  el.dataset.delay = (i % 4) * 80;
  revealObserver.observe(el);
});

// ─── Animated counter ───
function animateCounter(el, target, suffix = '', duration = 2000) {
  const isFloat = String(target).includes('.');
  const startTime = performance.now();
  function update(now) {
    const progress = Math.min((now - startTime) / duration, 1);
    const eased = 1 - Math.pow(1 - progress, 4);
    const current = target * eased;
    el.textContent = isFloat
      ? current.toFixed(1) + suffix
      : Math.floor(current).toLocaleString() + suffix;
    if (progress < 1) requestAnimationFrame(update);
  }
  requestAnimationFrame(update);
}

const statsObserver = new IntersectionObserver((entries) => {
  entries.forEach(entry => {
    if (!entry.isIntersecting) return;
    const el = entry.target, id = el.id;
    if (id === 'stat-bandwidth') animateCounter(el, 80, '%');
    if (id === 'stat-accuracy')  animateCounter(el, 99.9, '%');
    if (id === 'stat-bodies')    animateCounter(el, 5000, '+');
    if (id === 'stat-latency')   animateCounter(el, 0, 'ms', 800);
    statsObserver.unobserve(el);
  });
}, { threshold: 0.5 });
document.querySelectorAll('.stat-value').forEach(el => statsObserver.observe(el));

// ─── Hero typed subtitle ───
const heroSub = document.getElementById('hero-typed');
if (heroSub) {
  const phrases = [
    'Decentralized physics for the browser.',
    'Neural AI predicts — locally, instantly.',
    'Every client is a compute node.',
    'φ-governed. Rust-fast. Wasm-light.',
  ];
  let pi = 0, ci = 0, deleting = false;
  function type() {
    const phrase = phrases[pi];
    if (!deleting) {
      heroSub.textContent = phrase.slice(0, ++ci);
      if (ci === phrase.length) { deleting = true; setTimeout(type, 2200); return; }
    } else {
      heroSub.textContent = phrase.slice(0, --ci);
      if (ci === 0) { deleting = false; pi = (pi + 1) % phrases.length; }
    }
    setTimeout(type, deleting ? 35 : 55);
  }
  type();
}

// ─── Flowing 3D Wave Field ───
import * as THREE from 'https://cdn.skypack.dev/three@0.136.0';

let scene, camera, renderer, waveMesh, ribbonMesh, positions, dummy, colors;

const PHI = 1.6180339887;
const GOLDEN_ANGLE = 2.39996323;
const COUNT = 800;

function initVisuals() {
  const container = document.createElement('div');
  Object.assign(container.style, {
    position: 'fixed', top: 0, left: 0, width: '100%', height: '100%',
    zIndex: 0, pointerEvents: 'none',
  });
  document.body.prepend(container);

  scene = new THREE.Scene();
  scene.fog = new THREE.FogExp2(0x05080f, 0.015);

  camera = new THREE.PerspectiveCamera(55, window.innerWidth / window.innerHeight, 0.1, 1200);
  camera.position.set(0, 18, 40);
  camera.lookAt(0, 0, 0);

  renderer = new THREE.WebGLRenderer({ alpha: true, antialias: true, powerPreference: 'high-performance' });
  renderer.setSize(window.innerWidth, window.innerHeight);
  renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
  renderer.toneMapping = THREE.ACESFilmicToneMapping;
  renderer.toneMappingExposure = 1.2;
  container.appendChild(renderer.domElement);

  // Lighting — three-point cinematic
  scene.add(new THREE.AmbientLight(0x0a0e17, 1.5));

  const keyLight = new THREE.PointLight(0x00f0d0, 3, 80);
  keyLight.position.set(-15, 25, 15);
  scene.add(keyLight);

  const fillLight = new THREE.PointLight(0xff2a85, 2, 80);
  fillLight.position.set(20, 8, -15);
  scene.add(fillLight);

  const rimLight = new THREE.PointLight(0xd4af37, 2, 60);
  rimLight.position.set(0, -15, 20);
  scene.add(rimLight);

  // Wave mesh — instanced spheres with per-instance colour
  const geo = new THREE.SphereGeometry(0.25, 8, 6);
  const mat = new THREE.MeshPhysicalMaterial({
    vertexColors: true,
    metalness: 0.6,
    roughness: 0.25,
    clearcoat: 0.8,
    clearcoatRoughness: 0.15,
    transparent: true,
    opacity: 0.85,
  });

  waveMesh = new THREE.InstancedMesh(geo, mat, COUNT);
  waveMesh.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
  dummy = new THREE.Object3D();
  positions = [];

  // Distribute in a golden-spiral disc
  for (let i = 0; i < COUNT; i++) {
    const t = i / (COUNT - 1);
    const r = Math.sqrt(t) * 28;
    const theta = GOLDEN_ANGLE * i;
    const bx = Math.cos(theta) * r;
    const bz = Math.sin(theta) * r;
    positions.push({ bx, bz, phase: i * PHI, speed: 0.5 + Math.random() * 0.5 });
  }

  // Per-instance colour attribute
  const colArr = new Float32Array(COUNT * 3);
  const teal   = new THREE.Color(0x00f0d0);
  const gold   = new THREE.Color(0xf9d75c);
  const magenta = new THREE.Color(0xff2a85);
  const palette = [teal, gold, magenta];
  for (let i = 0; i < COUNT; i++) {
    const c = palette[i % 3].clone().lerp(palette[(i + 1) % 3], (i % 13) / 13);
    colArr[i * 3]     = c.r;
    colArr[i * 3 + 1] = c.g;
    colArr[i * 3 + 2] = c.b;
  }
  geo.setAttribute('color', new THREE.InstancedBufferAttribute(colArr, 3));

  scene.add(waveMesh);

  // Ribbon lines — connecting particles with flowing curves
  const ribbonCount = 60;
  const ribbonGeo = new THREE.BufferGeometry();
  const ribbonPts = [];
  for (let r = 0; r < ribbonCount; r++) {
    const startIdx = Math.floor(Math.random() * COUNT);
    for (let s = 0; s < 8; s++) {
      const idx = (startIdx + s * 7) % COUNT;
      const p = positions[idx];
      ribbonPts.push(new THREE.Vector3(p.bx, 0, p.bz));
    }
  }
  ribbonGeo.setFromPoints(ribbonPts);
  const ribbonMat = new THREE.LineBasicMaterial({
    color: 0x00f0d0, transparent: true, opacity: 0.08, linewidth: 1,
  });
  ribbonMesh = new THREE.LineSegments(ribbonGeo, ribbonMat);
  scene.add(ribbonMesh);

  window.addEventListener('resize', () => {
    camera.aspect = window.innerWidth / window.innerHeight;
    camera.updateProjectionMatrix();
    renderer.setSize(window.innerWidth, window.innerHeight);
  });

  animate();
}

let time = 0;
function animate() {
  requestAnimationFrame(animate);
  time += 0.008;

  // Mouse-reactive camera
  const mX = mx !== -999 ? (mx / window.innerWidth) * 2 - 1 : 0;
  const mY = my !== -999 ? -(my / window.innerHeight) * 2 + 1 : 0;
  camera.position.x += (mX * 8 - camera.position.x) * 0.03;
  camera.position.y += (18 + mY * 6 - camera.position.y) * 0.03;
  camera.lookAt(0, 2, 0);

  // Global slow orbit
  const orbit = time * 0.15;

  for (let i = 0; i < COUNT; i++) {
    const p = positions[i];

    // Wave displacement: stacked sine waves for organic flow
    const wave1 = Math.sin(p.bx * 0.15 + time * p.speed * 2) * 3;
    const wave2 = Math.cos(p.bz * 0.12 + time * p.speed * 1.7 + p.phase) * 2;
    const wave3 = Math.sin((p.bx + p.bz) * 0.08 + time * 1.3) * 1.5;
    const y = wave1 + wave2 + wave3;

    // Gentle orbital rotation of the entire field
    const rx = p.bx * Math.cos(orbit) - p.bz * Math.sin(orbit);
    const rz = p.bx * Math.sin(orbit) + p.bz * Math.cos(orbit);

    // Scale pulses with the wave height
    const s = 0.6 + Math.abs(y) * 0.12;

    dummy.position.set(rx, y, rz);
    dummy.scale.set(s, s, s);
    dummy.rotation.set(time + i * 0.01, time * 0.7 + i * 0.005, 0);
    dummy.updateMatrix();
    waveMesh.setMatrixAt(i, dummy.matrix);
  }

  waveMesh.instanceMatrix.needsUpdate = true;

  // Update ribbon positions to follow wave
  const rPositions = ribbonMesh.geometry.attributes.position.array;
  for (let i = 0; i < rPositions.length / 3; i++) {
    const idx = i % COUNT;
    const p = positions[idx];
    const wave = Math.sin(p.bx * 0.15 + time * p.speed * 2) * 3
               + Math.cos(p.bz * 0.12 + time * p.speed * 1.7 + p.phase) * 2;
    const rx = p.bx * Math.cos(orbit) - p.bz * Math.sin(orbit);
    const rz = p.bx * Math.sin(orbit) + p.bz * Math.cos(orbit);
    rPositions[i * 3]     = rx;
    rPositions[i * 3 + 1] = wave;
    rPositions[i * 3 + 2] = rz;
  }
  ribbonMesh.geometry.attributes.position.needsUpdate = true;

  // Cycle ribbon colour
  const hue = (time * 0.02) % 1;
  ribbonMesh.material.color.setHSL(hue, 0.7, 0.5);
  ribbonMesh.material.opacity = 0.06 + Math.sin(time) * 0.03;

  renderer.render(scene, camera);
}

initVisuals();

// ─── Table row hover glow ───
document.querySelectorAll('.features-table tbody tr').forEach(row => {
  row.addEventListener('mouseenter', () => row.style.boxShadow = 'inset 3px 0 0 var(--teal)');
  row.addEventListener('mouseleave', () => row.style.boxShadow = 'none');
});

// ─── Smooth anchor scroll ───
document.querySelectorAll('a[href^="#"]').forEach(a => {
  a.addEventListener('click', e => {
    e.preventDefault();
    const target = document.querySelector(a.getAttribute('href'));
    if (target) target.scrollIntoView({ behavior: 'smooth', block: 'start' });
    if (mobileMenu) mobileMenu.classList.remove('open');
  });
});

// ─── Card tilt effect ───
document.querySelectorAll('.arch-card, .persp-card, .money-card').forEach(card => {
  card.addEventListener('mousemove', e => {
    const r = card.getBoundingClientRect();
    const x = (e.clientX - r.left) / r.width  - .5;
    const y = (e.clientY - r.top)  / r.height - .5;
    card.style.transform = `translateY(-6px) rotateX(${-y*8}deg) rotateY(${x*8}deg)`;
  });
  card.addEventListener('mouseleave', () => card.style.transform = '');
});

// ─── Roadmap progress fill ───
const phases = document.querySelectorAll('.roadmap-phase');
const roadmapObserver = new IntersectionObserver(entries => {
  entries.forEach(entry => {
    if (entry.isIntersecting) entry.target.classList.add('phase-active');
  });
}, { threshold: .3 });
phases.forEach(p => roadmapObserver.observe(p));

// ─── XSS / injection guard on any dynamic inputs ───
function sanitize(str) {
  const d = document.createElement('div');
  d.textContent = str;
  return d.innerHTML;
}
