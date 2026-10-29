#!/usr/bin/env python3
"""Capture pinned reference circular button geometry, real pointer states, and content.

Usage: python3 scripts/capture-button-circular.py /path/to/material output.json
Captures actual CSS :active and selected/disabled states plus icon/text/loading. Requires git, bun, npx, agent-browser.
Uses original createButton and complete main.scss compiled with Sass 1.85.1; no reference source edits.
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
    # All imports of the actual button and progress must match the pinned revision.
    capture.require_reference(parser, source, "src", "Reference HEAD must be ",
                              "Reference src (including styles and progress) must be clean")
    sampler = capture.sampler(__file__)
    data = {"reference": REVISION, "tools": capture.tool_versions()}
    with tempfile.TemporaryDirectory(prefix="material-button-circular-") as directory:
        work = Path(directory)
        capture.bundle(work, "import createButton from " + capture.module(source / "src/components/button/button.ts") + ";\n"
                       "window.referenceCreateButton=createButton;\n")
        capture.compile_sass(source / "src/styles/main.scss", work / "buttons.css")
        data["stylesheet_sha256"] = hashlib.sha256((work / "buttons.css").read_bytes()).hexdigest()
        (work / "index.html").write_text('<!doctype html><meta charset="utf-8"><link rel="stylesheet" href="buttons.css"><script src="entry.js"></script><body></body>')
        with capture.serve(directory) as port:
            with capture.browser_session("material-button-circular-") as browser:
                evaluate = browser.evaluate
                browser("open", f"http://127.0.0.1:{port}")
                browser("set", "media", "light")
                evaluate(sampler)
                data["media"] = evaluate("({dark:matchMedia('(prefers-color-scheme: dark)').matches,reduced_motion:matchMedia('(prefers-reduced-motion: reduce)').matches})")
                data["ua"] = evaluate("navigator.userAgent")
                data["states"] = []
                for size in ("xs", "s", "m", "l", "xl"):
                    for shape in ("round", "square"):
                        config = {"size": size, "shape": shape}
                        browser("mouse", "move", "500", "400")
                        evaluate("mountCircular(" + json.dumps(config) + ")")
                        row = {**config, "default": evaluate("snapshotCircular()")}
                        browser("mouse", "move", "40", "35")
                        browser("mouse", "down", "left")
                        row["pressed"] = evaluate("snapshotCircular()")
                        browser("mouse", "up", "left")
                        evaluate("circularButton.disable()")
                        row["disabled"] = evaluate("snapshotCircular()")
                        browser("mouse", "move", "500", "400")
                        evaluate("mountCircular(" + json.dumps({**config, "toggle": True, "selected": True}) + ")")
                        row["selected"] = evaluate("snapshotCircular()")
                        browser("mouse", "move", "40", "35")
                        browser("mouse", "down", "left")
                        row["selected_pressed"] = evaluate("snapshotCircular()")
                        browser("mouse", "up", "left")
                        evaluate("circularButton.disable()")
                        row["selected_disabled"] = evaluate("snapshotCircular()")
                        for state in ("pressed", "selected_pressed"):
                            if not row[state]["active"]:
                                raise ValueError("Real pointer press did not activate " + str(config))
                        data["states"].append(row)
                browser("mouse", "move", "500", "400")
                data.update(evaluate("circularContentMatrix()"))
                heights = {"xs": 32, "s": 40, "m": 56, "l": 96, "xl": 136}
                selected_round = {"xs": 12, "s": 12, "m": 16, "l": 28, "xl": 28}
                pressed_round = {"xs": 8, "s": 8, "m": 12, "l": 16, "xl": 16}
                for row in data["states"]:
                    radius = "50%" if row["shape"] == "round" else "12px"
                    for state in ("default", "pressed", "disabled"):
                        if row[state]["css"]["radius"] != [radius] * 4:
                            raise ValueError("Circular resting/pressed radius changed")
                    selected_radius = selected_round[row["size"]] if row["shape"] == "round" else heights[row["size"]] / 2
                    selected_radius = f"{selected_radius:g}px"
                    for state in ("selected", "selected_disabled"):
                        if row[state]["css"]["radius"] != [selected_radius] * 4:
                            raise ValueError("Circular selected radius changed")
                    pressed_radius = pressed_round[row["size"]] if row["shape"] == "round" else 12
                    if row["selected_pressed"]["css"]["radius"] != [f"{pressed_radius}px"] * 4:
                        raise ValueError("Circular selected press radius changed")
                    for state in ("default", "pressed", "disabled", "selected", "selected_pressed", "selected_disabled"):
                        sample = row[state]
                        if sample["rect"]["width"] != 40 or sample["rect"]["height"] != heights[row["size"]]:
                            raise ValueError("Circular width/height cascade changed")
                for row in data["rows"]:
                    for state in ("before", "progress", "loading", "restored"):
                        sample = row[state]
                        if sample["rect"]["width"] != 40 or sample["rect"]["height"] != heights[row["size"]]:
                            raise ValueError("Circular content changed layout dimensions")
                        if sample["css"]["padding"] != ["0px"] * 4:
                            raise ValueError("Circular size padding changed")
                        if sample["text"] and sample["text"]["display"] != "none":
                            raise ValueError("Circular text must be hidden")
                    if not row["loading"]["disabled"] or row["loading"]["busy"] != "true":
                        raise ValueError("Loading semantics changed")
                if data["api"]["inline_width"]["rect"]["width"] != 40 or data["api"]["inline_important_width"]["rect"]["width"] != 200:
                    raise ValueError("Circular width importance changed")
            capture.write_json(args.output, data)


if __name__ == "__main__":
    main()
