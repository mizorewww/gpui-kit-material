#!/usr/bin/env python3
"""Record the vendored gpui-fast packages against their upstream source.

With --published, the source is the crates.io archive of that release: the script
downloads each vendored crate's archive, checks it against the index checksum, and
compares. With --rev, the source is an unreleased upstream commit: gpui-fast's
repository names its crates `gpui`, `gpui_web`, ...; only its release script
(`script/prepare-release.ts`) gives them their `gpui-fast-*` names and registry
dependencies, so a Git `[patch]` cannot select them. The script runs that release step
at the commit (as --version) and packages the result as crates.io would receive it;
at a release tag it reproduces the published archives, apart from their Cargo.lock.

Either way it writes each vendored crate's SOURCE.json with --write: the upstream file
hashes and every file that differs (MATERIAL-PATCHES.md says why).

    scripts/vendor-gpui-fast.py --published <version> --work <empty scratch dir> [--write]
    scripts/vendor-gpui-fast.py --rev <commit> --version <version> --work <dir> [--write]

Without --write it only reports the difference. Needs Python 3.12+ (tarfile's
extraction filter); --rev also needs git, bun and cargo. It compiles nothing.
"""
import argparse
import hashlib
import json
import subprocess
import tarfile
import urllib.request
from pathlib import Path

REPOSITORY = "https://github.com/longbridge/gpui-fast"
# The crates that still carry local changes (MATERIAL-PATCHES.md); the others come from
# crates.io.
CRATES = ["gpui-fast", "gpui-fast-apple"]
USER_AGENT = "gpui-kit-material vendor-gpui-fast.py (github.com/mizorewww/gpui-kit-material)"
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


def fetch(url: str) -> bytes:
    request = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    with urllib.request.urlopen(request) as response:
        return response.read()


def published(version: str, work: Path) -> tuple[Path, dict[str, dict[str, str]]]:
    """Downloads and extracts the vendored crates' archives at `version`."""
    upstream = work / "upstream"
    origins = {}
    for name in CRATES:
        metadata = json.loads(fetch(f"https://crates.io/api/v1/crates/{name}/{version}"))
        expected = metadata["version"]["checksum"]
        url = f"https://static.crates.io/crates/{name}/{name}-{version}.crate"
        archive = work / f"{name}-{version}.crate"
        archive.write_bytes(fetch(url))
        actual = sha256(archive)
        if actual != expected:
            raise SystemExit(f"{name} {version}: archive {actual} is not the index's {expected}")
        with tarfile.open(archive) as crate:
            crate.extractall(upstream, filter="data")
        origins[name] = {"archive_url": url, "archive_sha256": actual}
        # gpui-fast's release archives ship no .cargo_vcs_info.json; record one if they do.
        vcs = upstream / f"{name}-{version}" / ".cargo_vcs_info.json"
        if vcs.exists():
            origins[name]["revision"] = json.loads(vcs.read_text())["git"]["sha1"]
    return upstream, origins


def from_commit(rev: str, version: str, work: Path) -> tuple[Path, str]:
    """Packages upstream at `rev` with its own release step, named `version`."""
    checkout = work / "gpui-fast"
    if not checkout.exists():
        run("git", "clone", "--quiet", REPOSITORY, str(checkout), cwd=work)
    run("git", "checkout", "--quiet", rev, cwd=checkout)
    rev = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=checkout, text=True).strip()
    run("bun", "script/prepare-release.ts", "--tag", "v" + version, cwd=checkout)
    release = checkout / "target/release-workspace"
    run("cargo", "package", "--workspace", "--no-verify", "--allow-dirty", "--quiet", cwd=release)
    upstream = work / "upstream"
    for archive in sorted((release / "target/package").glob("*.crate")):
        with tarfile.open(archive) as crate:
            crate.extractall(upstream, filter="data")
    return upstream, rev


def main() -> None:
    parser = argparse.ArgumentParser()
    source = parser.add_mutually_exclusive_group(required=True)
    source.add_argument("--published", metavar="VERSION")
    source.add_argument("--rev")
    parser.add_argument("--version", help="the version --rev is packaged as")
    parser.add_argument("--work", required=True, type=Path)
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    vendor = Path(__file__).resolve().parent.parent / "vendor"
    work = args.work.resolve()
    work.mkdir(parents=True, exist_ok=True)
    if args.published:
        version = args.published
        upstream, origins = published(version, work)
    else:
        if not args.version:
            parser.error("--rev needs --version")
        version = args.version
        upstream, rev = from_commit(args.rev, version, work)
        origins = {name: {"revision": rev} for name in CRATES}

    for name in CRATES:
        original = files(upstream / f"{name}-{version}", IGNORED)
        local = files(vendor / name, IGNORED | PROVENANCE)
        modified = {path: {"upstream_sha256": original[path], "local_sha256": local[path]}
                    for path in original if path in local and original[path] != local[path]}
        added = {path: local[path] for path in local if path not in original}
        removed = {path: original[path] for path in original if path not in local}
        print(f"{name}: {len(modified)} modified, {len(added)} added, {len(removed)} removed")
        for path in [*modified, *added, *removed]:
            print(f"  {path}")
        if args.write:
            origin = origins[name]
            if args.published:
                described = (f"the published crates.io {version} archive, its checksum checked "
                             "against the index, by scripts/vendor-gpui-fast.py")
            else:
                described = ("upstream at a commit, packaged by its own release step "
                             f"(bun script/prepare-release.ts --tag v{version}; "
                             "cargo package --workspace --no-verify) by scripts/vendor-gpui-fast.py")
            record = {
                "package": name,
                "version": version,
                "repository": REPOSITORY,
                **origin,
                "source": described,
                "upstream_files_sha256": original,
                "modified_files": modified,
                "added_files": added,
                "removed_files": removed,
                "not_vendored": sorted(IGNORED - {".cargo-ok"}),
                "reasons": "MATERIAL-PATCHES.md",
            }
            (vendor / name / "SOURCE.json").write_text(json.dumps(record, indent=2) + "\n")


if __name__ == "__main__":
    main()
