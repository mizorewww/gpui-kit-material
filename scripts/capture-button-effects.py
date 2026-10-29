#!/usr/bin/env python3
"""Capture pinned md3 button color transitions at exact browser animation times.

Usage: python3 scripts/capture-button-effects.py /path/to/floor-material output.json
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
    session = "material-button-effects-" + uuid.uuid4().hex[:12]
    sampler = Path(__file__).with_suffix(".js").read_text()
    with tempfile.TemporaryDirectory(prefix="material-button-effects-") as directory:
        work = Path(directory)
        run("npx", "--yes", "sass@1.85.1", "--no-source-map", "--style=expanded",
            str(source / "src/styles/main.scss"), str(work / "buttons.css"))
        (work / "index.html").write_text('<!doctype html><html><head><meta charset="utf-8"></head><body></body></html>')
        handler = functools.partial(http.server.SimpleHTTPRequestHandler, directory=directory)
        server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), handler)
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        try:
            run("agent-browser", "--session", session, "open", f"http://127.0.0.1:{server.server_port}")
            result = run("agent-browser", "--session", session, "eval", "--stdin", input=sampler)
            data = json.loads(result)
            if data.get("reference") != REVISION:
                raise ValueError("Sampler reference revision does not match the checkout")
            for state, time in (("variant", 125), ("disabled", 125), ("reversed", 25)):
                if not {"background-color", "color"}.issubset(data[state]):
                    raise ValueError("Incomplete button transition capture: " + state)
                for property_name in ("background-color", "color"):
                    if data[state][property_name]["time"] != time:
                        raise ValueError("Incorrect animation sample time: " + state)
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
