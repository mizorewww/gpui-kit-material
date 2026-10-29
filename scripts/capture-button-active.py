#!/usr/bin/env python3
"""Capture pinned reference controlled active button states.

Usage: python3 scripts/capture-button-active.py /path/to/material output.json
Captures actual setActive API across variants, sizes, shapes, ripple and selection. Requires git, bun, npx, agent-browser.
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
    with tempfile.TemporaryDirectory(prefix="material-button-active-") as directory:
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
            session = "material-button-active-" + uuid.uuid4().hex[:12]
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
