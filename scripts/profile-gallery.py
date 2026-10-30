#!/usr/bin/env python3
"""Build and record a finite native Gallery CPU profile with binary provenance.

The report's scope and exit status remain authoritative. This runner never turns
diagnostic or incomplete measurements into a whole-gallery conformance PASS.
"""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import subprocess
import sys


ROOT = Path(__file__).resolve().parents[1]


def output(*args):
    return subprocess.check_output(args, cwd=ROOT)


def repository_identity(root):
    # Include staged, unstaged and non-ignored untracked inputs without copying
    # their contents into the evidence. Ignored build output is not source.
    def git(*args):
        return subprocess.check_output(["git", "-C", str(root), *args])

    files = git("ls-files", "-z", "--cached", "--others", "--exclude-standard")
    digest = hashlib.sha256()
    for name in sorted(set(files.split(b"\0")) - {b""}):
        path = root / os.fsdecode(name)
        digest.update(name + b"\0")
        if path.is_symlink():
            digest.update(b"symlink\0" + os.fsencode(os.readlink(path)))
        elif path.is_file():
            digest.update(b"file\0")
            with path.open("rb") as stream:
                while chunk := stream.read(1024 * 1024):
                    digest.update(chunk)
        else:
            digest.update(b"missing\0")
        digest.update(b"\0")
    return {
        "repository": str(root),
        "commit": git("rev-parse", "HEAD").decode().strip(),
        "status": git("status", "--porcelain", "--untracked-files=all").decode(),
        "source_files_sha256": digest.hexdigest(),
        "scope": "tracked and non-ignored untracked files; external configuration is not fingerprinted",
    }


def source_identity(features):
    # Match the feature selection of the build. Resolve metadata on every check
    # so changing a path dependency or its feature graph cannot evade comparison.
    # Do not persist raw metadata: it can contain registry URLs and arbitrary
    # package metadata. Keep only local package identities and dependency edges.
    command = ["cargo", "metadata", "--locked", "--format-version", "1",
               "--no-default-features", "--features",
               ",".join("gpui-kit-material/" + feature for feature in features.split(","))]
    metadata = json.loads(output(*command))
    packages = {package["id"]: package for package in metadata["packages"]}
    nodes = {node["id"]: node for node in metadata["resolve"]["nodes"]}
    material_manifest = ROOT / "crates/material/Cargo.toml"
    seeds = [key for key, package in packages.items()
             if Path(package["manifest_path"]).resolve() == material_manifest]
    if len(seeds) != 1:
        raise RuntimeError("metadata did not resolve exactly one Material package")
    reachable = set()
    pending = seeds[:]
    while pending:
        key = pending.pop()
        if key not in reachable:
            reachable.add(key)
            pending.extend(nodes[key]["dependencies"])

    root = ROOT.resolve()
    repositories = {}
    local_packages = []
    for key in sorted(reachable):
        package = packages[key]
        if package["source"] is not None:
            continue
        manifest = Path(package["manifest_path"]).resolve()
        identity = {
            "name": package["name"], "version": package["version"],
            "manifest": str(manifest),
            "manifest_sha256": hashlib.sha256(manifest.read_bytes()).hexdigest(),
            "features": sorted(nodes[key]["features"]),
            "dependencies": sorted({packages[dep]["name"] + "@" + packages[dep]["version"]
                                    for dep in nodes[key]["dependencies"]}),
        }
        if root not in manifest.parents:
            # A non-Git external source cannot be fingerprinted by this policy;
            # fail closed instead of silently omitting it from the evidence.
            repo = Path(subprocess.check_output(
                ["git", "-C", str(manifest.parent), "rev-parse", "--show-toplevel"]
            ).decode().strip()).resolve()
            identity["external_repository"] = str(repo)
            if str(repo) not in repositories:
                repositories[str(repo)] = repository_identity(repo)
        local_packages.append(identity)
    return {
        "root": repository_identity(root),
        "metadata_command": command,
        "local_packages": sorted(local_packages, key=lambda package: package["manifest"]),
        "external_repositories": [repositories[key] for key in sorted(repositories)],
        "dependency_scope": "Material resolved dependency closure, including target-conditional and dev dependencies; registry/Git sources are pinned by Cargo.lock",
    }


def runner_exit_code(capture_code, inputs_stable, scroll, report_path):
    """0: complete run, CPU and FPS gates both pass. 1: a measured gate
    failed. 2: evidence incomplete, unstable or unverifiable.

    A startup run measures only the CPU gate, so it can fail (1) but never
    pass (0). Only a scroll run that the gallery itself reports as eligible,
    with exit 0, complete traversal and an eligible CPU report, returns 0.
    """
    if not inputs_stable:
        return 2
    if capture_code == 1:
        return 1
    if capture_code != 0 or not scroll:
        return 2
    try:
        report = json.loads(report_path.read_text())
    except (OSError, ValueError):
        return 2
    passed = (report.get("exit_code") == 0
              and report.get("acceptance_eligible") is True
              and not report.get("errors")
              and (report.get("cpu") or {}).get("acceptance_eligible") is True
              and (report.get("coverage") or {}).get("complete_gallery_traversal") is True)
    return 0 if passed else 2


def web_profile(args, destination):
    """Bounded headless browser run of the WASM Gallery (www/profile.js)."""
    import functools
    import http.server
    import threading
    import time
    import uuid

    build = ["./scripts/build-gallery-web.sh", "--release", "--" + args.backend]
    provenance = {"build_command": build, "requested_backend": args.backend,
                  "requested_scenario": "web-" + ("scroll" if args.scroll else "startup"),
                  "source_before_build": repository_identity(ROOT),
                  "acceptance_eligible": False}

    def save():
        (destination / "build.json").write_text(json.dumps(provenance, indent=2) + "\n")

    save()
    with (destination / "build.log").open("wb") as log:
        built = subprocess.run(build, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT)
    provenance["build_exit_code"] = built.returncode
    provenance["source_after_build"] = repository_identity(ROOT)
    www = ROOT / "crates/gallery-web/www"
    provenance["wasm_sha256"] = hashlib.sha256(
        (www / "wasm/gpui_kit_material_gallery_web_bg.wasm").read_bytes()).hexdigest() \
        if built.returncode == 0 else None
    save()
    if built.returncode:
        return 2

    class Handler(http.server.SimpleHTTPRequestHandler):
        def end_headers(self):
            # Cross-origin isolation gives performance.now() its finest resolution.
            self.send_header("Cross-Origin-Opener-Policy", "same-origin")
            self.send_header("Cross-Origin-Embedder-Policy", "require-corp")
            self.send_header("Cache-Control", "no-store")
            super().end_headers()

        def log_message(self, *_):
            pass

    server = http.server.ThreadingHTTPServer(
        ("127.0.0.1", 0), functools.partial(Handler, directory=str(www)))
    threading.Thread(target=server.serve_forever, daemon=True).start()
    session = "gallery-profile-" + uuid.uuid4().hex[:12]
    query = f"profile={'scroll' if args.scroll else 'startup'}&seconds={args.seconds}"
    if args.input_hz is not None:
        query += f"&hz={args.input_hz}"
    url = f"http://127.0.0.1:{server.server_address[1]}/?{query}"
    browser_args = "--disable-gpu-vsync,--disable-frame-rate-limit,--enable-unsafe-webgpu"
    provenance.update(url=url, browser_args=browser_args, viewport=[1280, 800])

    def browser(*command, timeout=60):
        return subprocess.run(["agent-browser", "--session", session, "--args", browser_args,
                               *command], capture_output=True, text=True, timeout=timeout)

    code = 2
    try:
        browser("set", "viewport", "1280", "800")
        opened = browser("open", url)
        if opened.returncode:
            provenance["capture_error"] = opened.stderr + opened.stdout
            return 2
        deadline = time.monotonic() + args.seconds + 60
        report = None
        while time.monotonic() < deadline:
            time.sleep(1)
            result = browser("eval", "JSON.stringify(window.__galleryProfile ?? null)")
            if result.returncode:
                continue
            value = json.loads(result.stdout)
            value = json.loads(value) if isinstance(value, str) else value
            if value and value.get("done"):
                report = value["report"]
                break
        if report is None:
            provenance["capture_error"] = "browser profile did not finish before the deadline"
        else:
            (destination / "frames.json").write_text(json.dumps(report, indent=2) + "\n")
            code = report.get("exit_code", 2)
        console = browser("console")
        (destination / "console.txt").write_text(console.stdout)
    except subprocess.TimeoutExpired:
        provenance["capture_error"] = "browser command timed out"
    finally:
        browser("close")
        server.shutdown()
        provenance["capture_exit_code"] = code
        provenance["source_after_capture"] = repository_identity(ROOT)
        provenance["inputs_stable"] = (
            provenance["source_after_build"] == provenance["source_after_capture"])
        save()
    # The browser report is never acceptance-eligible: 1 for a measured
    # failure, otherwise 2.
    return 1 if code == 1 and provenance["inputs_stable"] else 2


# ---------------------------------------------------------------------------
# AGENTS.md performance targets.
#
# Every target is judged from raw per-frame data with nearest-rank quantiles;
# maxima are reported beside them and an average never decides a verdict. A
# target without the data to judge it is UNVERIFIED, never PASS.

MS = 1_000_000
TARGETS = {
    "steady_cpu_p95": ("steady-state main-thread CPU p95 (excluding drawable wait)", 1 * MS),
    "steady_cpu_max": ("steady-state main-thread CPU max (excluding drawable wait)", 2 * MS),
    "drawable_wait_on_main_thread": ("main-thread drawable wait (must not occur; any frame > 0.1 ms fails)", MS // 10),
    "idle_frames": ("idle window draws and presents nothing", 0),
    "forced_redraw_max": ("forced redraw of an unchanged page, every frame", 3 * MS // 10),
    "first_frame_cpu": ("first frame: window open to first presented frame, main-thread CPU", 16 * MS),
    "page_switch": ("switching Gallery pages", 8 * MS),
    "gpu_p95": ("GPU time p95 (reported separately)", 2 * MS),
    "fps_vsync_off_min": ("new frames/s with vsync off, minimum 100 ms window", 500),
    "missed_refreshes_vsync_on": ("missed refreshes with vsync on at 120 Hz", 0),
}
# WASM: the steady-state budgets x2 (AGENTS.md); other targets keep their values.
WASM_FACTOR = {"steady_cpu_p95": 2, "steady_cpu_max": 2}


def nearest_rank(values, q):
    values = sorted(values)
    if not values:
        return None
    return values[max(0, min(len(values), math.ceil(q * len(values))) - 1)]


def distribution(values):
    values = [v for v in values if v is not None]
    return {"count": len(values), "p50_ns": nearest_rank(values, 0.5),
            "p95_ns": nearest_rank(values, 0.95), "p99_ns": nearest_rank(values, 0.99),
            "max_ns": max(values) if values else None}


def verdict(name, measured, *, wasm=False, compare="<=", note=None):
    label, limit = TARGETS[name]
    limit *= WASM_FACTOR.get(name, 1) if wasm else 1
    if measured is None:
        result = "UNVERIFIED"
    elif compare == "<=":
        result = "PASS" if measured <= limit else "FAIL"
    else:
        result = "PASS" if measured >= limit else "FAIL"
    entry = {"target": label, "limit": limit, "measured": measured, "verdict": result}
    if note:
        entry["note"] = note
    return entry


def bench_steady(report):
    """Steady frames of an engine_bench report: drawn frames after settling."""
    return [s for s in report.get("raw_scopes", []) if s.get("steady") and s.get("draw_count")]


def missed_refreshes(frame_end_ns, refresh_ns):
    """Intervals between successive new frames longer than 1.5 refreshes,
    counted as the refreshes they skipped."""
    if not refresh_ns or len(frame_end_ns) < 2:
        return None
    missed = 0
    for a, b in zip(frame_end_ns, frame_end_ns[1:]):
        gap = b - a
        if gap > 1.5 * refresh_ns:
            missed += round(gap / refresh_ns) - 1
    return missed


def bench_targets(reports):
    """Target verdicts from engine_bench reports keyed by scenario-vsync."""
    out = {"source": "crates/material/examples/engine_bench.rs (standalone, not the Gallery)",
           "metrics": {}, "targets": {}}
    m = out["metrics"]
    scroll = reports.get("scroll-off")
    if scroll:
        frames = bench_steady(scroll)
        m["scroll_vsync_off"] = {
            "steady_frames": len(frames),
            "cpu_excluding_drawable_wait": distribution(s["cpu_excluding_drawable_wait_ns"] for s in frames),
            "drawable_wait": distribution(s["drawable_wait_ns"] for s in frames),
            "frames_with_drawable_wait_over_0_1ms": sum(s["drawable_wait_ns"] > MS // 10 for s in frames),
            "layout": distribution(s.get("request_layout_ns") for s in frames),
            "prepaint": distribution(s.get("prepaint_ns") for s in frames),
            "paint": distribution(s.get("paint_ns") for s in frames),
            "present": distribution(s.get("present_ns") for s in frames),
            "gpu": distribution(s.get("gpu_ns") for s in frames),
            "new_frames_per_s": scroll["summary"]["new_frames_per_s"],
            "min_100ms_window_new_frames_per_s": scroll["summary"]["min_100ms_window_new_frames_per_s"],
            "engine_counters": scroll.get("engine_counters"),
        }
        s = m["scroll_vsync_off"]
        out["targets"]["steady_cpu_p95"] = verdict("steady_cpu_p95", s["cpu_excluding_drawable_wait"]["p95_ns"])
        out["targets"]["steady_cpu_max"] = verdict("steady_cpu_max", s["cpu_excluding_drawable_wait"]["max_ns"])
        out["targets"]["drawable_wait_on_main_thread"] = verdict(
            "drawable_wait_on_main_thread", s["drawable_wait"]["max_ns"],
            note=f"{s['frames_with_drawable_wait_over_0_1ms']} of {len(frames)} frames waited > 0.1 ms")
        out["targets"]["gpu_p95"] = verdict("gpu_p95", s["gpu"]["p95_ns"])
        out["targets"]["fps_vsync_off_min"] = verdict(
            "fps_vsync_off_min", s["min_100ms_window_new_frames_per_s"], compare=">=",
            note=f"overall {s['new_frames_per_s']:.1f} new frames/s; successful submissions only")
    on = reports.get("scroll-on")
    if on:
        refresh = on.get("display_refresh_interval_ns")
        ends = [t for t in on.get("raw_successful_new_frame_end_ns", [])
                if on["steady_window_ns"][0] <= t <= on["steady_window_ns"][1]]
        missed = missed_refreshes(ends, refresh)
        hz = round(1e9 / refresh, 1) if refresh else None
        m["scroll_vsync_on"] = {"display_refresh_hz": hz, "missed_refreshes": missed,
                                "new_frames_per_s": on["summary"]["new_frames_per_s"],
                                "cpu_excluding_drawable_wait": distribution(
                                    s["cpu_excluding_drawable_wait_ns"] for s in bench_steady(on))}
        note = f"measured on a {hz} Hz display"
        if hz is None or abs(hz - 120) > 1:
            note += "; 120 Hz unverified on this display"
            out["targets"]["missed_refreshes_vsync_on"] = verdict(
                "missed_refreshes_vsync_on", None, note=note + f" ({missed} missed at {hz} Hz)")
        else:
            out["targets"]["missed_refreshes_vsync_on"] = verdict("missed_refreshes_vsync_on", missed, note=note)
    idle = reports.get("idle-off") or reports.get("idle-on")
    if idle:
        drawn = idle["summary"]["steady_frames_drawn"]
        presented = idle["summary"]["steady_frames_presented"]
        m["idle"] = {"frames_drawn": drawn, "frames_presented": presented,
                     "seconds": (idle["steady_window_ns"][1] - idle["steady_window_ns"][0]) / 1e9}
        out["targets"]["idle_frames"] = verdict("idle_frames", max(drawn, presented))
    redraw = reports.get("redraw-off") or reports.get("redraw-on")
    if redraw:
        frames = bench_steady(redraw)
        m["forced_redraw"] = {"cpu_excluding_drawable_wait": distribution(
            s["cpu_excluding_drawable_wait_ns"] for s in frames)}
        dist = m["forced_redraw"]["cpu_excluding_drawable_wait"]
        out["targets"]["forced_redraw_max"] = verdict(
            "forced_redraw_max", dist["max_ns"], note=f"p95 {dist['p95_ns']} ns; root view notified each frame")
    startups = [r for k, r in sorted(reports.items()) if k.startswith("startup")]
    if startups:
        cpu = [r["summary"].get("first_frame_main_thread_cpu_ns") for r in startups]
        m["first_frame"] = {
            "runs": len(startups),
            "main_thread_cpu_ns": cpu,
            "initial_draw_ns": [r["summary"].get("initial_draw_ns") for r in startups],
            "open_window_cpu_ns": [r.get("first_frame_cpu", {}).get("open_window_cpu_ns") for r in startups],
            "wall_to_first_frame_ns": [r.get("first_frame_cpu", {}).get("wall_to_after_first_frame_ns") for r in startups],
        }
        worst = None if any(c is None for c in cpu) else max(cpu)
        out["targets"]["first_frame_cpu"] = verdict("first_frame_cpu", worst, note="worst of the runs")
    out["targets"]["page_switch"] = verdict(
        "page_switch", None, note="the standalone benchmark has no pages; measured from the Gallery shell")
    return out


def gallery_targets(frames, *, page_switch_ns=None):
    """Target verdicts from a native Gallery frames.json (startup or scroll)."""
    cpu = frames.get("cpu", frames)
    scopes = cpu.get("cpu_scopes", [])
    out = {"source": "native Gallery frames.json", "metrics": {}, "targets": {}}
    if not scopes:
        return out
    steady = [s for s in scopes[1:] if s.get("draw_count")]
    excl = [s["cpu_elapsed_ns"] - s.get("presentation_wait_ns", 0) for s in steady]
    waits = [s.get("presentation_wait_ns", 0) for s in steady]
    gpu = [f.get("gpu_time_ns") for f in cpu.get("frames", [])[1:]]
    first = cpu.get("frames", [{}])[0] if cpu.get("frames") else {}
    m = out["metrics"]
    m["steady_cpu_excluding_drawable_wait"] = distribution(excl)
    m["drawable_wait"] = distribution(waits)
    m["gpu"] = distribution(gpu)
    initial = scopes[0]["cpu_elapsed_ns"] if scopes[0].get("source") == "InitialDraw" else None
    m["first_frame_cpu_ns"] = (initial + first["cpu_frame_ns"]) if initial and first.get("cpu_frame_ns") else None
    out["targets"]["steady_cpu_p95"] = verdict("steady_cpu_p95", m["steady_cpu_excluding_drawable_wait"]["p95_ns"])
    out["targets"]["steady_cpu_max"] = verdict("steady_cpu_max", m["steady_cpu_excluding_drawable_wait"]["max_ns"])
    out["targets"]["drawable_wait_on_main_thread"] = verdict(
        "drawable_wait_on_main_thread", m["drawable_wait"]["max_ns"])
    out["targets"]["gpu_p95"] = verdict("gpu_p95", m["gpu"]["p95_ns"])
    out["targets"]["first_frame_cpu"] = verdict(
        "first_frame_cpu", m["first_frame_cpu_ns"],
        note="InitialDraw scope plus the first presenting frame's scope; window creation CPU is not included in Gallery evidence")
    windows = [w["fps"] for w in frames.get("windows", []) if not w.get("partial")]
    if frames.get("schema") == "gallery-scroll-observations-v1":
        out["targets"]["fps_vsync_off_min"] = verdict(
            "fps_vsync_off_min", min(windows) if windows else None, compare=">=",
            note="Gallery driver windows (>= 250 ms)")
    # Page switching: the md3.io-style shell reports `page_switch_ns` per switch
    # once it lands; until then this stays UNVERIFIED.
    switches = page_switch_ns if page_switch_ns is not None else frames.get("page_switch_ns")
    out["targets"]["page_switch"] = verdict(
        "page_switch", max(switches) if switches else None,
        note=None if switches else "hook: the Gallery shell must report page_switch_ns (one CPU duration per switch)")
    return out


def print_targets(table, file=sys.stdout):
    for name, entry in table["targets"].items():
        measured = entry["measured"]
        if measured is None:
            shown = "-"
        elif name.startswith("fps"):
            shown = f"{measured:.0f}/s"
        elif name in ("idle_frames", "missed_refreshes_vsync_on"):
            shown = str(measured)
        else:
            shown = f"{measured / MS:.3f} ms"
        print(f"  {entry['verdict']:<10} {name:<30} {shown:>12}  {entry['target']}"
              + (f" [{entry['note']}]" if entry.get("note") else ""), file=file)


BENCH_SCENARIOS = ("startup-1", "startup-2", "startup-3", "scroll-off", "scroll-on", "idle-off", "redraw-off")


def bench_profile(args, destination):
    """Build and run the standalone engine benchmark scenarios."""
    features = "serde,embedded-symbols,frame-profiler" + (",gpui-fast" if args.backend == "fast" else "")
    command = ["cargo", "build", "--locked", "--release", "-p", "gpui-kit-material",
               "--example", "engine_bench", "--no-default-features", "--features", features,
               "--message-format=json-render-diagnostics"]
    provenance = {"build_command": command, "requested_backend": args.backend,
                  "source_before_build": repository_identity(ROOT),
                  "load_average_before": os.getloadavg(), "acceptance_eligible": False}
    with (destination / "build.log").open("wb") as log, (destination / "build-events.jsonl").open("wb") as events:
        built = subprocess.run(command, cwd=ROOT, stdout=events, stderr=log)
    provenance["build_exit_code"] = built.returncode
    if built.returncode:
        (destination / "build.json").write_text(json.dumps(provenance, indent=2) + "\n")
        return 2
    binary = None
    for line in (destination / "build-events.jsonl").read_text().splitlines():
        event = json.loads(line)
        if (event.get("reason") == "compiler-artifact" and event.get("executable")
                and event.get("target", {}).get("name") == "engine_bench"):
            binary = Path(event["executable"])
    provenance["binary"] = str(binary)
    provenance["binary_sha256"] = hashlib.sha256(binary.read_bytes()).hexdigest()
    reports = {}
    seconds = max(2.0, min(args.seconds, 30.0))
    for name in BENCH_SCENARIOS:
        scenario, _, rest = name.partition("-")
        vsync = rest if rest in ("on", "off") else "off"
        out = destination / f"{name}.json"
        run = [str(binary), "--scenario", scenario, "--vsync", vsync,
               "--seconds", str(seconds), "--content", args.bench_content, "--out", str(out)]
        provenance.setdefault("runs", []).append({"command": run, "load_average": os.getloadavg()})
        with (destination / f"{name}.log").open("wb") as log:
            try:
                subprocess.run(run, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, timeout=seconds + 60)
            except subprocess.TimeoutExpired:
                provenance["runs"][-1]["error"] = "timed out; process killed"
        if out.exists():
            reports[name] = json.loads(out.read_text())
    provenance["load_average_after"] = os.getloadavg()
    provenance["source_after_capture"] = repository_identity(ROOT)
    provenance["inputs_stable"] = provenance["source_before_build"] == provenance["source_after_capture"]
    (destination / "build.json").write_text(json.dumps(provenance, indent=2) + "\n")
    table = bench_targets(reports)
    table["load_average"] = [provenance["load_average_before"], provenance["load_average_after"]]
    (destination / "targets.json").write_text(json.dumps(table, indent=2) + "\n")
    print(f"Engine benchmark targets ({args.bench_content} rows, load {provenance['load_average_before']}):")
    print_targets(table)
    failed = any(t["verdict"] == "FAIL" for t in table["targets"].values())
    return 1 if failed else 2


def interaction_frames(data):
    """A gallery-interactions-v1 report as the frames `gallery_targets` judges.
    The first drawing scope after each page switch is that switch's cost, not a
    steady-state frame; `page_switch_ns` reports it."""
    scopes = data["all_scopes"]
    switch_frames = set()
    for label, at in data["marks_ns"]:
        if label.startswith("switch:"):
            first = next((i for i, s in enumerate(scopes)
                          if s["start_ns"] >= at and s["draw_count"]), None)
            if first is not None:
                switch_frames.add(first)
    initial = [s for s in scopes if s["source"] == "InitialDraw"]
    steady = [s for i, s in enumerate(scopes)
              if s["source"] != "InitialDraw" and i not in switch_frames]
    presented = [s for s in steady if s["present_count"]]
    frames = [{"gpu_time_ns": s["gpu_time_ns"]} for s in presented]
    if frames and presented:
        frames[0]["cpu_frame_ns"] = presented[0]["cpu_elapsed_ns"]
    return {"cpu_scopes": initial + steady, "frames": frames,
            "page_switch_ns": data["page_switch_ns"], "schema": data["schema"]}


def summarize(paths):
    """Print target verdicts for existing engine_bench reports or Gallery frames.json."""
    reports = {}
    code = 2
    for path in paths:
        data = json.loads(Path(path).read_text())
        if data.get("schema") == "material-engine-bench-v1":
            key = data["scenario"]
            key = f"{key}-{len([k for k in reports if k.startswith('startup')]) + 1}" if key == "startup" \
                else f"{key}-{'on' if data['requested_vsync'] else 'off'}"
            reports[key] = data
        else:
            if data.get("schema") == "gallery-interactions-v1":
                data = interaction_frames(data)
            table = gallery_targets(data)
            print(f"{path}:")
            print_targets(table)
            code = 1 if any(t["verdict"] == "FAIL" for t in table["targets"].values()) else code
    if reports:
        table = bench_targets(reports)
        print("engine_bench:")
        print_targets(table)
        code = 1 if any(t["verdict"] == "FAIL" for t in table["targets"].values()) else code
    return code


def md3io_probe(args, destination):
    """Run `www/profile.js`'s frame probe on a third-party page (md3.io) in
    Chromium via agent-browser, with the same flags as `--web`."""
    import time
    import uuid
    source = (ROOT / "crates/gallery-web/www/profile.js").read_text()
    probe = source.split("// BEGIN frame-probe\n", 1)[1].split("// END frame-probe", 1)[0]
    session = "md3io-probe-" + uuid.uuid4().hex[:12]
    browser_args = "--disable-gpu-vsync,--disable-frame-rate-limit"
    provenance = {"url": args.md3io, "browser_args": browser_args, "viewport": [1280, 800],
                  "seconds": args.seconds, "probe": "www/profile.js runFrameProbe",
                  "load_average": os.getloadavg(), "acceptance_eligible": False}

    def browser(*command, timeout=60):
        return subprocess.run(["agent-browser", "--session", session, "--args", browser_args,
                               *command], capture_output=True, text=True, timeout=timeout)

    code = 2
    try:
        browser("set", "viewport", "1280", "800")
        opened = browser("open", args.md3io)
        if opened.returncode:
            provenance["error"] = opened.stderr + opened.stdout
            return 2
        time.sleep(3)
        browser("eval", f"window.__probe = null; ({probe})({{seconds: {args.seconds}}})"
                        ".then(r => window.__probe = r); 0")
        deadline = time.monotonic() + args.seconds + 30
        while time.monotonic() < deadline:
            time.sleep(1)
            result = browser("eval", "JSON.stringify(window.__probe ?? null)")
            if result.returncode:
                continue
            value = json.loads(result.stdout)
            value = json.loads(value) if isinstance(value, str) else value
            if value:
                (destination / "probe.json").write_text(json.dumps(value, indent=2) + "\n")
                print(f"{args.md3io}: {value['frames']} frames, p50 {value['p50_ms']} ms, "
                      f"p95 {value['p95_ms']} ms, max {value['max_ms']} ms")
                code = 0
                break
        else:
            provenance["error"] = "probe did not finish before the deadline"
    finally:
        browser("close")
        (destination / "probe-provenance.json").write_text(json.dumps(provenance, indent=2) + "\n")
    return code


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path, nargs="?",
                        help="new evidence directory outside the repository")
    parser.add_argument("--seconds", type=float, default=10)
    parser.add_argument("--backend", choices=("upstream", "fast"), default="fast")
    parser.add_argument("--scroll", action="store_true",
                        help="record full top-bottom-top Gallery traversals with vertical sync disabled")
    parser.add_argument("--scroll-input", choices=("wheel", "programmatic"),
                        help="scroll input path (default: programmatic); requires --scroll")
    parser.add_argument("--input-hz", type=int,
                        help="timer-driven scroll input rate, 100..=2000 (default 1000); requires --scroll")
    parser.add_argument("--web", action="store_true",
                        help="profile the WASM Gallery in headless Chromium via agent-browser")
    parser.add_argument("--bench", action="store_true",
                        help="run the standalone engine benchmark (startup x3, scroll with vsync off and on, "
                             "idle, forced redraw) instead of the Gallery and judge every target")
    parser.add_argument("--bench-content", choices=("material", "simple"), default="material",
                        help="engine benchmark rows: Material components or plain quads and text")
    parser.add_argument("--md3io", metavar="URL",
                        help="run the www/profile.js frame probe on a reference page (e.g. https://md3.io/) "
                             "in Chromium for the WASM comparison")
    parser.add_argument("--summarize", nargs="+", type=Path, metavar="JSON",
                        help="print target verdicts for existing engine_bench reports or Gallery frames.json")
    args = parser.parse_args()
    if args.summarize:
        return summarize(args.summarize)
    if args.output is None:
        parser.error("an output directory is required")
    if args.scroll_input and not args.scroll:
        parser.error("--scroll-input requires --scroll")
    if args.input_hz is not None and (not args.scroll or not 100 <= args.input_hz <= 2000):
        parser.error("--input-hz requires --scroll and must be in 100..=2000")
    if not math.isfinite(args.seconds) or not 0.1 <= args.seconds <= 3600:
        parser.error("--seconds must be finite and in 0.1..=3600")
    if (args.scroll or args.web) and not 2 <= args.seconds <= 120:
        parser.error("--scroll and --web require --seconds in 2..=120")
    destination = args.output.resolve()
    if destination == ROOT or ROOT in destination.parents:
        parser.error("use an output directory outside the repository to keep source fingerprints stable")
    destination.mkdir(parents=True, exist_ok=False)
    if args.md3io:
        result = md3io_probe(args, destination)
        print(f"Evidence: {destination}; runner exit {result}.")
        return result
    if args.bench:
        result = bench_profile(args, destination)
        print(f"Evidence: {destination}; runner exit {result}.")
        return result
    if args.web:
        if args.scroll_input:
            parser.error("--scroll-input is native only; the browser driver uses DOM wheel events")
        result = web_profile(args, destination)
        print(f"Evidence: {destination}; runner exit {result}.")
        return result
    features = "serde,embedded-symbols,frame-profiler"
    if args.backend == "fast":
        features += ",gpui-fast"
    command = ["cargo", "build", "--locked", "--release", "-p", "gpui-kit-material",
               "--example", "gallery", "--no-default-features", "--features", features,
               "--message-format=json-render-diagnostics"]
    provenance = {
        "source_before_build": source_identity(features),
        "build_command": command,
        "requested_backend": args.backend,
        "scroll_input_mode": (args.scroll_input or "programmatic") if args.scroll else None,
        "scroll_input_hz": (args.input_hz or 1000) if args.scroll else None,
        "requested_scenario": "bidirectional-scroll" if args.scroll else "startup",
        "rustc": output("rustc", "-Vv").decode(),
        "cargo": output("cargo", "-V").decode().strip(),
        "build_environment": {key: os.environ.get(key) for key in (
            "RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "RUSTUP_TOOLCHAIN", "CARGO_TARGET_DIR",
            "CARGO_BUILD_TARGET", "CARGO_PROFILE_RELEASE_OPT_LEVEL", "CARGO_PROFILE_RELEASE_DEBUG",
            "CARGO_PROFILE_RELEASE_LTO", "CARGO_PROFILE_RELEASE_CODEGEN_UNITS")},
        "acceptance_eligible": False,
        "external_configuration_unverified": True,
        "execution_constraint": "run with no concurrent build modifying this executable or its sources",
    }

    def save():
        (destination / "build.json").write_text(json.dumps(provenance, indent=2) + "\n")

    save()
    # Build and profile at the caller's ordinary process priority.
    with (destination / "build.log").open("wb") as log, (destination / "build-events.jsonl").open("wb") as events:
        built = subprocess.run(command, cwd=ROOT, stdout=events, stderr=log)
    provenance["build_exit_code"] = built.returncode
    provenance["source_after_build"] = source_identity(features)
    save()
    if built.returncode:
        print(f"Build failed; see {destination / 'build.log'}", file=sys.stderr)
        return 2
    if provenance["source_before_build"] != provenance["source_after_build"]:
        print("Sources changed during build; capture cancelled.", file=sys.stderr)
        return 2
    artifacts = []
    for line in (destination / "build-events.jsonl").read_text().splitlines():
        event = json.loads(line)
        target = event.get("target", {})
        if (event.get("reason") == "compiler-artifact"
                and target.get("name") == "gallery"
                and "example" in target.get("kind", [])
                and Path(target.get("src_path", "")).resolve() == ROOT / "crates/material/examples/gallery.rs"
                and event.get("executable")):
            artifacts.append(event)
    if len(artifacts) != 1:
        provenance["capture_error"] = "build did not report exactly one gallery executable"
        save()
        return 2
    provenance["compiler_artifact"] = artifacts[0]
    binary = Path(artifacts[0]["executable"])
    provenance["binary"] = str(binary)
    provenance["binary_sha256"] = hashlib.sha256(binary.read_bytes()).hexdigest()
    save()
    env = os.environ.copy()
    env.pop("MATERIAL_SCROLL_PROFILE", None)
    env.pop("MATERIAL_SCROLL_INPUT", None)
    env.pop("MATERIAL_SCROLL_INPUT_HZ", None)
    if args.scroll:
        env["MATERIAL_SCROLL_PROFILE"] = "1"
        env["MATERIAL_SCROLL_INPUT"] = args.scroll_input or "programmatic"
        if args.input_hz is not None:
            env["MATERIAL_SCROLL_INPUT_HZ"] = str(args.input_hz)
    env.update(MATERIAL_PROFILE_OUTPUT=str(destination / "frames.json"),
               MATERIAL_PROFILE_SECONDS=str(args.seconds))
    with (destination / "run.log").open("wb") as log:
        try:
            result = subprocess.run([str(binary)], cwd=ROOT, env=env,
                                    stdout=log, stderr=subprocess.STDOUT,
                                    timeout=args.seconds + 60)
            code = result.returncode
        except subprocess.TimeoutExpired:
            code = 2
            provenance["capture_error"] = "capture timed out; process killed, evidence incomplete"
        except OSError as error:
            code = 2
            provenance["capture_error"] = f"could not execute reported artifact: {error}"
    provenance["capture_exit_code"] = code
    provenance["source_after_capture"] = source_identity(features)
    provenance["binary_sha256_after_capture"] = hashlib.sha256(binary.read_bytes()).hexdigest()
    provenance["inputs_stable"] = (
        provenance["source_after_build"] == provenance["source_after_capture"]
        and provenance["binary_sha256"] == provenance["binary_sha256_after_capture"])
    result = runner_exit_code(code, provenance["inputs_stable"], args.scroll,
                              destination / "frames.json")
    try:
        table = gallery_targets(json.loads((destination / "frames.json").read_text()))
        (destination / "targets.json").write_text(json.dumps(table, indent=2) + "\n")
        print_targets(table)
    except (OSError, ValueError, KeyError, TypeError) as error:
        print(f"Could not judge targets: {error}", file=sys.stderr)
    provenance["runner_exit_code"] = result
    save()
    print(f"Evidence: {destination}; capture exit {code}; runner exit {result}.")
    return result


if __name__ == "__main__":
    sys.exit(main())
