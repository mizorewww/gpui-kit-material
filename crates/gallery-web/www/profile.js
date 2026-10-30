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
//
// `?profile=idle` records an untouched window: every new frame after the
// settle period counts against the "idle windows draw nothing" target.
// `targets` judges the AGENTS.md budgets for WASM (steady-state x2) from the
// raw scopes; a target without data is UNVERIFIED, never PASS.

const BUDGET_MS = 2;
const WINDOW_MS = 250;
// AGENTS.md: WASM steady-state budgets are the native ones x2.
const WASM_STEADY_P95_MS = 2;
const WASM_STEADY_MAX_MS = 4;
const SETTLE_MS = 750;

export function installProfiler(params) {
  const mode = params.get("profile");
  if (mode === "probe") {
    // The same probe `--md3io` injects into md3.io, for a like-for-like
    // comparison; it waits for the Gallery to load before measuring.
    const seconds = Number(params.get("seconds") ?? "10");
    runFrameProbe({ seconds, hz: Number(params.get("hz") ?? "1000"), settleMs: 3000 }).then(
      (report) => (window.__galleryProfile = { done: true, report }),
    );
    return null;
  }
  if (mode !== "startup" && mode !== "scroll" && mode !== "idle") return null;
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
  // Hook for the md3.io-style shell: it reports the main-thread CPU duration
  // of each page switch (ms) here; the report judges them against 8 ms.
  const pageSwitches = [];
  window.__galleryPageSwitch = (ms) => {
    if (recording && Number.isFinite(ms) && pageSwitches.length < 4096) pageSwitches.push(ms);
  };
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
    const targetFailure = Object.values(targets).some((t) => t.verdict === "FAIL");
    const settled = rows.filter((r) => startedAt !== null && r.start_ms >= startedAt + SETTLE_MS);
    const steady = settled.map((r) => r.cpu_ms);
    const steadySummary = summarize(steady);
    const firstNew = rows.find((r) => r.successful_new_frames !== null && r.successful_new_frames > 0);
    const idleNewFrames =
      mode === "idle" && settled.length
        ? (settled[settled.length - 1].successful_new_frames ?? NaN) -
          (rows.filter((r) => r.start_ms < startedAt + SETTLE_MS).pop()?.successful_new_frames ?? 0)
        : null;
    const judge = (target, limit, measured, compare = "<=", note = undefined) => ({
      target,
      limit,
      measured,
      verdict:
        measured === null || Number.isNaN(measured)
          ? "UNVERIFIED"
          : (compare === "<=" ? measured <= limit : measured >= limit)
            ? "PASS"
            : "FAIL",
      ...(note ? { note } : {}),
    });
    const targets = {
      steady_cpu_p95: judge("steady-state rAF CPU p95 (WASM budget x2)", WASM_STEADY_P95_MS,
        mode === "scroll" ? steadySummary.p95_ms : null),
      steady_cpu_max: judge("steady-state rAF CPU max (WASM budget x2)", WASM_STEADY_MAX_MS,
        mode === "scroll" ? steadySummary.max_ms : null),
      idle_frames: judge("idle window draws nothing (new frames after settling)", 0, idleNewFrames),
      first_frame: judge("synchronous run() plus the first presenting rAF scope (reported; native budget 16 ms)", 16,
        state.initial && firstNew ? state.initial.end - state.initial.start + firstNew.cpu_ms : null),
      page_switch: judge("page switch main-thread CPU, worst", 8,
        pageSwitches.length ? Math.max(...pageSwitches) : null, "<=",
        pageSwitches.length ? undefined : "hook window.__galleryPageSwitch(ms) not called by the shell"),
      gpu_p95: judge("GPU time p95", 2, null, "<=", "no WebGPU timestamp queries"),
      fps_vsync_off_min: judge("new frames/s, minimum full window", 500,
        mode === "scroll" ? minimumFps : null, ">="),
    };
    const report = {
      schema: "gallery-web-frames-v2",
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
      steady_frames_summary: steadySummary,
      steady_definition: `rAF scopes starting ${SETTLE_MS} ms after the measured run started`,
      page_switch_ms: pageSwitches,
      targets,
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
      exit_code: cpuFailure || fpsFailure || targetFailure ? 1 : 2,
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

// BEGIN frame-probe
// Self-contained per-frame main-thread probe for any page, including md3.io.
// `scripts/profile-gallery.py --md3io` injects this function's source with
// agent-browser; it can also be pasted into DevTools. Each animation frame is
// timed from its requestAnimationFrame callback to a MessageChannel task
// posted there, which runs after that frame's style, layout, paint and
// commit, so it covers the page's rAF work plus the browser's main-thread
// rendering. Input handlers that run before rAF and compositor-thread
// scrolling are not included: compare with a DevTools trace (see
// docs/DEVELOPMENT.md). Wheel input: fixed 8 px steps at `hz` over the element
// under the viewport centre, reversing every `legPx`.
function runFrameProbe({ seconds = 10, hz = 1000, legPx = 4000, settleMs = 750 } = {}) {
  return new Promise((resolve) => {
    const starts = [];
    const ends = [];
    let pending = null;
    let running = true;
    const done = new MessageChannel();
    done.port1.onmessage = () => {
      if (pending !== null) {
        starts.push(pending);
        ends.push(performance.now());
        pending = null;
      }
    };
    const frame = () => {
      if (!running) return;
      pending = performance.now();
      done.port2.postMessage(0);
      requestAnimationFrame(frame);
    };
    requestAnimationFrame(frame);
    const target = document.elementFromPoint(innerWidth / 2, innerHeight / 2) ?? document.body;
    const input = new MessageChannel();
    const begun = performance.now();
    let next = begun + settleMs;
    let leg = 0;
    let down = true;
    let dispatched = 0;
    input.port1.onmessage = () => {
      if (!running) return;
      const now = performance.now();
      if (now >= next) {
        target.dispatchEvent(new WheelEvent("wheel", {
          deltaY: down ? 8 : -8, deltaMode: 0, clientX: innerWidth / 2, clientY: innerHeight / 2,
          bubbles: true, cancelable: true,
        }));
        // Untrusted wheel events do not scroll; scroll the nearest scroller too.
        let scroller = target;
        while (scroller && scroller !== document.documentElement &&
               !(scroller.scrollHeight > scroller.clientHeight &&
                 /(auto|scroll)/.test(getComputedStyle(scroller).overflowY))) {
          scroller = scroller.parentElement;
        }
        (scroller ?? document.scrollingElement).scrollBy(0, down ? 8 : -8);
        dispatched += 1;
        leg += 8;
        if (leg >= legPx) { leg = 0; down = !down; }
        next = Math.max(next + 1000 / hz, now);
      }
      input.port2.postMessage(0);
    };
    input.port2.postMessage(0);
    setTimeout(() => {
      running = false;
      input.port1.close();
      const settled = [];
      for (let i = 0; i < starts.length; i += 1) {
        if (starts[i] >= begun + settleMs) settled.push(ends[i] - starts[i]);
      }
      const sorted = [...settled].sort((a, b) => a - b);
      const rank = (q) => (sorted.length ? sorted[Math.max(0, Math.ceil(q * sorted.length) - 1)] : null);
      const span = (performance.now() - begun - settleMs) / 1000;
      resolve({
        schema: "frame-probe-v1",
        url: location.href,
        user_agent: navigator.userAgent,
        device_pixel_ratio: devicePixelRatio,
        viewport_css_px: [innerWidth, innerHeight],
        definition: "rAF callback start to a MessageChannel task posted in it (after style/layout/paint/commit)",
        frames: settled.length,
        frames_per_s: span > 0 ? settled.length / span : null,
        p50_ms: rank(0.5),
        p95_ms: rank(0.95),
        max_ms: sorted.length ? sorted[sorted.length - 1] : null,
        wheel_inputs: dispatched,
        raw_ms: settled,
      });
    }, settleMs + seconds * 1000);
  });
}
// END frame-probe
