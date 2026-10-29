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


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path, help="new evidence directory outside the repository")
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
    args = parser.parse_args()
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
    provenance["runner_exit_code"] = result
    save()
    print(f"Evidence: {destination}; capture exit {code}; runner exit {result}.")
    return result


if __name__ == "__main__":
    sys.exit(main())
