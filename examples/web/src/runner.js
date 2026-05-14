// Single-example runner: reads ?example=NAME, dynamically imports the matching
// wasm-bindgen JS shim, calls init(), and shows minimal status.

// Must run before the wasm import: wasm-bindgen captures `AudioContext` at module load.
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

// Pre-fetch bundled resources zip and expose it on `window` for `Filesystem::new_web`.
// Examples that use `Image::from_path` etc. need this.
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
  // wasm-bindgen --target web exports init() as the default export. Calling it
  // instantiates the wasm and runs the example's `main()`.
  await mod.default();
  setStatus('');
} catch (err) {
  console.error(err);
  setStatus(`error loading ${name}: ${err}`);
}

// Resume each `AudioContext` on the first user gesture and set `window.__ggezAudioState` so
// `ggez::audio::AudioContext::state()` can read. See `install_web_audio_unlock` in src/audio.rs.
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
