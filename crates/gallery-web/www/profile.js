// Opt-in browser frame evidence for the shared Gallery. Inactive unless the
// page URL has `?profile=startup` or `?profile=scroll`; ordinary visits never
// wrap requestAnimationFrame.
//
// Each GPUI frame on the web runs inside one requestAnimationFrame callback
// (layout, prepaint, paint and WebGPU submission). Wrapping that callback with
// performance.now() measures the whole synchronous CPU frame scope from page
// start, so the window's first (asynchronously opened) draw is included;
// `run()` is timed as its own startup scope. Buffers are preallocated so
// recording never allocates per frame.
// The finished report is published as `window.__galleryProfile`.

const BUDGET_MS = 2;
const WINDOW_MS = 250;

export function installProfiler(params) {
  const mode = params.get("profile");
  if (mode !== "startup" && mode !== "scroll") return null;
  const seconds = Number(params.get("seconds") ?? "10");
  const hz = Number(params.get("hz") ?? "1000");
  const legPx = Number(params.get("legpx") ?? "40000");
  const invalid = [];
  if (!(seconds >= 2 && seconds <= 120)) invalid.push("seconds must be in 2..=120");
  if (!(hz >= 100 && hz <= 2000)) invalid.push("hz must be in 100..=2000");
  if (!(legPx >= 1000)) invalid.push("legpx must be >= 1000");

  // Generous fixed capacity: 4000 frames/s and inputs at the requested rate.
  const frameCapacity = Math.ceil(seconds + 2) * 4000;
  const frameStart = new Float64Array(frameCapacity);
  const frameEnd = new Float64Array(frameCapacity);
  const frameCount = new Float64Array(frameCapacity).fill(NaN);
  let frames = 0;
  let overflow = false;
  let recording = true;
  let counters = null; // wasm exports, set once the module has loaded
  const errors = [...invalid];

  const nativeRaf = window.requestAnimationFrame.bind(window);
  window.requestAnimationFrame = (callback) =>
    nativeRaf((timestamp) => {
      const start = performance.now();
      try {
        callback(timestamp);
      } finally {
        const end = performance.now();
        if (recording) {
          if (frames < frameCapacity) {
            frameStart[frames] = start;
            frameEnd[frames] = end;
            // Read after the scope ends: App is no longer borrowed.
            const count = counters?.gallery_successful_new_frames();
            frameCount[frames] = count ?? NaN;
            frames += 1;
          } else {
            overflow = true;
          }
        }
      }
    });

  const state = { mode, seconds, hz, legPx, initial: null };
  return {
    /** Time the synchronous startup (includes the first window draw). */
    timeRun(run) {
      const start = performance.now();
      run();
      state.initial = { start, end: performance.now() };
    },
    start(module) {
      counters = module;
      if (errors.length) return finish();
      // Browser graphics initialize asynchronously: wait (bounded) for the
      // canvas and the Gallery window before starting the measured run.
      const waitStarted = performance.now();
      const ready = () => {
        const vsync = module.gallery_vsync_status() ?? null;
        if (vsync === null || !document.querySelector("canvas")) {
          if (performance.now() - waitStarted > 20000) {
            errors.push("Gallery window or canvas did not appear within 20 s");
            return finish();
          }
          return setTimeout(ready, 50);
        }
        state.window_ready_ms = performance.now();
        const startedAt = performance.now();
        const inputs = mode === "scroll" ? startScrollDriver(state, startedAt) : null;
        setTimeout(() => finish(vsync, startedAt, inputs), seconds * 1000);
      };
      ready();
    },
  };

  function finish(vsyncInitial = null, startedAt = null, inputs = null) {
    recording = false;
    inputs?.stop();
    const durations = [];
    const rows = [];
    for (let i = 0; i < frames; i += 1) {
      const ms = frameEnd[i] - frameStart[i];
      durations.push(ms);
      rows.push({
        start_ms: frameStart[i],
        end_ms: frameEnd[i],
        cpu_ms: ms,
        successful_new_frames: Number.isNaN(frameCount[i]) ? null : frameCount[i],
        over_budget: ms > BUDGET_MS,
      });
    }
    const scopes = state.initial
      ? [state.initial.end - state.initial.start, ...durations]
      : durations;
    const windows = rateWindows(rows, startedAt);
    const full = windows.filter((w) => !w.partial && w.fps !== null).map((w) => w.fps);
    const minimumFps = full.length ? Math.min(...full) : null;
    if (overflow) errors.push("frame buffer full; later frames were not recorded");
    if (counters?.gallery_successful_new_frames() == null) {
      errors.push("successful new-frame counter unavailable");
    }
    if (minimumFps === null) errors.push("no complete >=250ms observation window");
    if (mode === "scroll" && !(inputs?.summary().dispatched > 0)) {
      errors.push("no wheel input was dispatched");
    }
    const vsyncFinal = counters?.gallery_vsync_status() ?? null;
    const cpuFailure = scopes.some((ms) => ms > BUDGET_MS);
    const fpsFailure = mode === "scroll" && minimumFps !== null && minimumFps < 500;
    const report = {
      schema: "gallery-web-frames-v1",
      mode,
      budget_ms: BUDGET_MS,
      scope:
        "performance.now() around each requestAnimationFrame callback (all synchronous GPUI frame work including WebGPU command submission) from page start, plus the synchronous run() startup; browser graphics open the window asynchronously, so its first draw is captured as an rAF scope",
      timer_resolution_note: `crossOriginIsolated=${self.crossOriginIsolated}; browsers coarsen performance.now() (about 5 us isolated, 100 us otherwise)`,
      user_agent: navigator.userAgent,
      device_pixel_ratio: devicePixelRatio,
      viewport_css_px: [innerWidth, innerHeight],
      requested: { seconds: state.seconds, input_hz: state.hz, leg_px: state.legPx },
      initial_scope_ms: state.initial ? state.initial.end - state.initial.start : null,
      cpu_summary: summarize(scopes),
      subsequent_frames_summary: summarize(durations),
      gpu_time_ms: null,
      gpu_time_reason: "WebGPU timestamp queries are not wired into the web renderer",
      vsync_initial: vsyncInitial,
      vsync_final: vsyncFinal,
      vsync_note:
        "browser presentation is rAF/compositor paced; launch Chromium with --disable-gpu-vsync,--disable-frame-rate-limit for uncapped frames",
      windows,
      minimum_observed_window_fps: minimumFps,
      inputs: inputs?.summary() ?? null,
      coverage: {
        complete_gallery_traversal: false,
        reason:
          "the browser driver cannot read the Gallery list position; legs scroll a fixed distance and rely on clamping at the ends",
      },
      frames: rows,
      errors,
      acceptance_eligible: false,
      eligibility_reasons: [
        "Gallery traversal coverage is not verifiable from the browser driver",
        "window FPS is an average per 250 ms window, not a per-frame minimum",
      ],
      exit_code: cpuFailure || fpsFailure ? 1 : 2,
    };
    window.__galleryProfile = { done: true, report };
    console.info(`Gallery profile done: exit ${report.exit_code}`);
  }
}

// Untrusted DOM wheel events on the canvas, from a MessageChannel loop that is
// independent of requestAnimationFrame. Fixed 8px steps; legs reverse after
// legPx so both ends are reached by clamping.
function startScrollDriver(state, startedAt) {
  const canvas = document.querySelector("canvas");
  const interval = 1000 / state.hz;
  const capacity = Math.ceil(state.seconds * state.hz) + 16;
  const inputTimes = new Float64Array(capacity);
  let inputs = 0;
  let down = true;
  let leg = 0;
  let turns = 0;
  let next = startedAt;
  let running = Boolean(canvas);
  const channel = new MessageChannel();
  channel.port1.onmessage = () => {
    if (!running) return;
    const now = performance.now();
    if (now >= next) {
      const rect = canvas.getBoundingClientRect();
      canvas.dispatchEvent(
        new WheelEvent("wheel", {
          deltaY: down ? 8 : -8,
          deltaMode: WheelEvent.DOM_DELTA_PIXEL,
          clientX: rect.left + rect.width / 2,
          clientY: rect.top + rect.height / 2,
          bubbles: true,
          cancelable: true,
        }),
      );
      if (inputs < capacity) inputTimes[inputs] = now;
      inputs += 1;
      leg += 8;
      if (leg >= state.legPx) {
        leg = 0;
        down = !down;
        turns += 1;
      }
      // Never burst to catch up: missed deadlines are skipped.
      next = Math.max(next + interval, now);
    }
    channel.port2.postMessage(0);
  };
  if (running) channel.port2.postMessage(0);
  return {
    stop() {
      running = false;
      channel.port1.close();
    },
    summary() {
      const recorded = Math.min(inputs, capacity);
      let longestGap = null;
      for (let i = 1; i < recorded; i += 1) {
        const gap = inputTimes[i] - inputTimes[i - 1];
        if (longestGap === null || gap > longestGap) longestGap = gap;
      }
      return {
        source: "MessageChannel loop independent of requestAnimationFrame; untrusted DOM WheelEvent on the canvas",
        canvas_found: Boolean(canvas),
        dispatched: inputs,
        requested_interval_ms: interval,
        longest_input_gap_ms: longestGap,
        turns,
        step_px: 8,
      };
    },
  };
}

function rateWindows(rows, startedAt) {
  const windows = [];
  const observed = rows.filter((r) => r.successful_new_frames !== null && r.start_ms >= (startedAt ?? 0));
  let anchor = 0;
  for (let i = 1; i < observed.length; i += 1) {
    const a = observed[anchor];
    const b = observed[i];
    const span = b.end_ms - a.end_ms;
    if (span < WINDOW_MS && i + 1 !== observed.length) continue;
    windows.push({
      start_ms: a.end_ms,
      end_ms: b.end_ms,
      fps: span > 0 ? ((b.successful_new_frames - a.successful_new_frames) * 1000) / span : null,
      partial: span < WINDOW_MS,
    });
    anchor = i;
  }
  return windows;
}

function summarize(values) {
  const sorted = [...values].sort((a, b) => a - b);
  const rank = (percent) =>
    sorted.length ? sorted[Math.max(0, Math.ceil((sorted.length * percent) / 100) - 1)] : null;
  return {
    frames: values.length,
    max_ms: sorted.length ? sorted[sorted.length - 1] : null,
    median_ms: rank(50),
    p95_ms: rank(95),
    p99_ms: rank(99),
    over_budget_frames: values.filter((ms) => ms > BUDGET_MS).length,
  };
}
