#!/usr/bin/env python3
"""Capture pinned reference divider layout and setter states.

Usage: python3 scripts/capture-divider.py /path/to/material output.json
Captures actual createDivider across orientations, variants, directions, parent layouts and contexts. Requires git, bun, npx, agent-browser.
Uses original createDivider and complete main.scss compiled with Sass 1.85.1; no reference source edits.
"""
import hashlib
import json
from pathlib import Path
import tempfile

import reference_capture as capture
from reference_capture import REVISION


def main():
    parser = capture.parser(__doc__)
    args = parser.parse_args()
    source = args.source.resolve()
    # All imports of the actual divider must match the pinned revision.
    capture.require_reference(parser, source, "src", "Reference HEAD must be ",
                              "Reference src (including styles) must be clean")
    sampler = capture.sampler(__file__)
    data = {"reference": REVISION, "tools": capture.tool_versions()}
    with tempfile.TemporaryDirectory(prefix="material-divider-") as directory:
        work = Path(directory)
        capture.bundle(work, "import { createDivider } from " + capture.module(source / "src/components/divider/divider.ts") + ";\n"
                       "window.referenceCreateDivider=createDivider;\n")
        capture.compile_sass(source / "src/styles/main.scss", work / "divider.css")
        data["stylesheet_sha256"] = hashlib.sha256((work / "divider.css").read_bytes()).hexdigest()
        (work / "index.html").write_text('<!doctype html><meta charset="utf-8"><link rel="stylesheet" href="divider.css"><script src="entry.js"></script><body></body>')
        with capture.serve(directory) as port:
            with capture.browser_session("material-divider-") as browser:
                evaluate = browser.evaluate
                browser("open", f"http://127.0.0.1:{port}")
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
            capture.write_json(args.output, data)


if __name__ == "__main__":
    main()
