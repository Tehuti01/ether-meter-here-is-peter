// ─── Scroll Reveal ───
const revealElements = document.querySelectorAll('.reveal');
const revealObserver = new IntersectionObserver((entries) => {
  entries.forEach(entry => {
    if (entry.isIntersecting) {
      entry.target.classList.add('visible');
    }
  });
}, { threshold: 0.1, rootMargin: '0px 0px -40px 0px' });
revealElements.forEach(el => revealObserver.observe(el));

// ─── Nav Scroll Effect ───
const nav = document.getElementById('nav');
let lastScroll = 0;
window.addEventListener('scroll', () => {
  const scrollY = window.scrollY;
  nav.classList.toggle('scrolled', scrollY > 60);
  lastScroll = scrollY;
}, { passive: true });

// ─── Animated Counter ───
function animateCounter(el, target, suffix = '', duration = 2000) {
  const isFloat = String(target).includes('.');
  const start = 0;
  const startTime = performance.now();
  function update(now) {
    const elapsed = now - startTime;
    const progress = Math.min(elapsed / duration, 1);
    const eased = 1 - Math.pow(1 - progress, 4);
    let current = start + (target - start) * eased;
    if (isFloat) {
      el.textContent = current.toFixed(1) + suffix;
    } else {
      el.textContent = Math.floor(current).toLocaleString() + suffix;
    }
    if (progress < 1) requestAnimationFrame(update);
  }
  requestAnimationFrame(update);
}

const statsObserver = new IntersectionObserver((entries) => {
  entries.forEach(entry => {
    if (entry.isIntersecting) {
      const el = entry.target;
      const id = el.id;
      if (id === 'stat-bandwidth') animateCounter(el, 80, '%');
      if (id === 'stat-accuracy') animateCounter(el, 99.9, '%');
      if (id === 'stat-bodies') { animateCounter(el, 5000, '+'); }
      if (id === 'stat-latency') animateCounter(el, 0, 'ms', 800);
      statsObserver.unobserve(el);
    }
  });
}, { threshold: 0.5 });
document.querySelectorAll('.stat-value').forEach(el => statsObserver.observe(el));

// ─── Floating Particles Background ───
const canvas = document.createElement('canvas');
canvas.id = 'particles';
Object.assign(canvas.style, {
  position: 'fixed', top: 0, left: 0, width: '100%', height: '100%',
  zIndex: 0, pointerEvents: 'none', opacity: '0.4'
});
document.body.prepend(canvas);

const ctx = canvas.getContext('2d');
let particles = [];
const PARTICLE_COUNT = 60;

function resizeCanvas() {
  canvas.width = window.innerWidth;
  canvas.height = window.innerHeight;
}
resizeCanvas();
window.addEventListener('resize', resizeCanvas);

for (let i = 0; i < PARTICLE_COUNT; i++) {
  particles.push({
    x: Math.random() * canvas.width,
    y: Math.random() * canvas.height,
    vx: (Math.random() - 0.5) * 0.3,
    vy: (Math.random() - 0.5) * 0.3,
    r: Math.random() * 1.5 + 0.5,
    color: Math.random() > 0.7 ? '#c8a64e' : '#00e5c8'
  });
}

function drawParticles() {
  ctx.clearRect(0, 0, canvas.width, canvas.height);
  particles.forEach(p => {
    p.x += p.vx;
    p.y += p.vy;
    if (p.x < 0) p.x = canvas.width;
    if (p.x > canvas.width) p.x = 0;
    if (p.y < 0) p.y = canvas.height;
    if (p.y > canvas.height) p.y = 0;
    ctx.beginPath();
    ctx.arc(p.x, p.y, p.r, 0, Math.PI * 2);
    ctx.fillStyle = p.color;
    ctx.globalAlpha = 0.6;
    ctx.fill();
  });
  // Draw connections
  ctx.globalAlpha = 0.08;
  ctx.strokeStyle = '#00e5c8';
  ctx.lineWidth = 0.5;
  for (let i = 0; i < particles.length; i++) {
    for (let j = i + 1; j < particles.length; j++) {
      const dx = particles[i].x - particles[j].x;
      const dy = particles[i].y - particles[j].y;
      const dist = Math.sqrt(dx * dx + dy * dy);
      if (dist < 150) {
        ctx.beginPath();
        ctx.moveTo(particles[i].x, particles[i].y);
        ctx.lineTo(particles[j].x, particles[j].y);
        ctx.stroke();
      }
    }
  }
  ctx.globalAlpha = 1;
  requestAnimationFrame(drawParticles);
}
drawParticles();

// ─── Table Row Hover Glow ───
document.querySelectorAll('.features-table tbody tr').forEach(row => {
  row.addEventListener('mouseenter', () => {
    row.style.boxShadow = 'inset 3px 0 0 var(--teal)';
  });
  row.addEventListener('mouseleave', () => {
    row.style.boxShadow = 'none';
  });
});

// ─── Smooth anchor scroll ───
document.querySelectorAll('a[href^="#"]').forEach(a => {
  a.addEventListener('click', e => {
    e.preventDefault();
    const target = document.querySelector(a.getAttribute('href'));
    if (target) target.scrollIntoView({ behavior: 'smooth', block: 'start' });
  });
});
