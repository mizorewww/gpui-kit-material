#!/usr/bin/env python3
"""Capture pinned md3 button state-layer and shadow transitions at exact browser animation times.

Usage: python3 scripts/capture-button-states.py /path/to/floor-material output.json
Requires npx, git, and agent-browser. Dart Sass is pinned to 1.85.1.
"""
from pathlib import Path
import tempfile

import reference_capture as capture
from reference_capture import REVISION


def main():
    parser = capture.parser(__doc__)
    parser.add_argument("--reduced-motion", action="store_true")
    args = parser.parse_args()
    source = args.source.resolve()
    capture.require_reference(parser, source, "src/styles", "The reference checkout must be at ",
                              "Reference styles must be clean before capture")
    sampler = capture.sampler(__file__)
    with tempfile.TemporaryDirectory(prefix="material-button-states-") as directory:
        work = Path(directory)
        capture.compile_sass(source / "src/styles/main.scss", work / "buttons.css")
        (work / "index.html").write_text('<!doctype html><html><head><meta charset="utf-8"></head><body></body></html>')
        with capture.serve(directory) as port, capture.browser_session("material-button-states-") as browser:
            browser("open", f"http://127.0.0.1:{port}")
            evaluate = browser.evaluate
            if args.reduced_motion:
                browser("set", "media", "light", "reduced-motion")
            evaluate(sampler)
            data = evaluate("({reference:window.reference,ua:navigator.userAgent,reduced_motion:matchMedia('(prefers-reduced-motion: reduce)').matches})")
            if data["reference"] != REVISION or data["reduced_motion"] != args.reduced_motion:
                raise ValueError("Reference revision or motion preference did not match")
            browser("mouse", "move", "500", "500")
            browser("mouse", "move", "60", "40")
            data["hover"] = evaluate("window.sampleState()")
            evaluate("window.finishState()")
            browser("mouse", "down", "left")
            data["pressed_over_hover"] = evaluate("window.sampleState()")
            evaluate("window.finishState()")
            browser("mouse", "up", "left")
            evaluate("window.finishState()")
            browser("mouse", "move", "500", "500")
            evaluate("window.finishState(); document.getElementById('sentinel').focus(); true")
            browser("press", "Tab")
            data["keyboard_focus"] = evaluate("window.sampleState()")
            evaluate("window.finishState(); document.activeElement.blur(); true")
            browser("mouse", "move", "60", "40")
            evaluate("window.finishState()")
            evaluate("document.getElementById('probe').disabled=true; true")
            browser("mouse", "move", "61", "40")
            data["disabled_exit"] = evaluate("window.sampleState()")
            for state in ("hover", "pressed_over_hover", "keyboard_focus", "disabled_exit"):
                captured = {(item["property"], item["pseudo"]): item
                            for item in data[state]["animations"]}
                layer = captured.get(("opacity", "::before"))
                if layer is None or layer["duration"] != 100 or layer["time"] != 50:
                    raise ValueError("Missing or unexpected state-layer transition: " + state)
                expected_target = {"hover": 0.08, "pressed_over_hover": 0.10,
                                   "keyboard_focus": 0.10, "disabled_exit": 0.0}[state]
                if float(layer["frames"][-1]["opacity"]) != expected_target:
                    raise ValueError("Unexpected state-layer target: " + state)
                if state != "keyboard_focus":
                    shadow = captured.get(("box-shadow", None))
                    if shadow is None or shadow["duration"] != 250 or shadow["time"] != 125:
                        raise ValueError("Missing or unexpected shadow transition: " + state)
            if not (data["hover"]["hovered"] and data["pressed_over_hover"]["pressed"]
                    and data["pressed_over_hover"]["hovered"] and data["keyboard_focus"]["focused"]
                    and not data["keyboard_focus"]["hovered"] and data["disabled_exit"]["disabled"]
                    and not data["disabled_exit"]["hovered"]):
                raise ValueError("Browser pseudo-class state did not match the requested events")
            capture.write_json(args.output, data)


if __name__ == "__main__":
    main()
