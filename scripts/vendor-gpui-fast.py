#!/usr/bin/env python3
"""Rebuild the gpui-fast packages at an upstream commit and record vendor/ against them.

gpui-fast's repository names its crates `gpui`, `gpui_web`, ...; only its release
script (`script/prepare-release.ts`) gives them their `gpui-fast-*` names and
registry dependencies, so a Git `[patch]` cannot select them. This script runs that
release step at `--rev`, packages the result as crates.io would receive it, and writes
each vendored crate's SOURCE.json: the upstream file hashes and every file that
differs (MATERIAL-PATCHES.md says why). At v0.1.13 it reproduces the published
archives exactly, apart from their package-level Cargo.lock.

    scripts/vendor-gpui-fast.py --rev <commit> --work <empty scratch dir> [--write]

Without --write it only reports the difference. Needs git, bun and cargo; it downloads
the repository and the crates.io index, and compiles nothing.
"""
import argparse
import hashlib
import json
import subprocess
import tarfile
from pathlib import Path

REPOSITORY = "https://github.com/longbridge/gpui-fast"
# The registry version the vendored packages keep, so Kit's `0.1.13` requirements and
# the unvendored gpui-fast-macros/-platform 0.1.13 (unchanged on main) still match.
VERSION = "0.1.13"
CRATES = ["gpui-fast", "gpui-fast-apple", "gpui-fast-linux", "gpui-fast-macos",
          "gpui-fast-web", "gpui-fast-wgpu", "gpui-fast-windows"]
UNVENDORED = ["gpui-fast-macros", "gpui-fast-platform"]
# Package-level lockfiles are not vendored; the workspace Cargo.lock owns resolution.
IGNORED = {".cargo_vcs_info.json", ".cargo-ok", "Cargo.lock"}
PROVENANCE = {"SOURCE.json", "MATERIAL-PATCHES.md"}


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def files(root: Path, skip: set[str]) -> dict[str, str]:
    return {str(path.relative_to(root)): sha256(path) for path in sorted(root.rglob("*"))
            if path.is_file() and path.name not in skip and str(path.relative_to(root)) not in skip}


def run(*args: str, cwd: Path) -> None:
    subprocess.run(args, cwd=cwd, check=True)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--rev", required=True)
    parser.add_argument("--work", required=True, type=Path)
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    vendor = Path(__file__).resolve().parent.parent / "vendor"
    work = args.work.resolve()
    checkout = work / "gpui-fast"
    if not checkout.exists():
        run("git", "clone", "--quiet", REPOSITORY, str(checkout), cwd=work)
    run("git", "checkout", "--quiet", args.rev, cwd=checkout)
    rev = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=checkout, text=True).strip()
    run("bun", "script/prepare-release.ts", "--tag", "v" + VERSION, cwd=checkout)
    release = checkout / "target/release-workspace"
    run("cargo", "package", "--workspace", "--no-verify", "--allow-dirty", "--quiet", cwd=release)
    upstream = work / "upstream"
    for archive in sorted((release / "target/package").glob("*.crate")):
        with tarfile.open(archive) as crate:
            crate.extractall(upstream, filter="data")

    registry = Path.home() / ".cargo/registry/src"
    for name in UNVENDORED:
        published = next(registry.glob(f"*/{name}-{VERSION}"), None)
        if published is None:
            print(f"{name}: no published {VERSION} in the registry cache to compare")
            continue
        same = files(published, IGNORED) == files(upstream / f"{name}-{VERSION}", IGNORED)
        print(f"{name}: {'equals' if same else 'DIFFERS FROM'} the published {VERSION} at {rev[:8]}")

    for name in CRATES:
        original = files(upstream / f"{name}-{VERSION}", IGNORED)
        local = files(vendor / name, IGNORED | PROVENANCE)
        modified = {path: {"upstream_sha256": original[path], "local_sha256": local[path]}
                    for path in original if path in local and original[path] != local[path]}
        added = {path: local[path] for path in local if path not in original}
        removed = {path: original[path] for path in original if path not in local}
        print(f"{name}: {len(modified)} modified, {len(added)} added, {len(removed)} removed")
        for path in [*modified, *added, *removed]:
            print(f"  {path}")
        if args.write:
            source = {
                "package": name,
                "version": VERSION,
                "repository": REPOSITORY,
                "revision": rev,
                "source": "upstream main, packaged by its own release step "
                          "(bun script/prepare-release.ts --tag v" + VERSION + "; "
                          "cargo package --workspace --no-verify) by scripts/vendor-gpui-fast.py",
                "upstream_files_sha256": original,
                "modified_files": modified,
                "added_files": added,
                "removed_files": removed,
                "not_vendored": sorted(IGNORED - {".cargo-ok"}),
                "reasons": "MATERIAL-PATCHES.md",
            }
            (vendor / name / "SOURCE.json").write_text(json.dumps(source, indent=2) + "\n")


if __name__ == "__main__":
    main()
