// Gallery page: lists examples (from manifest.json) and loads the selected one
// into an iframe pointed at run.html?example=NAME.

const nav = document.getElementById('examples-nav');
const frame = document.getElementById('example-frame');
const placeholder = document.getElementById('placeholder');

const manifest = await fetch('/manifest.json')
  .then(r => (r.ok ? r.json() : Promise.reject(new Error(r.status))))
  .catch(() => null);

if (!manifest || !manifest.examples?.length) {
  nav.innerHTML = `<p class="loading">No examples built yet. Run <code>npm run build</code>.</p>`;
} else {
  const list = document.createElement('ul');
  for (const ex of manifest.examples) {
    const li = document.createElement('li');
    const btn = document.createElement('button');
    btn.type = 'button';
    btn.textContent = ex.name;
    btn.dataset.example = ex.name;
    btn.addEventListener('click', () => select(ex.name));
    li.appendChild(btn);
    list.appendChild(li);
  }
  nav.replaceChildren(list);

  // Deep-link via ?example=NAME or #NAME.
  const initial = new URLSearchParams(location.search).get('example')
    ?? location.hash.slice(1)
    ?? null;
  if (initial && manifest.examples.some(e => e.name === initial)) {
    select(initial);
  }
}

function select(name) {
  for (const b of nav.querySelectorAll('button')) {
    b.setAttribute('aria-current', b.dataset.example === name ? 'true' : 'false');
  }
  placeholder.hidden = true;
  frame.hidden = false;
  // Force a reload even when re-selecting the same example.
  frame.src = 'about:blank';
  requestAnimationFrame(() => {
    frame.src = `/run.html?example=${encodeURIComponent(name)}`;
  });
  history.replaceState(null, '', `?example=${encodeURIComponent(name)}`);
}
