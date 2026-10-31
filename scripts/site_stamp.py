#!/usr/bin/env python3
"""What the published site was built from, so that deploying a stale one fails.

scripts/build-gallery-web.sh records its source in the module's directory, and
scripts/build-site.py records its own and the module's beside target/site (not in it,
which would publish the record). scripts/deploy-site.sh runs `verify`.

    site_stamp.py state                               the working tree, as JSON
    site_stamp.py module <dir> <profile> <state>      after building the module
    site_stamp.py verify <site directory>             exits 1 with the reasons
"""
import hashlib
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MODULE_STAMP = "BUILD.json"
# What the module is compiled from. A later commit touching none of these (documentation,
# say) can reuse it through `build-site.sh --skip-wasm`; any other change cannot.
MODULE_INPUTS = ("crates", "vendor", "Cargo.toml", "Cargo.lock", "rust-toolchain.toml",
                 "scripts/build-gallery-web.sh")


def git(*args, check=True):
    return subprocess.run(["git", *args], cwd=ROOT, check=check, capture_output=True, text=True)


def state():
    """The commit, and whether the tree differs from it (untracked files count)."""
    return {"commit": git("rev-parse", "HEAD").stdout.strip(),
            "dirty": bool(git("status", "--porcelain").stdout.strip())}


def settled(before):
    """`before`, marked dirty when the tree moved while something was built from it."""
    return {"commit": before["commit"], "dirty": before["dirty"] or state() != before}


def digest(directory):
    """A hash of every file's path and content."""
    total = hashlib.sha256()
    for path in sorted(p for p in directory.rglob("*") if p.is_file()):
        total.update(str(path.relative_to(directory)).encode() + b"\0")
        total.update(hashlib.sha256(path.read_bytes()).digest())
    return total.hexdigest()


def site_stamp_path(site):
    return site.parent / f"{site.name}.json"


def write_module(directory, profile, before):
    stamp = {**settled(before), "profile": profile}
    (directory / MODULE_STAMP).write_text(json.dumps(stamp, indent=2) + "\n")


def write_site(site, module_directory, before):
    """Called last by build-site.py, over the finished directory."""
    module_stamp = module_directory / MODULE_STAMP
    module = json.loads(module_stamp.read_text()) if module_stamp.exists() else None
    stamp = {**settled(before), "module": module, "digest": digest(site)}
    site_stamp_path(site).write_text(json.dumps(stamp, indent=2) + "\n")


def problems(site):
    """Why this site must not be published, given its stamp and the current checkout."""
    path = site_stamp_path(site)
    if not path.exists():
        return [f"{path} is missing: run ./scripts/build-site.sh"]
    stamp = json.loads(path.read_text())
    now = state()
    found = []
    if now["dirty"]:
        found.append("the working tree has changes; deploy-site.sh publishes deploy/site from it")
    if stamp["commit"] != now["commit"]:
        found.append(f"the site was built from {stamp['commit'][:12]}, and HEAD is {now['commit'][:12]}")
    if stamp["dirty"]:
        found.append("the site was built from a tree with uncommitted changes")
    module = stamp.get("module")
    if not module:
        found.append("the site's module has no build record: run ./scripts/build-site.sh without --skip-wasm")
    else:
        if module["profile"] != "wasm-release":
            found.append(f"the site's module is a {module['profile']} build")
        if module["dirty"]:
            found.append("the site's module was built from a tree with uncommitted changes")
        if module["commit"] != now["commit"]:
            changed = git("diff", "--name-only", module["commit"], now["commit"], "--", *MODULE_INPUTS, check=False)
            if changed.returncode:
                found.append(f"the site's module was built from {module['commit'][:12]}, which this repository cannot compare")
            elif changed.stdout.strip():
                first = changed.stdout.split()[0]
                found.append(f"the site's module was built from {module['commit'][:12]}, and {first} has changed since")
    if stamp.get("digest") != digest(site):
        found.append(f"{site} changed after it was built")
    return found


def main():
    args = sys.argv[1:]
    if args == ["state"]:
        print(json.dumps(state()))
    elif len(args) == 4 and args[0] == "module":
        write_module(Path(args[1]), args[2], json.loads(args[3]))
    elif len(args) == 2 and args[0] == "verify":
        found = problems(Path(args[1]))
        for problem in found:
            print(f"Refusing to deploy: {problem}.", file=sys.stderr)
        return 1 if found else 0
    else:
        print(__doc__, file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main())
