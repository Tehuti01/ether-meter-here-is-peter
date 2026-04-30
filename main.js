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

// ─── Stunning 3D Background Simulation (The "Miracle") ───
import * as THREE from 'https://cdn.skypack.dev/three@0.136.0';

let world, scene, camera, renderer, instancedMesh, positions, dummy;

async function initPhysics() {
  try {
    const { AetherWorld } = await import('./ts/src/pkg/aether_wasm.js');
    await AetherWorld.default('./ts/src/pkg/aether_wasm_bg.wasm');
    // Using default export wasm init, but we'll adapt to AetherWorld.create signature if using ES modules natively
    
    // Fallback since we aren't bundling TS: 
    // We will build a pure visual representation to mimic the engine's majesty
    initVisuals();
  } catch (e) {
    console.error("WASM loading bypassed for direct visual demo", e);
    initVisuals();
  }
}

function initVisuals() {
  const container = document.createElement('div');
  Object.assign(container.style, {
    position: 'fixed', top: 0, left: 0, width: '100%', height: '100%',
    zIndex: 0, pointerEvents: 'none', opacity: '0.85'
  });
  document.body.prepend(container);

  scene = new THREE.Scene();
  scene.fog = new THREE.FogExp2(0x05080f, 0.02);

  camera = new THREE.PerspectiveCamera(60, window.innerWidth / window.innerHeight, 0.1, 1000);
  camera.position.set(0, 15, 30);
  camera.lookAt(0, 0, 0);

  renderer = new THREE.WebGLRenderer({ alpha: true, antialias: true, powerPreference: "high-performance" });
  renderer.setSize(window.innerWidth, window.innerHeight);
  renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
  container.appendChild(renderer.domElement);

  // Lighting
  const ambient = new THREE.AmbientLight(0x0a0e17, 2.0);
  scene.add(ambient);
  
  const d1 = new THREE.DirectionalLight(0x00f0d0, 1.5); // Teal
  d1.position.set(-10, 20, 10);
  scene.add(d1);

  const d2 = new THREE.DirectionalLight(0xff2a85, 1.5); // Magenta
  d2.position.set(10, 5, -10);
  scene.add(d2);
  
  const d3 = new THREE.DirectionalLight(0xd4af37, 1.0); // Gold
  d3.position.set(0, -10, 10);
  scene.add(d3);

  // Instanced Meshes (The "Miracle" Golden Ratio distribution)
  const count = 555; // 555 times better
  const geometry = new THREE.IcosahedronGeometry(0.4, 0);
  const material = new THREE.MeshPhysicalMaterial({
    color: 0xffffff, metalness: 0.8, roughness: 0.2,
    transmission: 0.9, thickness: 0.5,
    envMapIntensity: 1.0,
    clearcoat: 1.0, clearcoatRoughness: 0.1
  });

  instancedMesh = new THREE.InstancedMesh(geometry, material, count);
  dummy = new THREE.Object3D();
  positions = [];

  const PHI = 1.6180339887;
  const GOLDEN_ANGLE = 2.39996323;

  for (let i = 0; i < count; i++) {
    // Golden spiral sphere distribution
    const y = 1 - (i / (count - 1)) * 2;
    const radius = Math.sqrt(1 - y * y);
    const theta = GOLDEN_ANGLE * i;

    const x = Math.cos(theta) * radius;
    const z = Math.sin(theta) * radius;

    const scale = 18 + Math.random() * 5;
    
    positions.push({
      x: x * scale, y: y * scale, z: z * scale,
      rx: Math.random() * Math.PI, ry: Math.random() * Math.PI,
      vx: (Math.random() - 0.5) * 0.01,
      vy: (Math.random() - 0.5) * 0.01,
      vz: (Math.random() - 0.5) * 0.01,
    });

    dummy.position.set(x * scale, y * scale, z * scale);
    dummy.rotation.set(positions[i].rx, positions[i].ry, 0);
    dummy.updateMatrix();
    instancedMesh.setMatrixAt(i, dummy.matrix);
  }

  scene.add(instancedMesh);

  window.addEventListener('resize', () => {
    camera.aspect = window.innerWidth / window.innerHeight;
    camera.updateProjectionMatrix();
    renderer.setSize(window.innerWidth, window.innerHeight);
  });

  animateVisuals();
}

let time = 0;
function animateVisuals() {
  requestAnimationFrame(animateVisuals);
  time += 0.005;

  const mX = mx !== -999 ? (mx / window.innerWidth) * 2 - 1 : 0;
  const mY = my !== -999 ? -(my / window.innerHeight) * 2 + 1 : 0;

  camera.position.x += (mX * 5 - camera.position.x) * 0.05;
  camera.position.y += (15 + mY * 5 - camera.position.y) * 0.05;
  camera.lookAt(0, 0, 0);

  for (let i = 0; i < positions.length; i++) {
    const p = positions[i];
    
    // Orbital mechanics
    p.x += p.vx; p.y += p.vy; p.z += p.vz;
    p.rx += 0.01; p.ry += 0.01;
    
    // Gentle golden ratio swirling
    const angle = time * 0.5;
    const s = Math.sin(angle * (i % 3 === 0 ? 1 : -1));
    const c = Math.cos(angle * (i % 2 === 0 ? 1 : -1));

    dummy.position.set(
      p.x * c - p.z * s,
      p.y + Math.sin(time * 2 + i) * 1.5,
      p.x * s + p.z * c
    );
    
    dummy.rotation.set(p.rx, p.ry, 0);
    dummy.updateMatrix();
    instancedMesh.setMatrixAt(i, dummy.matrix);
  }

  instancedMesh.instanceMatrix.needsUpdate = true;
  renderer.render(scene, camera);
}

// Start
initPhysics();

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
