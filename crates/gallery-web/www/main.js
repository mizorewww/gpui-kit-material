try {
  const { default: init, run } = await import("./wasm/gpui_kit_material_gallery_web.js");
  await init();
  run();
  document.getElementById("loading")?.remove();
} catch (error) {
  console.error("Material Gallery failed to start", error);
  document.getElementById("loading").textContent = `Unable to start Material Gallery: ${error}`;
}
