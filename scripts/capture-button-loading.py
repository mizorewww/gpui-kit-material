#!/usr/bin/env python3
"""Capture pinned reference button loading DOM, geometry, and circular motion.

Usage: python3 scripts/capture-button-loading.py /path/to/material output.json
Captures ordinary and reduced motion. Requires git, bun, npx, agent-browser.
Uses original createButton/circularIndeterminateFrame and Sass 1.85.1; no reference source edits.
"""
from pathlib import Path
import tempfile

import reference_capture as capture
from reference_capture import REVISION


def main():
    parser = capture.parser(__doc__)
    args = parser.parse_args()
    source = args.source.resolve()
    # All imports of the actual button and progress must match the pinned revision.
    capture.require_reference(parser, source, "src", "Reference HEAD must be ",
                              "Reference src (including styles and progress) must be clean")
    sampler = capture.sampler(__file__)
    data = {"reference": REVISION, "tools": capture.tool_versions(), "modes": {}}
    with tempfile.TemporaryDirectory(prefix="material-button-loading-") as directory:
        work = Path(directory)
        capture.bundle(work, "import createButton from " + capture.module(source / "src/components/button/button.ts") + ";\n"
                       "import {circularIndeterminateFrame} from " + capture.module(source / "src/components/progress/features/motion.ts") + ";\n"
                       "window.referenceCreateButton=createButton;window.referenceCircularFrame=circularIndeterminateFrame;\n")
        capture.compile_sass(source / "src/styles/main.scss", work / "buttons.css")
        (work / "index.html").write_text('<!doctype html><meta charset="utf-8"><link rel="stylesheet" href="buttons.css"><script src="entry.js"></script><body></body>')
        with capture.serve(directory) as port:
            for mode in ("ordinary", "reduced"):
                with capture.browser_session("material-button-loading-") as browser:
                    evaluate = browser.evaluate
                    browser("open", f"http://127.0.0.1:{port}")
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
            capture.write_json(args.output, data)


if __name__ == "__main__":
    main()
