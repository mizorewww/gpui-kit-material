#!/usr/bin/env python3
"""Capture pinned reference divider layout and setter states.

Usage: python3 scripts/capture-divider.py /path/to/material output.json
Captures actual createDivider across orientations, variants, directions, parent layouts and contexts. Requires git, bun, npx, agent-browser.
Uses original createDivider and complete main.scss compiled with Sass 1.85.1; no reference source edits.
"""
import argparse
import functools
import hashlib
import http.server
import json
from pathlib import Path
import subprocess
import tempfile
import threading
import uuid

REVISION = "c910072f3fc5b660276731d8a935d7186f288364"


def run(*args, **kwargs):
    result = subprocess.run(args, text=True, capture_output=True, **kwargs)
    if result.returncode:
        raise RuntimeError(f"Command {args!r} failed: {result.stderr} {result.stdout}")
    return result.stdout


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    source = args.source.resolve()
    if run("git", "-C", str(source), "rev-parse", "HEAD").strip() != REVISION:
        parser.error("Reference HEAD must be " + REVISION)
    # All imports of the actual divider must match the pinned revision.
    if run("git", "-C", str(source), "status", "--porcelain", "--", "src"):
        parser.error("Reference src (including styles) must be clean")
    sampler = Path(__file__).with_suffix(".js").read_text()
    data = {"reference": REVISION, "tools": {
        "bun": run("bun", "--version").strip(),
        "sass": run("npx", "--yes", "sass@1.85.1", "--version").strip(),
        "agent_browser": run("agent-browser", "--version").strip(),
    }}
    with tempfile.TemporaryDirectory(prefix="material-divider-") as directory:
        work = Path(directory)
        entry = ("import { createDivider } from " + json.dumps(str(source / "src/components/divider/divider.ts")) + ";\n"
                 "window.referenceCreateDivider=createDivider;\n")
        (work / "entry.ts").write_text(entry)
        run("bun", "build", str(work / "entry.ts"), "--outfile", str(work / "entry.js"))
        run("npx", "--yes", "sass@1.85.1", "--no-source-map", "--style=expanded",
            str(source / "src/styles/main.scss"), str(work / "divider.css"))
        data["stylesheet_sha256"] = hashlib.sha256((work / "divider.css").read_bytes()).hexdigest()
        (work / "index.html").write_text('<!doctype html><meta charset="utf-8"><link rel="stylesheet" href="divider.css"><script src="entry.js"></script><body></body>')
        handler = functools.partial(http.server.SimpleHTTPRequestHandler, directory=directory)
        server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), handler)
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        try:
            session = "material-divider-" + uuid.uuid4().hex[:12]
            def browser(*commands):
                return run("agent-browser", "--session", session, *commands)
            def evaluate(code):
                return json.loads(browser_input(code))
            def browser_input(code):
                return run("agent-browser", "--session", session, "eval", "--stdin", input=code)
            try:
                browser("open", f"http://127.0.0.1:{server.server_port}")
                browser("set", "media", "light")
                evaluate(sampler)
                data["media"] = evaluate("({dark:matchMedia('(prefers-color-scheme: dark)').matches,reduced_motion:matchMedia('(prefers-reduced-motion: reduce)').matches})")
                data["ua"] = evaluate("navigator.userAgent")
                data["environment"] = evaluate("({viewport:{width:innerWidth,height:innerHeight},dpr:devicePixelRatio,zoom:visualViewport.scale,os:navigator.platform})")
                data.update(evaluate("captureDividerMatrix()"))
                if len(data["rows"]) != 1152 or len(data["custom"]) != 48:
                    raise ValueError("Incomplete divider matrix")
                if len(data["setters"]) != 2 or any(len(row["steps"]) != 13 for row in data["setters"]):
                    raise ValueError("Incomplete setter round trips")
                for row in data["setters"]:
                    for step in row["steps"][1:]:
                        if not step["chainable"] or step["sample"]["animation_count"]:
                            raise ValueError("Setter chaining or animation changed")
                default_row = next(row for row in data["rows"] if row["mode"] == "light" and row["scale"] == 1
                                   and row["direction"] == "ltr" and row["layout"] == "block" and row["context"] == "plain"
                                   and row["orientation"] == "horizontal" and row["variant"] == "full-width")
                if data["defaults"] != default_row["sample"]:
                    raise ValueError("Empty config differs from explicit defaults")
                for row in data["custom"]:
                    if row["sample"]["animation_count"] or row["sample"]["transition_duration"] != "0s":
                        raise ValueError("Custom style introduced a transition")
                for row in data["rows"]:
                    sample = row["sample"]
                    expected_color = "rgb(202, 196, 208)" if row["mode"] == "light" else "rgb(73, 69, 79)"
                    if sample["background"] != expected_color:
                        raise ValueError("Baseline theme mode did not resolve expected outline-variant")
                    if sample["animation_count"] or sample["transition_duration"] != "0s":
                        raise ValueError("Unexpected divider animation")
                    expected_orientation = None if row["orientation"] == "horizontal" else "vertical"
                    if sample["aria_orientation"] != expected_orientation:
                        raise ValueError("Orientation semantics mismatch")
                    cross = "height" if row["orientation"] == "horizontal" else "width"
                    if sample["rect"][cross] != 1:
                        raise ValueError("Default divider thickness changed")
                    if row["context"] == "plain" and row["orientation"] == "horizontal" and not row["layout"].startswith("row"):
                        margin = 0 if row["variant"] == "full-width" else (16 if row["variant"] == "inset" else 32)
                        if sample["rect"]["width"] != 240 - margin:
                            raise ValueError("Logical inset width mismatch")
                browser("set", "media", "light", "reduced-motion")
                reduced = evaluate("matchMedia('(prefers-reduced-motion: reduce)').matches")
                if not reduced:
                    raise ValueError("Reduced-motion emulation failed")
                reduced_matrix = evaluate("captureDividerMatrix()")
                normal_matrix = {key: data[key] for key in ("rows", "custom", "setters", "defaults")}
                if reduced_matrix != normal_matrix:
                    raise ValueError("Divider differs with reduced motion; retain separate evidence before proceeding")
                data["reduced_motion_check"] = {
                    "media_matches": reduced,
                    "identical_to_normal": True,
                    "matrix_sha256": hashlib.sha256(json.dumps(normal_matrix, sort_keys=True, separators=(",", ":")).encode()).hexdigest(),
                }
                data["scope"] = "Pinned-source browser reference only. No implementation conformance verdict. Fixed-size host layouts, baseline light/dark and four root font scales; no raster or accessibility-tree assertion."
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
