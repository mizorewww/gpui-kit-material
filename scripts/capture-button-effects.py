#!/usr/bin/env python3
"""Capture pinned md3 button color transitions at exact browser animation times.

Usage: python3 scripts/capture-button-effects.py /path/to/floor-material output.json
Requires npx, git, and agent-browser. Dart Sass is pinned to 1.85.1.
"""
import json
from pathlib import Path
import tempfile

import reference_capture as capture
from reference_capture import REVISION


def main():
    parser = capture.parser(__doc__)
    args = parser.parse_args()
    source = args.source.resolve()
    capture.require_reference(parser, source, "src/styles", "The reference checkout must be at ",
                              "Reference styles must be clean before capture")
    sampler = capture.sampler(__file__)
    with tempfile.TemporaryDirectory(prefix="material-button-effects-") as directory:
        work = Path(directory)
        capture.compile_sass(source / "src/styles/main.scss", work / "buttons.css")
        (work / "index.html").write_text('<!doctype html><html><head><meta charset="utf-8"></head><body></body></html>')
        with capture.serve(directory) as port, capture.browser_session("material-button-effects-") as browser:
            browser("open", f"http://127.0.0.1:{port}")
            result = browser.eval_text(sampler)
            data = json.loads(result)
            if data.get("reference") != REVISION:
                raise ValueError("Sampler reference revision does not match the checkout")
            for state, time in (("variant", 125), ("disabled", 125), ("reversed", 25)):
                if not {"background-color", "color"}.issubset(data[state]):
                    raise ValueError("Incomplete button transition capture: " + state)
                for property_name in ("background-color", "color"):
                    if data[state][property_name]["time"] != time:
                        raise ValueError("Incorrect animation sample time: " + state)
            capture.write_json(args.output, data)


if __name__ == "__main__":
    main()
