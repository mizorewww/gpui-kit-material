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
  profiler.start({ gallery_window_open: () => false });
  const published = window.__galleryProfile;
  assert.ok(published?.done, `${mode}: no report was published`);
  assert.equal(published.report.exit_code, 2, `${mode}: an invalid run is incomplete evidence`);
  assert.ok(published.report.errors.length > 0, `${mode}: the invalid request is reported`);
  for (const [name, target] of Object.entries(published.report.targets)) {
    assert.ok(["PASS", "FAIL", "UNVERIFIED"].includes(target.verdict), `${mode}: ${name}`);
  }
}

// The probe takes its wheel target once the page has settled, past a splash, and says
// whether the wheel moved anything.
const canvas = { tagName: "CANVAS", wheels: 0, dispatchEvent() { this.wheels += 1; }, closest: () => null, parentElement: null };
const splash = { tagName: "DIV", wheels: 0, dispatchEvent() { this.wheels += 1; }, closest: () => splash, parentElement: null };
const scrollingElement = { scrollTop: 0, scrollBy() {} };
// The first look at the page finds the splash and every later one the canvas, and the
// first run's canvas draws on every read, so nothing here waits on a timer: a busy
// message loop can hold timers back on Windows runners.
let picks = 0;
let frames = 5;
Object.assign(globalThis, {
  document: {
    querySelector: () => null,
    elementFromPoint: () => (picks++ > 0 ? canvas : splash),
    scrollingElement,
    documentElement: scrollingElement,
    body: canvas,
  },
  requestAnimationFrame: (callback) => setTimeout(callback, 4),
  location: { href: "http://localhost/?profile=probe" },
  WheelEvent: class {},
  getComputedStyle: () => ({ overflowY: "visible" }),
});
Object.assign(window, {
  __galleryNewFrames: () => (frames += 1),
});
async function probe() {
  delete window.__galleryProfile;
  installProfiler(new URLSearchParams("profile=probe&seconds=0.2&settle=20&hz=500"));
  while (!window.__galleryProfile?.done) await new Promise((resolve) => setTimeout(resolve, 10));
  return window.__galleryProfile.report;
}
let report = await probe();
assert.ok(canvas.wheels > 0, "the probe's wheel reaches the canvas once the page settles");
assert.equal(report.wheel_target, "CANVAS");
assert.equal(report.wheel_inputs, splash.wheels + canvas.wheels);
assert.ok(report.new_frames > 0 && report.moved, "a canvas that drew has moved");

// A wheel that draws nothing and scrolls nothing is reported, not counted as a scroll.
window.__galleryNewFrames = () => frames;
report = await probe();
assert.equal(report.new_frames, 0);
assert.equal(report.scrolled_px, 0);
assert.equal(report.moved, false, "an idle page is not a scroll");

console.log("www/profile.js reports");
// The probe's frame channel stays open for the page's life.
process.exit(0);
