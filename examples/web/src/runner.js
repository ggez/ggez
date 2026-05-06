// Single-example runner: reads ?example=NAME, dynamically imports the matching
// wasm-bindgen JS shim, calls init(), and shows minimal status.

const status = document.getElementById('status');
const params = new URLSearchParams(location.search);
const name = params.get('example');

function setStatus(msg) {
  status.textContent = msg;
}

// AudioContext is suspended until a user gesture occurs. rodio/cpal opens in ggez init,
// so it stays suspended and no audio plays.
// Track every created context and resume on first click/keydown/touch in the page.
(function unlockAudio() {
  const created = new Set();
  const Ctx = window.AudioContext || window.webkitAudioContext;
  if (!Ctx) return;
  const Wrapped = new Proxy(Ctx, {
    construct(target, args) {
      const ctx = new target(...args);
      created.add(ctx);
      return ctx;
    },
  });
  window.AudioContext = Wrapped;
  if (window.webkitAudioContext) window.webkitAudioContext = Wrapped;

  const tryResume = () => {
    for (const ctx of created) {
      if (ctx.state === 'suspended') ctx.resume().catch(() => {});
    }
  };
  const opts = { capture: true, passive: true };
  for (const ev of ['pointerdown', 'keydown', 'touchstart', 'mousedown']) {
    window.addEventListener(ev, tryResume, opts);
  }
})();

if (!name) {
  setStatus('no ?example= in URL');
  throw new Error('missing ?example=NAME');
}

document.title = `ggez · ${name}`;
setStatus(`loading ${name}…`);

try {
  const mod = await import(/* @vite-ignore */ `/examples/${name}/${name}.js`);
  // wasm-bindgen --target web exports init() as the default export. Calling it
  // instantiates the wasm and runs the example's `main()`.
  await mod.default();
  setStatus('');
} catch (err) {
  console.error(err);
  setStatus(`error loading ${name}: ${err}`);
}
