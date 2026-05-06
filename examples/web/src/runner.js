// Single-example runner: reads ?example=NAME, dynamically imports the matching
// wasm-bindgen JS shim, calls init(), and shows minimal status.

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
