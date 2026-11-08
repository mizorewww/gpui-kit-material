#!/usr/bin/env python3
"""Build and record a finite Gallery frame profile with binary provenance.

Natively it runs one of the Gallery's interaction scripts (`interactions.rs`) on a
release build with upstream's `profiler` feature, which times each frame's draw and
submission; in a browser it runs `www/profile.js`. The budgets it judges are
diagnostics (AGENTS.md), never a release gate or a conformance PASS.
"""
import argparse
import hashlib
import json
import math
import os
import re
from pathlib import Path
import subprocess
import sys


ROOT = Path(__file__).resolve().parents[1]


def output(*args):
    return subprocess.check_output(args, cwd=ROOT)


def machine_environment():
    """What a timing depends on besides the code, so a report can say what it was taken
    under. macOS only; anything unread stays null rather than being guessed."""
    def read(*command):
        try:
            return subprocess.run(command, capture_output=True, text=True, timeout=15).stdout
        except (OSError, subprocess.SubprocessError):
            return ""

    environment = {"load_average": list(os.getloadavg()), "low_power_mode": None,
                   "power_source": None, "display_refresh_hz": None, "display": None}
    match = re.search(r"^\s*(?:lowpowermode|powermode)\s+(\d)", read("pmset", "-g"), re.M)
    if match:
        environment["low_power_mode"] = match.group(1) != "0"
    match = re.search(r"drawing from '([^']+)'", read("pmset", "-g", "ps"))
    if match:
        environment["power_source"] = match.group(1)
    try:
        displays = json.loads(read("system_profiler", "SPDisplaysDataType", "-json"))
        gpus = displays["SPDisplaysDataType"]
        screens = [screen for gpu in gpus for screen in gpu.get("spdisplays_ndrvs", [])]
        main = next((s for s in screens if s.get("spdisplays_main") == "spdisplays_yes"), screens[0])
        environment["display"] = main.get("_spdisplays_resolution")
        match = re.search(r"@\s*([\d.]+)\s*Hz", environment["display"] or "")
        if match:
            environment["display_refresh_hz"] = float(match.group(1))
    except (ValueError, KeyError, IndexError):
        pass
    return environment


def environment_label(environment):
    """One line for a report, leading with what most changes the numbers."""
    power = {True: "LOW POWER MODE", False: "normal power", None: "power mode unknown"}
    refresh = environment["display_refresh_hz"]
    load = environment["load_average"][0]
    return (f"{power[environment['low_power_mode']]}, load {load:.2f}, "
            f"display {f'{refresh:g} Hz' if refresh else 'refresh unknown'}")


def warn_about_environment(environment):
    if environment["low_power_mode"]:
        print("WARNING: Low Power Mode is on (pmset -g lowpowermode): the CPU is capped, and its "
              "timings do not compare with a run without it. Instruction counts still do.",
              file=sys.stderr)


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
               ",".join("gpui-kit-material-gallery/" + feature for feature in features.split(","))]
    metadata = json.loads(output(*command))
    packages = {package["id"]: package for package in metadata["packages"]}
    nodes = {node["id"]: node for node in metadata["resolve"]["nodes"]}
    material_manifest = ROOT / "crates/gallery/Cargo.toml"
    seeds = [key for key, package in packages.items()
             if Path(package["manifest_path"]).resolve() == material_manifest]
    if len(seeds) != 1:
        raise RuntimeError("metadata did not resolve exactly one Gallery package")
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


def web_profile(args, destination):
    """Bounded headless browser run of the WASM Gallery (www/profile.js)."""
    import functools
    import http.server
    import threading
    import time
    import uuid

    build = ["./scripts/build-gallery-web.sh", "--release", "--" + args.backend]
    provenance = {"build_command": build, "requested_backend": args.backend,
                  "requested_scenario": "web-" + args.web_mode,
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
    query = f"profile={args.web_mode}&seconds={args.seconds}"
    if args.input_hz is not None:
        query += f"&hz={args.input_hz}"
    url = f"http://127.0.0.1:{server.server_address[1]}/?{query}"
    browser_args = "--disable-gpu-vsync,--disable-frame-rate-limit,--enable-unsafe-webgpu"
    provenance.update(url=url, browser_args=browser_args, viewport=[1280, 800],
                      environment=machine_environment())
    save()
    print(f"Conditions: {environment_label(provenance['environment'])}")

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
            report["environment"] = provenance["environment"]
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
    "steady_cpu_p95": ("steady-state frame (draw + submission) p95", 1 * MS),
    "steady_cpu_max": ("steady-state frame (draw + submission) max", 2 * MS),
    "material_list_vs_kit": ("Material list instructions per frame / equivalent Kit list, worst of p50 and p95", 2.0),
    "first_frame_vs_kit": ("first frame (draw + submission) / Kit's stories baseline on this machine", 1.2),
    "page_switch": ("switching Gallery pages (first frame after the switch)", 8 * MS),
}
# WASM: the steady-state budgets x2 (AGENTS.md); www/profile.js judges them.
# The upstream profiler times draw and submission, not the frame callback around them,
# drawable waits outside the submission or the GPU: none of those is reported.


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


def verdict(name, measured, *, note=None):
    label, limit = TARGETS[name]
    if measured is None:
        result = "UNVERIFIED"
    else:
        result = "PASS" if measured <= limit else "FAIL"
    entry = {"target": label, "limit": limit, "measured": measured, "verdict": result}
    if note:
        entry["note"] = note
    return entry


def first_frame_verdict(first_frame_ns, kit_baseline_ns, note):
    """AGENTS.md: at most 20% above Kit's stories baseline on the same machine."""
    if first_frame_ns is None or not kit_baseline_ns:
        missing = "no first frame measured" if first_frame_ns is None \
            else "pass --kit-first-frame-ms, measured on this machine"
        return verdict("first_frame_vs_kit", None, note=f"{note}; {missing}")
    return verdict("first_frame_vs_kit", first_frame_ns / kit_baseline_ns,
                   note=f"{note}; {first_frame_ns} ns over a {kit_baseline_ns} ns baseline")


def material_list_verdict(reports):
    """AGENTS.md: a Material list costs at most 2x the instructions per frame of
    an equivalent Kit list. component_bench reports of both, same mode and backend."""
    def instructions(report):
        column = report["columns"].index("instructions")
        return [frame[column] for frame in report["frames"]]

    material = reports.get("material-list")
    kit = reports.get("kit-list")
    if not material or not kit:
        return {}, verdict("material_list_vs_kit", None, note="needs component_bench reports of both lists")
    problems = []
    if material["mode"] != kit["mode"]:
        problems.append(f"modes differ ({material['mode']}, {kit['mode']})")
    if material["metadata"].get("backend") != kit["metadata"].get("backend"):
        problems.append("backends differ")
    if material.get("viewport") != kit.get("viewport") or material.get("scale") != kit.get("scale"):
        problems.append("viewports differ")
    if material["mode"] == "scroll" and not (material.get("scroll_steps_that_moved") and kit.get("scroll_steps_that_moved")):
        problems.append("a list did not scroll")
    ours, theirs = distribution(instructions(material)), distribution(instructions(kit))
    metrics = {"mode": material["mode"], "material_instructions": ours, "kit_instructions": theirs}
    if not theirs["p50_ns"] or not theirs["p95_ns"]:
        problems.append("no instruction counts (they read 0 outside native macOS)")
    if problems:
        return metrics, verdict("material_list_vs_kit", None, note="; ".join(problems))
    ratios = {q: ours[f"{q}_ns"] / theirs[f"{q}_ns"] for q in ("p50", "p95", "max")}
    metrics["ratios"] = ratios
    return metrics, verdict("material_list_vs_kit", max(ratios["p50"], ratios["p95"]),
                            note=f"{material['mode']}; max-frame ratio {ratios['max']:.2f}")


def gallery_targets(data, *, kit_first_frame_ns=None):
    """Target verdicts from a native Gallery interaction profile (gallery-interactions-v2).

    The first frame after each page switch is that switch's cost, not a steady-state
    frame; the initial draw, in `open_window`, starts the first frame."""
    frames = data.get("frames", [])
    out = {"source": "native Gallery interaction profile", "metrics": {}, "targets": {}}
    if not frames:
        return out
    switch_frames = set()
    for label, at in data.get("marks_ns", []):
        if label.startswith("switch:"):
            first = next((i for i, f in enumerate(frames) if f["start_ns"] >= at), None)
            if first is not None:
                switch_frames.add(first)
    steady = [f["frame_ns"] for i, f in enumerate(frames)
              if not f.get("initial") and i not in switch_frames]
    m = out["metrics"]
    m["steady_frame"] = distribution(steady)
    m["by_step"] = {}
    for f in frames:
        kind = f["step"].split(":")[0]
        m["by_step"].setdefault(kind, []).append(f["frame_ns"])
    m["by_step"] = {kind: distribution(values) for kind, values in m["by_step"].items()}
    initial = frames[0]["frame_ns"] if frames[0].get("initial") else None
    presented = next((f for f in frames[1:] if f.get("present_ns") is not None), None)
    m["first_frame_ns"] = initial + presented["frame_ns"] if initial and presented else None
    out["targets"]["steady_cpu_p95"] = verdict("steady_cpu_p95", m["steady_frame"]["p95_ns"])
    out["targets"]["steady_cpu_max"] = verdict("steady_cpu_max", m["steady_frame"]["max_ns"])
    out["targets"]["first_frame_vs_kit"] = first_frame_verdict(
        m["first_frame_ns"], kit_first_frame_ns,
        "the initial draw plus the first submitted frame; window creation is not included")
    switches = data.get("page_switch_ns")
    m["page_switch"] = distribution(switches or [])
    out["targets"]["page_switch"] = verdict(
        "page_switch", max(switches) if switches else None,
        note=None if switches else "the script switched no page")
    return out


def print_targets(table, file=sys.stdout):
    for name, entry in table["targets"].items():
        measured = entry["measured"]
        if measured is None:
            shown = "-"
        elif name.endswith("_vs_kit"):
            shown = f"{measured:.2f}x"
        else:
            shown = f"{measured / MS:.3f} ms"
        print(f"  {entry['verdict']:<10} {name:<30} {shown:>12}  {entry['target']}"
              + (f" [{entry['note']}]" if entry.get("note") else ""), file=file)


def kit_first_frame_ns(args):
    return round(args.kit_first_frame_ms * MS) if args.kit_first_frame_ms else None


def summarize(paths, *, kit_baseline_ns=None):
    """Print target verdicts for existing Gallery interaction profiles, or for
    component_bench reports of the Material and Kit lists."""
    lists = {}
    code = 2
    for path in paths:
        data = json.loads(Path(path).read_text())
        if data.get("schema") == "component-bench-v1":
            lists[data["content"]] = data
        elif data.get("schema") == "gallery-interactions-v2":
            table = gallery_targets(data, kit_first_frame_ns=kit_baseline_ns)
            print(f"{path}:")
            print_targets(table)
            code = 1 if any(t["verdict"] == "FAIL" for t in table["targets"].values()) else code
        else:
            print(f"{path}: not a gallery-interactions-v2 or component-bench-v1 report", file=sys.stderr)
    if lists:
        metrics, entry = material_list_verdict(lists)
        table = {"metrics": metrics, "targets": {"material_list_vs_kit": entry}}
        print("component_bench:")
        print_targets(table)
        code = 1 if entry["verdict"] == "FAIL" else code
    return code


def probe_movement_warning(url, moved):
    """The warning a probe result's `moved` calls for, or None. `moved` is true, false, or
    null when the probe could not tell (a canvas page that does not report its draws):
    only false says the page stood still, and null is reported as unknown, not as idle."""
    if moved is False:
        return f"{url}: the wheel moved nothing, so this measured an idle page"
    if moved is None:
        return f"{url}: whether the wheel moved the page is unknown"
    return None


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
                warning = probe_movement_warning(args.md3io, value.get("moved"))
                if warning:
                    print(warning, file=sys.stderr)
                code = 0
                break
        else:
            provenance["error"] = "probe did not finish before the deadline"
    finally:
        browser("close")
        (destination / "probe-provenance.json").write_text(json.dumps(provenance, indent=2) + "\n")
    return code


SCRIPTS = ("interactions", "hover", "switch", "scroll-specs", "scroll-overview")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path, nargs="?",
                        help="new evidence directory outside the repository")
    parser.add_argument("--seconds", type=float, default=10,
                        help="browser runs only: how long to record")
    parser.add_argument("--backend", choices=("upstream", "fast"), default="fast")
    parser.add_argument("--script", choices=SCRIPTS, default="interactions",
                        help="native: the Gallery interaction script to run (interactions.rs)")
    parser.add_argument("--web", action="store_true",
                        help="profile the WASM Gallery in headless Chromium via agent-browser")
    parser.add_argument("--web-mode", choices=("startup", "scroll", "idle"), default="startup",
                        help="what the browser run records (www/profile.js)")
    parser.add_argument("--input-hz", type=int,
                        help="browser scroll input rate, 100..=2000 (default 1000)")
    parser.add_argument("--md3io", metavar="URL",
                        help="run the www/profile.js frame probe on a reference page (e.g. https://md3.io/) "
                             "in Chromium for the WASM comparison")
    parser.add_argument("--summarize", nargs="+", type=Path, metavar="JSON",
                        help="print target verdicts for existing Gallery interaction profiles or "
                             "component_bench reports")
    parser.add_argument("--kit-first-frame-ms", type=float, metavar="MS",
                        help="first frame of Kit's stories on this machine; "
                             "without it the first-frame target is UNVERIFIED")
    args = parser.parse_args()
    if args.summarize:
        return summarize(args.summarize, kit_baseline_ns=kit_first_frame_ns(args))
    if args.output is None:
        parser.error("an output directory is required")
    if args.input_hz is not None and (args.web_mode != "scroll" or not 100 <= args.input_hz <= 2000):
        parser.error("--input-hz requires --web-mode scroll and must be in 100..=2000")
    if (args.web or args.md3io) and not 2 <= args.seconds <= 120:
        parser.error("browser runs need --seconds in 2..=120")
    destination = args.output.resolve()
    if destination == ROOT or ROOT in destination.parents:
        parser.error("use an output directory outside the repository to keep source fingerprints stable")
    destination.mkdir(parents=True, exist_ok=False)
    warn_about_environment(machine_environment())
    if args.md3io:
        result = md3io_probe(args, destination)
        print(f"Evidence: {destination}; runner exit {result}.")
        return result
    if args.web:
        result = web_profile(args, destination)
        print(f"Evidence: {destination}; runner exit {result}.")
        return result
    features = "embedded-symbols,frame-profiler"
    if args.backend == "fast":
        features += ",gpui-fast"
    command = ["cargo", "build", "--locked", "--release", "-p", "gpui-kit-material-gallery",
               "--bin", "gallery", "--no-default-features", "--features", features,
               "--message-format=json-render-diagnostics"]
    provenance = {
        "source_before_build": source_identity(features),
        "build_command": command,
        "requested_backend": args.backend,
        "requested_script": args.script,
        "rustc": output("rustc", "-Vv").decode(),
        "cargo": output("cargo", "-V").decode().strip(),
        "build_environment": {key: os.environ.get(key) for key in (
            "RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "RUSTUP_TOOLCHAIN", "CARGO_TARGET_DIR",
            "CARGO_BUILD_TARGET", "CARGO_PROFILE_RELEASE_OPT_LEVEL", "CARGO_PROFILE_RELEASE_DEBUG",
            "CARGO_PROFILE_RELEASE_LTO", "CARGO_PROFILE_RELEASE_CODEGEN_UNITS")},
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
                and "bin" in target.get("kind", [])
                and Path(target.get("src_path", "")).resolve() == ROOT / "crates/gallery/src/main.rs"
                and event.get("executable")):
            artifacts.append(event)
    if len(artifacts) != 1:
        provenance["capture_error"] = "build did not report exactly one gallery executable"
        save()
        return 2
    binary = Path(artifacts[0]["executable"])
    provenance["binary"] = str(binary)
    provenance["binary_sha256"] = hashlib.sha256(binary.read_bytes()).hexdigest()
    provenance["environment"] = machine_environment()
    save()
    env = os.environ.copy()
    env.update(MATERIAL_PROFILE_SCRIPT=args.script,
               MATERIAL_PROFILE_OUTPUT=str(destination / "frames.json"))
    with (destination / "run.log").open("wb") as log:
        try:
            code = subprocess.run([str(binary)], cwd=ROOT, env=env, stdout=log,
                                  stderr=subprocess.STDOUT, timeout=180).returncode
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
    result = 2
    try:
        frames = json.loads((destination / "frames.json").read_text())
        if not any(f.get("present_ns") is not None for f in frames.get("frames", [])):
            provenance["capture_error"] = (
                "no frame was submitted: macOS gives a covered window no frames, and a remote "
                "desktop's overlay covers every window; run on a local display with the window in front")
            print(provenance["capture_error"], file=sys.stderr)
            code = 2
        table = gallery_targets(frames, kit_first_frame_ns=kit_first_frame_ns(args))
        table["environment"] = provenance["environment"]
        (destination / "targets.json").write_text(json.dumps(table, indent=2) + "\n")
        print(f"Conditions: {environment_label(table['environment'])}")
        print_targets(table)
        if code == 0 and provenance["inputs_stable"]:
            failed = any(t["verdict"] == "FAIL" for t in table["targets"].values())
            result = 1 if failed else 0
    except (OSError, ValueError, KeyError, TypeError) as error:
        print(f"Could not judge targets: {error}", file=sys.stderr)
    provenance["runner_exit_code"] = result
    save()
    print(f"Evidence: {destination}; capture exit {code}; runner exit {result}.")
    return result


if __name__ == "__main__":
    sys.exit(main())
