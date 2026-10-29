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


def source_identity():
    # Include staged, unstaged and non-ignored untracked inputs without copying
    # their contents into the evidence. Ignored build output is not source.
    files = output("git", "ls-files", "-z", "--cached", "--others", "--exclude-standard")
    digest = hashlib.sha256()
    for name in sorted(set(files.split(b"\0")) - {b""}):
        path = ROOT / os.fsdecode(name)
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
        "commit": output("git", "rev-parse", "HEAD").decode().strip(),
        "status": output("git", "status", "--porcelain").decode(),
        "source_files_sha256": digest.hexdigest(),
        "scope": "tracked and non-ignored untracked files; external configuration is not fingerprinted",
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path, help="new evidence directory outside the repository")
    parser.add_argument("--seconds", type=float, default=10)
    args = parser.parse_args()
    if not math.isfinite(args.seconds) or not 0.1 <= args.seconds <= 3600:
        parser.error("--seconds must be finite and in 0.1..=3600")
    destination = args.output.resolve()
    if destination == ROOT or ROOT in destination.parents:
        parser.error("use an output directory outside the repository to keep source fingerprints stable")
    destination.mkdir(parents=True, exist_ok=False)
    command = ["cargo", "build", "--locked", "--release", "-p", "gpui-kit-material",
               "--example", "gallery", "--features", "frame-profiler", "--jobs", "2",
               "--message-format=json-render-diagnostics"]
    provenance = {
        "source_before_build": source_identity(),
        "build_command": command,
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
    # Lower build priority only; profiling runs at ordinary process priority.
    with (destination / "build.log").open("wb") as log, (destination / "build-events.jsonl").open("wb") as events:
        built = subprocess.run(["nice", "-n", "10", *command], cwd=ROOT, stdout=events, stderr=log)
    provenance["build_exit_code"] = built.returncode
    provenance["source_after_build"] = source_identity()
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
    provenance["source_after_capture"] = source_identity()
    provenance["binary_sha256_after_capture"] = hashlib.sha256(binary.read_bytes()).hexdigest()
    provenance["inputs_stable"] = (
        provenance["source_after_build"] == provenance["source_after_capture"]
        and provenance["binary_sha256"] == provenance["binary_sha256_after_capture"])
    save()
    print(f"Evidence: {destination}; capture exit {code}; whole-gallery acceptance remains unverified.")
    return code if code in (1, 2) and provenance["inputs_stable"] else 2


if __name__ == "__main__":
    sys.exit(main())
