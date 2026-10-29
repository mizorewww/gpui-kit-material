#!/usr/bin/env python3
"""Capture pinned reference controlled active button states.

Usage: python3 scripts/capture-button-active.py /path/to/material output.json
Captures actual setActive API across variants, sizes, shapes, ripple and selection. Requires git, bun, npx, agent-browser.
Uses original createButton and complete main.scss compiled with Sass 1.85.1; no reference source edits.
"""
import hashlib
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
    data = {"reference": REVISION, "tools": capture.tool_versions()}
    with tempfile.TemporaryDirectory(prefix="material-button-active-") as directory:
        work = Path(directory)
        capture.bundle(work, "import createButton from " + capture.module(source / "src/components/button/button.ts") + ";\n"
                       "window.referenceCreateButton=createButton;\n")
        capture.compile_sass(source / "src/styles/main.scss", work / "buttons.css")
        data["stylesheet_sha256"] = hashlib.sha256((work / "buttons.css").read_bytes()).hexdigest()
        (work / "index.html").write_text('<!doctype html><meta charset="utf-8"><link rel="stylesheet" href="buttons.css"><script src="entry.js"></script><body></body>')
        with capture.serve(directory) as port:
            with capture.browser_session("material-button-active-") as browser:
                evaluate = browser.evaluate
                browser("open", f"http://127.0.0.1:{port}")
                browser("set", "media", "light")
                evaluate(sampler)
                data["media"] = evaluate("({dark:matchMedia('(prefers-color-scheme: dark)').matches,reduced_motion:matchMedia('(prefers-reduced-motion: reduce)').matches})")
                data["ua"] = evaluate("navigator.userAgent")
                browser("mouse", "move", "900", "700")
                data["rows"] = evaluate("captureActiveMatrix()")
                if len(data["rows"]) != 300:
                    raise ValueError("Incomplete active matrix")
                for row in data["rows"]:
                    pressed = {"xs":8,"s":8,"m":12,"l":16,"xl":16}[row["size"]]
                    for state in ("active", "disabled_active"):
                        sample = row[state]
                        if not sample["active_class"] or sample["radius"] != f"{pressed}px":
                            raise ValueError("Active shape mismatch")
                        expected_opacity = 0 if row["ripple"] else .1
                        if abs(float(sample["opacity"])-expected_opacity) > 1e-6:
                            raise ValueError("Active layer mismatch")
                        if sample["pointer_active"] or sample["waves"]:
                            raise ValueError("Controlled active incorrectly caused a pointer press")
                    if row["disabled_inactive"]["active_class"] or not row["disabled_inactive"]["disabled"]:
                        raise ValueError("Clearing active changed disabled state")
                    if row["before"]["ripple_container"] != row["ripple"]:
                        raise ValueError("Ripple configuration not applied")
                    if row["active"]["selected"] != row["before"]["selected"]:
                        raise ValueError("Active changed selection")
            capture.write_json(args.output, data)


if __name__ == "__main__":
    main()
