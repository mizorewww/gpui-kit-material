import { installProfiler } from "./profile.js";

// `scripts/build-site.py` injects content-hashed asset paths and their total
// uncompressed size; a plain `build-gallery-web.sh` output uses the defaults.
const assets = {
  glue: "./wasm/gpui_kit_material_gallery_web.js",
  wasm: "./wasm/gpui_kit_material_gallery_web_bg.wasm",
  symbols: {
    "MaterialSymbolsOutlined.ttf": "./wasm/MaterialSymbolsOutlined.ttf",
    "MaterialSymbolsRounded.ttf": "./wasm/MaterialSymbolsRounded.ttf",
  },
  totalBytes: null,
  ...JSON.parse(document.getElementById("gallery-assets")?.textContent ?? "{}"),
};
const glueUrl = new URL(assets.glue, import.meta.url);
const wasmUrl = new URL(assets.wasm, import.meta.url);

const splash = document.getElementById("splash");
const status = document.getElementById("status");
const progress = document.getElementById("progress");
const bar = document.getElementById("bar");

const NEEDS_GRAPHICS =
  "The Material Gallery needs WebGPU or WebGL2. Enable hardware acceleration, or try a current version of Chrome, Edge, Safari or Firefox.";
const CRASHED = "Something went wrong. Reload the page to start again.";

function fail(title, message, detail = "") {
  document.getElementById("title").textContent = title;
  status.textContent = message;
  document.getElementById("detail").textContent = detail;
  splash.classList.remove("done");
  splash.classList.add("failed");
  splash.hidden = false;
}

function showProgress(fraction) {
  progress.classList.remove("indeterminate");
  progress.setAttribute("aria-valuenow", String(Math.round(fraction * 100)));
  bar.style.transform = `translateX(${(fraction - 1) * 100}%)`;
}

function showIndeterminate(message) {
  progress.classList.add("indeterminate");
  progress.removeAttribute("aria-valuenow");
  bar.style.transform = "";
  status.textContent = message;
}

// True when the browser offers either backend the engine can use. The engine
// repeats this probe with its real canvas; this one only avoids downloading the
// gallery on a browser that can never draw it.
async function graphicsAvailable() {
  try {
    if (await navigator.gpu?.requestAdapter()) return true;
  } catch {
    // Fall through to WebGL2.
  }
  const context = document.createElement("canvas").getContext("webgl2");
  context?.getExtension("WEBGL_lose_context")?.loseContext();
  return Boolean(context);
}

// A streaming Response keeps instantiateStreaming for the module. `Content-Length`
// counts compressed bytes while the stream yields decompressed ones, so sizes come
// from the build; without them (a plain local build) the bar stays indeterminate.
const received = new Map();
async function fetchWithProgress(url, type) {
  const response = await fetch(url);
  if (!response.ok) throw new Error(`${url.pathname}: HTTP ${response.status}`);
  if (!assets.totalBytes || !response.body) return response;
  const counted = response.body.pipeThrough(new TransformStream({
    transform(chunk, controller) {
      received.set(url, (received.get(url) ?? 0) + chunk.byteLength);
      const bytes = [...received.values()].reduce((sum, value) => sum + value, 0);
      showProgress(Math.min(bytes / assets.totalBytes, 1));
      controller.enqueue(chunk);
    },
  }));
  return new Response(counted, { headers: { "Content-Type": type } });
}

// Starts every download now; the returned function delivers them to the module.
function fetchSymbols() {
  const files = Object.entries(assets.symbols).map(async ([name, path]) => {
    const response = await fetchWithProgress(new URL(path, import.meta.url), "font/ttf");
    return [name, new Uint8Array(await response.arrayBuffer())];
  });
  return async (module) => {
    for (const [name, bytes] of await Promise.all(files)) {
      module.provide_symbol_asset(name, bytes);
    }
  };
}

// Graphics setup is asynchronous inside `run()`, so its failure reaches the page
// only as the engine's log line and a bare paragraph appended to the body.
function watchEngineFailures() {
  const consoleError = console.error;
  console.error = (...args) => {
    consoleError.apply(console, args);
    const text = args.map(String).join(" ");
    if (text.includes("Failed to initialize browser graphics")) {
      document.querySelectorAll("body > p").forEach((node) => node.remove());
      fail("This browser can't draw the gallery", NEEDS_GRAPHICS, text);
    } else if (text.includes("panicked at")) {
      fail("The gallery stopped", CRASHED, text);
    }
  };
  addEventListener("error", (event) => {
    if (event.error instanceof WebAssembly.RuntimeError) {
      fail("The gallery stopped", CRASHED, String(event.error));
    }
  });
}

// Keep the splash up until the engine has really submitted a frame.
function removeSplashAfterFirstFrame(module) {
  const started = performance.now();
  const poll = () => {
    if (splash.classList.contains("failed")) return;
    const frames = module.gallery_successful_new_frames();
    // A backend without a frame counter still reports its VSync state once the
    // window exists, and that is the latest signal it can give.
    if (frames > 0 || (frames === undefined && module.gallery_vsync_status() !== undefined)) {
      splash.classList.add("done");
      setTimeout(() => { splash.hidden = true; }, 200);
      return;
    }
    if (performance.now() - started > 15000) {
      showIndeterminate("Still starting. This can take a while without a hardware GPU…");
    }
    requestAnimationFrame(poll);
  };
  requestAnimationFrame(poll);
}

// Opt-in only (?profile=...): wraps requestAnimationFrame before GPUI starts, so
// that path adds no polling or stream wrapping to the frames it measures.
const profiler = installProfiler(new URLSearchParams(location.search));
try {
  if (profiler) {
    const module = await import(glueUrl);
    await module.default({ module_or_path: wasmUrl });
    await fetchSymbols()(module);
    profiler.timeRun(module.run);
    splash.hidden = true;
    profiler.start(module);
  } else if (typeof WebAssembly !== "object" || !(await graphicsAvailable())) {
    fail("This browser can't draw the gallery", NEEDS_GRAPHICS);
  } else {
    watchEngineFailures();
    const provideSymbols = fetchSymbols();
    const [module, wasm] = await Promise.all([
      import(glueUrl),
      fetchWithProgress(wasmUrl, "application/wasm"),
    ]);
    await module.default({ module_or_path: wasm });
    await provideSymbols(module);
    showIndeterminate("Starting…");
    module.run();
    removeSplashAfterFirstFrame(module);
  }
} catch (error) {
  console.error("Material Gallery failed to start", error);
  fail("The gallery couldn't start", "Check your connection and reload the page.", String(error));
}
