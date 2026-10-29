#!/usr/bin/env python3
"""Capture the pinned md3 Sass themes using a real browser's CSS cascade.

Usage: python3 scripts/capture-reference-colors.py /path/to/floor-material output.json
Requires npx, git, and agent-browser. Dart Sass is pinned to 1.85.1.
"""
import json
from pathlib import Path
import tempfile

import reference_capture as capture


def main():
    parser = capture.parser(__doc__)
    args = parser.parse_args()
    source = args.source.resolve()
    capture.require_reference(parser, source, "src/styles", "The reference checkout must be at ",
                              "Reference styles must be clean before capture")
    sampler = capture.sampler(__file__)
    with tempfile.TemporaryDirectory(prefix="material-colors-") as directory:
        work = Path(directory)
        themes = sorted(path for path in (source / "src/styles/themes").glob("_*.scss")
                        if path.stem not in ("_base-theme", "_index"))
        if len(themes) != 20:
            parser.error("Expected the 20 pinned theme source files")
        (work / "reference.scss").write_text("\n".join("@use " + json.dumps(str(path)) + ";" for path in themes))
        capture.compile_sass(work / "reference.scss", work / "reference.css")
        (work / "index.html").write_text('<!doctype html><html><head><meta charset="utf-8"><link rel="stylesheet" href="reference.css"></head><body>Reference colors</body></html>')
        with capture.serve(directory) as port, capture.browser_session("material-colors-") as browser:
            browser("open", f"http://127.0.0.1:{port}")
            result = browser.eval_text(sampler)
            data = json.loads(result)
            if len(data["rows"]) != 120 or len(data["roles"]) != 52:
                raise ValueError("Incomplete reference capture")
            capture.write_json(args.output, data)


if __name__ == "__main__":
    main()
