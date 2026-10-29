#!/usr/bin/env python3
"""Capture pinned reference circular button geometry, real pointer states, and content.

Usage: python3 scripts/capture-button-circular.py /path/to/material output.json
Captures actual CSS :active and selected/disabled states plus icon/text/loading. Requires git, bun, npx, agent-browser.
Uses original createButton and complete main.scss compiled with Sass 1.85.1; no reference source edits.
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
    }}
    with tempfile.TemporaryDirectory(prefix="material-button-circular-") as directory:
        work = Path(directory)
        entry = ("import createButton from " + json.dumps(str(source / "src/components/button/button.ts")) + ";\n"
                 "window.referenceCreateButton=createButton;\n")
        (work / "entry.ts").write_text(entry)
        run("bun", "build", str(work / "entry.ts"), "--outfile", str(work / "entry.js"))
        run("npx", "--yes", "sass@1.85.1", "--no-source-map", "--style=expanded",
            str(source / "src/styles/main.scss"), str(work / "buttons.css"))
        data["stylesheet_sha256"] = hashlib.sha256((work / "buttons.css").read_bytes()).hexdigest()
        (work / "index.html").write_text('<!doctype html><meta charset="utf-8"><link rel="stylesheet" href="buttons.css"><script src="entry.js"></script><body></body>')
        handler = functools.partial(http.server.SimpleHTTPRequestHandler, directory=directory)
        server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), handler)
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        try:
            session = "material-button-circular-" + uuid.uuid4().hex[:12]
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
