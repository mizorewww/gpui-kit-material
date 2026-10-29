#!/usr/bin/env python3
"""Capture pinned md3 button state-layer and shadow transitions at exact browser animation times.

Usage: python3 scripts/capture-button-states.py /path/to/floor-material output.json
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
    parser.add_argument("--reduced-motion", action="store_true")
    args = parser.parse_args()
    source = args.source.resolve()
    if run("git", "-C", str(source), "rev-parse", "HEAD").strip() != REVISION:
        parser.error("The reference checkout must be at " + REVISION)
    if run("git", "-C", str(source), "status", "--porcelain", "--", "src/styles"):
        parser.error("Reference styles must be clean before capture")
    session = "material-button-states-" + uuid.uuid4().hex[:12]
    sampler = Path(__file__).with_suffix(".js").read_text()
    with tempfile.TemporaryDirectory(prefix="material-button-states-") as directory:
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
            def browser(*commands):
                return run("agent-browser", "--session", session, *commands)
            def evaluate(code):
                return json.loads(run(
                    "agent-browser", "--session", session, "eval", "--stdin", input=code))
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
