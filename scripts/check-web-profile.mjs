#!/usr/bin/env node
// `www/profile.js` reports only from a browser, so a mistake in building that report would
// otherwise surface only in a release run. An invalid request finishes at once, without a
// canvas or a module, which takes the whole of `finish()` with stubs for the browser.
import assert from "node:assert/strict";
import { fileURLToPath, pathToFileURL } from "node:url";
import path from "node:path";

const root = path.dirname(path.dirname(fileURLToPath(import.meta.url)));
Object.assign(globalThis, {
  self: { crossOriginIsolated: false },
  window: { requestAnimationFrame: () => 0 },
  document: { querySelector: () => null },
  devicePixelRatio: 2,
  innerWidth: 1280,
  innerHeight: 800,
});

const { installProfiler } = await import(
  pathToFileURL(path.join(root, "crates/gallery-web/www/profile.js"))
);
for (const mode of ["scroll", "startup", "idle"]) {
  delete window.__galleryProfile;
  const profiler = installProfiler(new URLSearchParams(`profile=${mode}&seconds=1`));
  profiler.start({ gallery_successful_new_frames: () => null, gallery_vsync_status: () => null });
  const published = window.__galleryProfile;
  assert.ok(published?.done, `${mode}: no report was published`);
  assert.equal(published.report.exit_code, 2, `${mode}: an invalid run is incomplete evidence`);
  assert.ok(published.report.errors.length > 0, `${mode}: the invalid request is reported`);
  for (const [name, target] of Object.entries(published.report.targets)) {
    assert.ok(["PASS", "FAIL", "UNVERIFIED"].includes(target.verdict), `${mode}: ${name}`);
  }
}
console.log("www/profile.js reports");
