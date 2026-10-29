#!/usr/bin/env python3
"""Capture pinned reference button loading DOM, geometry, and circular motion.

Usage: python3 scripts/capture-button-loading.py /path/to/material output.json
Captures ordinary and reduced motion. Requires git, bun, npx, agent-browser.
Uses original createButton/circularIndeterminateFrame and Sass 1.85.1; no reference source edits.
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
        parser.error("Reference HEAD must be " + REVISION)
    # All imports of the actual button and progress must match the pinned revision.
    if run("git", "-C", str(source), "status", "--porcelain", "--", "src"):
        parser.error("Reference src (including styles and progress) must be clean")
    sampler = Path(__file__).with_suffix(".js").read_text()
    data = {"reference": REVISION, "tools": {
        "bun": run("bun", "--version").strip(),
        "sass": run("npx", "--yes", "sass@1.85.1", "--version").strip(),
        "agent_browser": run("agent-browser", "--version").strip(),
    }, "modes": {}}
    with tempfile.TemporaryDirectory(prefix="material-button-loading-") as directory:
        work = Path(directory)
        entry = ("import createButton from " + json.dumps(str(source / "src/components/button/button.ts")) + ";\n"
                 "import {circularIndeterminateFrame} from " + json.dumps(str(source / "src/components/progress/features/motion.ts")) + ";\n"
                 "window.referenceCreateButton=createButton;window.referenceCircularFrame=circularIndeterminateFrame;\n")
        (work / "entry.ts").write_text(entry)
        run("bun", "build", str(work / "entry.ts"), "--outfile", str(work / "entry.js"))
        run("npx", "--yes", "sass@1.85.1", "--no-source-map", "--style=expanded",
            str(source / "src/styles/main.scss"), str(work / "buttons.css"))
        (work / "index.html").write_text('<!doctype html><meta charset="utf-8"><link rel="stylesheet" href="buttons.css"><script src="entry.js"></script><body></body>')
        handler = functools.partial(http.server.SimpleHTTPRequestHandler, directory=directory)
        server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), handler)
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        try:
            for mode in ("ordinary", "reduced"):
                session = "material-button-loading-" + uuid.uuid4().hex[:12]
                def browser(*commands):
                    return run("agent-browser", "--session", session, *commands)
                def evaluate(code):
                    return json.loads(run("agent-browser", "--session", session, "eval", "--stdin", input=code))
                try:
                    browser("open", f"http://127.0.0.1:{server.server_port}")
                    if mode == "reduced":
                        browser("set", "media", "light", "reduced-motion")
                    browser("mouse", "move", "900", "700")
                    captured = evaluate(sampler)
                    if captured["reduced_motion"] != (mode == "reduced"):
                        raise ValueError("Motion preference mismatch")
                    for row in captured["rows"]:
                        if not row["loading"]["disabled"] or row["loading"]["busy"] != "true":
                            raise ValueError("Loading state mismatch")
                        if row["shown"]["disabled"] or row["shown"]["busy"] is not None:
                            raise ValueError("showProgress must not disable or set busy")
                        if row["loading"]["canvas_style"] != ["24px", "24px"]:
                            raise ValueError("Default circular size clamp changed")
                        if row["restored"]["label"] != "Save changes" or row["restored"]["disabled"]:
                            raise ValueError("Loading restore changed")
                    if captured["after_initially_disabled"]["disabled"]:
                        raise ValueError("Reference no longer enables initially disabled buttons")
                    for row in captured["rows"]:
                        expected_slot = {"xs":20,"s":20,"m":24,"l":32,"xl":40}[row["size"]]
                        expected_gap = {"xs":4,"s":8,"m":8,"l":12,"xl":16}[row["size"]]
                        delta = row["shown"]["rect"]["width"] - row["before"]["rect"]["width"]
                        if abs(delta - (expected_slot + expected_gap if row["placement"] == "none" else 0)) > .02:
                            raise ValueError("Progress slot width changed")
                        if row["loading"]["canvas"]["width"] != min(24, expected_slot) or row["loading"]["canvas"]["height"] != 24:
                            raise ValueError("Progress canvas geometry changed")
                        if row["loading"]["progress_opacity"] != "0.7" or row["repeated"]["label"] != "Saving changes":
                            raise ValueError("Progress opacity or repeated loading semantics changed")
                    data["modes"][mode] = captured
                finally:
                    browser("close")
            args.output.parent.mkdir(parents=True, exist_ok=True)
            args.output.write_text(json.dumps(data, indent=2) + "\n")
        finally:
            server.shutdown()
            server.server_close()
            thread.join()


if __name__ == "__main__":
    main()
