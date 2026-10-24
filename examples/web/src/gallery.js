// Gallery page: lists examples (from manifest.json) and loads the selected one
// into an iframe pointed at run.html?example=NAME.

const SOURCE_URL = name => `https://github.com/ggez/ggez/blob/master/examples/${name}.rs`;

const nav = document.getElementById('examples-nav');
const filter = document.getElementById('filter');
const frame = document.getElementById('example-frame');
const frameWrap = document.getElementById('frame-wrap');
const frameTitle = document.getElementById('frame-title');
const frameSource = document.getElementById('frame-source');
const frameReload = document.getElementById('frame-reload');
const framePopout = document.getElementById('frame-popout');
const placeholder = document.getElementById('placeholder');

const examples = await fetch('/manifest.json')
  .then(r => (r.ok ? r.json() : Promise.reject(new Error(r.status))))
  .catch(() => null);

let current = null;

if (!examples?.length) {
  nav.querySelector('.loading').textContent =
    'No examples built yet. Run `npm run build`.';
} else {
  renderList(examples);

  // Deep-link via ?example=NAME or #NAME.
  const initial =
    new URLSearchParams(location.search).get('example') ||
    location.hash.slice(1) ||
    null;
  if (initial && examples.includes(initial)) {
    select(initial);
  }

  filter.addEventListener('input', () => {
    const q = filter.value.trim().toLowerCase();
    const filtered = q
      ? examples.filter(name => name.toLowerCase().includes(q))
      : examples;
    renderList(filtered);
  });
}

function renderList(items) {
  const existing = nav.querySelector('ul');
  existing?.remove();
  nav.querySelector('.loading')?.remove();
  nav.querySelector('.empty')?.remove();

  if (!items.length) {
    const p = document.createElement('p');
    p.className = 'empty';
    p.textContent = 'No matches.';
    nav.appendChild(p);
    return;
  }

  const list = document.createElement('ul');
  for (const name of items) {
    const li = document.createElement('li');
    const btn = document.createElement('button');
    btn.type = 'button';
    btn.dataset.example = name;
    if (name === current) btn.setAttribute('aria-current', 'true');

    const label = document.createElement('span');
    label.className = 'ex-name';
    label.textContent = prettify(name);

    btn.appendChild(label);

    btn.addEventListener('click', () => select(name));
    li.appendChild(btn);
    list.appendChild(li);
  }
  nav.appendChild(list);
}

function prettify(name) {
  // strip leading "NN_" so the readable label looks nicer
  return name.replace(/^(\d+)_/, '$1. ').replaceAll('_', ' ');
}

function select(name) {
  current = name;
  for (const b of nav.querySelectorAll('button[data-example]')) {
    b.setAttribute(
      'aria-current',
      b.dataset.example === name ? 'true' : 'false',
    );
  }
  placeholder.hidden = true;
  frameWrap.hidden = false;
  frameTitle.textContent = prettify(name);
  frameSource.href = SOURCE_URL(name);
  // Force a reload even when re-selecting the same example.
  loadFrame(name);
  history.replaceState(null, '', `?example=${encodeURIComponent(name)}`);
  document.title = `ggez · ${prettify(name)}`;
}

function loadFrame(name) {
  frame.src = 'about:blank';
  requestAnimationFrame(() => {
    frame.src = `/run.html?example=${encodeURIComponent(name)}`;
  });
}

frameReload?.addEventListener('click', () => {
  if (current) loadFrame(current);
});
framePopout?.addEventListener('click', () => {
  if (current) window.open(`/run.html?example=${encodeURIComponent(current)}`, '_blank');
});
