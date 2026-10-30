import { installProfiler } from "./profile.js";

// `scripts/build-site.py` injects content-hashed asset paths and their uncompressed
// sizes; a plain `build-gallery-web.sh` output uses the defaults.
const assets = {
  glue: "./wasm/gpui_kit_material_gallery_web.js",
  wasm: "./wasm/gpui_kit_material_gallery_web_bg.wasm",
  symbols: {
    "MaterialSymbolsOutlined.ttf": "./wasm/MaterialSymbolsOutlined.ttf",
    "MaterialSymbolsRounded.ttf": "./wasm/MaterialSymbolsRounded.ttf",
  },
  wasmBytes: null,
  symbolBytes: null,
  ...JSON.parse(document.getElementById("gallery-assets")?.textContent ?? "{}"),
};
const glueUrl = new URL(assets.glue, import.meta.url);
const wasmUrl = new URL(assets.wasm, import.meta.url);

const splash = document.getElementById("splash");
const status = document.getElementById("status");
const progress = document.getElementById("progress");
const bar = document.getElementById("bar");

const NEEDS_GRAPHICS =
  "The Material Gallery draws with WebGPU or WebGL2, and this browser offers neither. On a computer, enable hardware acceleration. On a phone, update the system and try a current version of Safari or Chrome.";
const CONTEXT_LOST =
  "The browser took back the graphics memory, which phones do to pages that use a lot of it. Reload to start again.";
const CRASHED = "Something went wrong. Reload the page to start again.";

document.getElementById("reload").addEventListener("click", () => location.reload());

// The canvas fills what the user can see: the visual viewport shrinks for an on-screen
// keyboard, and on iOS moves over a page that doesn't resize. A pinch-zoomed page (the
// canvas blocks pinches, but an assistive setting may not) keeps its layout.
function followVisibleArea() {
  const viewport = window.visualViewport;
  if (!viewport) return;
  const body = document.body;
  const update = () => {
    const zoomed = Math.abs(viewport.scale - 1) > 0.01;
    body.style.setProperty("--visible-height", zoomed ? "100dvh" : `${viewport.height}px`);
    body.style.transform = !zoomed && viewport.offsetTop > 0 ? `translateY(${viewport.offsetTop}px)` : "";
    if (window.scrollY) window.scrollTo(0, 0);
  };
  viewport.addEventListener("resize", update);
  viewport.addEventListener("scroll", update);
  addEventListener("orientationchange", update);
  update();
}

// The canvas's touch-action already stops pinch zoom and double-tap zoom wherever it
// is honored. iOS Safari still starts its own pinch and rubber-band gestures.
function blockPageGestures() {
  for (const gesture of ["gesturestart", "gesturechange"]) {
    document.addEventListener(gesture, (event) => event.preventDefault());
  }
  document.addEventListener("touchmove", (event) => {
    if (event.cancelable && !event.target.closest?.("#detail")) event.preventDefault();
  }, { passive: false });
}

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
async function fetchWithProgress(url, type, total, report) {
  const response = await fetch(url);
  if (!response.ok) throw new Error(`${url.pathname}: HTTP ${response.status}`);
  if (!total || !response.body) return response;
  let bytes = 0;
  const counted = response.body.pipeThrough(new TransformStream({
    transform(chunk, controller) {
      bytes += chunk.byteLength;
      report(Math.min(bytes / total, 1));
      controller.enqueue(chunk);
    },
  }));
  return new Response(counted, { headers: { "Content-Type": type } });
}

async function fetchSymbol(path, name, onBytes) {
  const url = new URL(path, import.meta.url);
  const response = await fetchWithProgress(url, "font/ttf", assets.symbolBytes?.[name], onBytes);
  return new Uint8Array(await response.arrayBuffer());
}

// The icon fonts are most of the download, and the page works without them: icons
// stay empty until their font is provided. So a slow connection waits for the module
// only. A font that can't be fetched is retried once, then its icons stay empty.
async function loadSymbolsInBackground(module) {
  const box = document.getElementById("icons-progress");
  const fill = document.getElementById("icons-bar");
  const entries = Object.entries(assets.symbols);
  const fractions = new Map();
  const report = () => {
    const sum = [...fractions.values()].reduce((total, value) => total + value, 0);
    fill.style.transform = `translateX(${(sum / entries.length - 1) * 100}%)`;
  };
  box.hidden = false;
  await Promise.all(entries.map(async ([name, path]) => {
    const onBytes = (fraction) => { fractions.set(name, fraction); report(); };
    for (const delay of [0, 2000]) {
      try {
        await new Promise((resolve) => setTimeout(resolve, delay));
        module.provide_symbol_asset(name, await fetchSymbol(path, name, onBytes));
        return;
      } catch (error) {
        console.warn(`Couldn't load ${name}`, error);
      }
    }
  }));
  box.hidden = true;
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

// Phones drop the WebGL context of a page under memory pressure, and nothing draws after.
// The event doesn't bubble, and the canvas appears only as graphics setup proceeds.
function watchContextLoss() {
  document.addEventListener("webglcontextlost", () => {
    fail("The gallery stopped drawing", CONTEXT_LOST);
  }, true);
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
followVisibleArea();
blockPageGestures();
try {
  if (profiler) {
    const module = await import(glueUrl);
    await module.default({ module_or_path: wasmUrl });
    for (const [name, path] of Object.entries(assets.symbols)) {
      module.provide_symbol_asset(name, await fetchSymbol(path, name));
    }
    profiler.timeRun(module.run);
    splash.hidden = true;
    profiler.start(module);
  } else if (typeof WebAssembly !== "object" || !(await graphicsAvailable())) {
    fail("This browser can't draw the gallery", NEEDS_GRAPHICS);
  } else {
    watchEngineFailures();
    const [module, wasm] = await Promise.all([
      import(glueUrl),
      fetchWithProgress(wasmUrl, "application/wasm", assets.wasmBytes, showProgress),
    ]);
    await module.default({ module_or_path: wasm });
    showIndeterminate("Starting…");
    watchContextLoss();
    module.run();
    removeSplashAfterFirstFrame(module);
    loadSymbolsInBackground(module);
  }
} catch (error) {
  console.error("Material Gallery failed to start", error);
  fail("The gallery couldn't start", "Check your connection and reload the page.", String(error));
}
