#!/usr/bin/env python3
"""Capture the pinned md3 Sass themes using a real browser's CSS cascade.

Usage: python3 scripts/capture-reference-colors.py /path/to/floor-material output.json
Requires npx, git, and agent-browser. Dart Sass is pinned to 1.85.1.
"""
import argparse
import functools
import http.server
import json
from pathlib import Path
import subprocess
import tempfile
import threading
import uuid

REVISION = "c910072f3fc5b660276731d8a935d7186f288364"


def run(*args, **kwargs):
    return subprocess.run(args, check=True, text=True, capture_output=True, **kwargs).stdout


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    source = args.source.resolve()
    if run("git", "-C", str(source), "rev-parse", "HEAD").strip() != REVISION:
        parser.error("The reference checkout must be at " + REVISION)
    if run("git", "-C", str(source), "status", "--porcelain", "--", "src/styles"):
        parser.error("Reference styles must be clean before capture")
    session = "material-colors-" + uuid.uuid4().hex[:12]
    sampler = Path(__file__).with_suffix(".js").read_text()
    with tempfile.TemporaryDirectory(prefix="material-colors-") as directory:
        work = Path(directory)
        themes = sorted(path for path in (source / "src/styles/themes").glob("_*.scss")
                        if path.stem not in ("_base-theme", "_index"))
        if len(themes) != 20:
            parser.error("Expected the 20 pinned theme source files")
        (work / "reference.scss").write_text("\n".join("@use " + json.dumps(str(path)) + ";" for path in themes))
        run("npx", "--yes", "sass@1.85.1", "--no-source-map", "--style=expanded",
            str(work / "reference.scss"), str(work / "reference.css"))
        (work / "index.html").write_text('<!doctype html><html><head><meta charset="utf-8"><link rel="stylesheet" href="reference.css"></head><body>Reference colors</body></html>')
        handler = functools.partial(http.server.SimpleHTTPRequestHandler, directory=directory)
        server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), handler)
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        try:
            run("agent-browser", "--session", session, "open", f"http://127.0.0.1:{server.server_port}")
            result = run("agent-browser", "--session", session, "eval", "--stdin", input=sampler)
            data = json.loads(result)
            if len(data["rows"]) != 120 or len(data["roles"]) != 52:
                raise ValueError("Incomplete reference capture")
            args.output.parent.mkdir(parents=True, exist_ok=True)
            args.output.write_text(json.dumps(data, indent=2) + "\n")
        finally:
            try:
                run("agent-browser", "--session", session, "close")
            finally:
                server.shutdown()
                server.server_close()
                thread.join()


if __name__ == "__main__":
    main()
