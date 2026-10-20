// Runs the example named by ?example=NAME.

// Must run before the wasm import, which captures `AudioContext` when it loads.
installAudioUnlock();

const status = document.getElementById('status');
const params = new URLSearchParams(location.search);
const name = params.get('example');

function setStatus(msg) {
  status.textContent = msg;
}

if (!name) {
  setStatus('no ?example= in URL');
  throw new Error('missing ?example=NAME');
}

document.title = `ggez · ${name}`;
setStatus(`loading ${name}…`);

// ggez mounts this zip as its resources directory (see `Filesystem::new_web`).
async function preloadResourcesZip() {
  try {
    const resp = await fetch('/resources.zip');
    if (!resp.ok) {
      console.warn(`resources.zip not available (HTTP ${resp.status}); sync VFS lookups will fail`);
      return;
    }
    window.__GGEZ_RESOURCES_ZIP__ = new Uint8Array(await resp.arrayBuffer());
  } catch (err) {
    console.warn('failed to preload resources.zip:', err);
  }
}

try {
  await preloadResourcesZip();
  const mod = await import(/* @vite-ignore */ `/examples/${name}/${name}.js`);
  // The default export instantiates the wasm and runs `main()`.
  await mod.default();
  setStatus('');
} catch (err) {
  console.error(err);
  setStatus(`error loading ${name}: ${err}`);
}

// Resumes each `AudioContext` on the first user gesture, and mirrors its state to
// `window.__ggezAudioState` for `ggez::audio::AudioContext::state()`.
function installAudioUnlock() {
  if (window.__ggezAudioUnlockInstalled) return;
  window.__ggezAudioUnlockInstalled = true;
  const Original = window.AudioContext || window.webkitAudioContext;
  if (!Original) return;
  class GgezUnlockingAudioContext extends Original {
    constructor() {
      super(...arguments);
      const sync = () => { window.__ggezAudioState = this.state; };
      sync();
      this.addEventListener('statechange', sync);
      if (this.state !== 'suspended') return;
      const unlock = () => { this.resume().catch(() => {}); };
      const opts = { capture: true, passive: true, once: true };
      for (const ev of ['pointerdown', 'keydown', 'touchstart', 'mousedown']) {
        addEventListener(ev, unlock, opts);
      }
    }
  }
  window.AudioContext = GgezUnlockingAudioContext;
  if (window.webkitAudioContext) window.webkitAudioContext = GgezUnlockingAudioContext;
}
