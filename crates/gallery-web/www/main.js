import { installProfiler } from "./profile.js";

// Opt-in only (?profile=...): wraps requestAnimationFrame before GPUI starts.
const profiler = installProfiler(new URLSearchParams(location.search));
try {
  const module = await import("./wasm/gpui_kit_material_gallery_web.js");
  await module.default();
  if (profiler) {
    profiler.timeRun(module.run);
  } else {
    module.run();
  }
  document.getElementById("loading")?.remove();
  profiler?.start(module);
} catch (error) {
  console.error("Material Gallery failed to start", error);
  document.getElementById("loading").textContent = `Unable to start Material Gallery: ${error}`;
}
