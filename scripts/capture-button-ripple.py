#!/usr/bin/env python3
"""Capture pinned reference button ripple in Chrome at exact WAAPI animation times.

Usage: python3 scripts/capture-button-ripple.py /path/to/material output.json
Captures ordinary and reduced motion. Requires git, bun, npx, agent-browser.
Uses original createButton/createRipple and Sass 1.85.1; no reference source edits.
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


def validate(data, mode):
    reduced = mode == "reduced"
    expansion = data["expansion"]
    release = data["release"]
    if reduced:
        if expansion["animations"] or release["animations"]:
            raise ValueError("Reduced motion unexpectedly animated")
    else:
        for animation in expansion["animations"] + release["animations"]:
            timing = animation["timing"]
            easing = (animation["keyframes"][0]["easing"] if animation["type"] == "CSSAnimation"
                      else timing["easing"])
            if timing["duration"] != 400 or easing != "cubic-bezier(0.4, 0, 0.2, 1)":
                raise ValueError("Unexpected reference ripple timing")
        if not any(a["name"] == "opacity" for a in release["animations"]):
            raise ValueError("Missing release opacity transition")
        if expansion["samples"][0]["transform"] != "matrix(0, 0, 0, 0, 0, 0)":
            raise ValueError("Expansion did not start at zero")
    if data["initial"]["opacity"] != 0.1 or release["samples"][-1]["opacity"] != 0:
        raise ValueError("Unexpected ripple opacity")
    for geometry in data["geometry"]:
        border = geometry["border"]
        if geometry["center_offset"] != {"x": border, "y": border}:
            raise ValueError("Reference border origin changed")
        if geometry["wave"]["width"] != 240:
            raise ValueError("Reference diameter changed")
    before = dict(data["paint_before"]["pixel_counts"])
    after = dict(data["paint_order"]["pixel_counts"])
    if not before.get("0,0,0,255") or not after.get("26,26,26,255"):
        raise ValueError("Raster probe did not establish ripple above black glyph interiors")
    if data["keyboard_and_click_wave_count"] != 0 or data["right_pointerdown_wave_count"] != 1:
        raise ValueError("Unexpected ripple event routing")
    if data["element_mouseleave"] != "mtrl-ripple-wave active":
        raise ValueError("Element mouseleave unexpectedly released ripple")
    for name in ("document_mouseleave", "pointercancel"):
        if data[name] != "mtrl-ripple-wave active fade-out":
            raise ValueError("Expected document cancellation: " + name)


def validate_dynamic(data):
    for key, snapshot in data.items():
        if key.endswith("_events"):
            continue
        expected_reduced = key in ("held_reduced", "fade_reduced", "release_while_reduced")
        if snapshot["reduced_motion"] != expected_reduced or not snapshot["connected"]:
            raise ValueError("Dynamic probe media/lifetime mismatch: " + key)
    held = data["held_reduced"]
    if held["animations"] or held["opacity"] != 0.1 or held["transform"] != "matrix(1, 0, 0, 1, 0, 0)":
        raise ValueError("Held reduced wave must immediately cover the button")
    before = data["held_before"]["animations"][0]
    restored = data["held_restored"]["animations"][0]
    if before["id"] == restored["id"] or restored["time"] != 0:
        raise ValueError("Returning to normal motion must create a fresh expansion")
    if data["held_restored"]["transform"] != "matrix(0, 0, 0, 0, 0, 0)":
        raise ValueError("New expansion must start at zero")
    if data["held_restored_100ms"]["transform"] != data["held_before"]["transform"]:
        raise ValueError("New expansion must reproduce the 100ms position")
    if not 0 < data["fade_before"]["opacity"] < 0.1:
        raise ValueError("Fade probe must start during an actual opacity transition")
    original_fade = next(a for a in data["fade_before"]["animations"] if a["name"] == "opacity")
    for key in ("fade_reduced", "fade_restored", "fade_restored_100ms"):
        fade = next((a for a in data[key]["animations"] if a["name"] == "opacity"), None)
        if fade is None or fade["id"] != original_fade["id"] or data[key]["opacity"] != data["fade_before"]["opacity"]:
            raise ValueError("Existing fade must survive the full stylesheet media change: " + key)
    if data["fade_reduced"]["transition"]["duration"] != "0s":
        raise ValueError("Reduced motion must disable new opacity transitions")
    for key in ("release_while_reduced", "reduced_release_restored", "reduced_release_restored_100ms"):
        if data[key]["opacity"] != 0 or any(a["name"] == "opacity" for a in data[key]["animations"]):
            raise ValueError("Changing media resurrected a faded wave: " + key)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    source = args.source.resolve()
    if run("git", "-C", str(source), "rev-parse", "HEAD").strip() != REVISION:
        parser.error("Reference HEAD must be " + REVISION)
    # All imports of the actual button and ripple must match the pinned revision.
    if run("git", "-C", str(source), "status", "--porcelain", "--", "src"):
        parser.error("Reference src (including styles and ripple) must be clean")
    sampler = Path(__file__).with_suffix(".js").read_text()
    data = {"reference": REVISION, "tools": {
        "bun": run("bun", "--version").strip(),
        "sass": run("npx", "--yes", "sass@1.85.1", "--version").strip(),
        "agent_browser": run("agent-browser", "--version").strip(),
    }, "modes": {}}
    with tempfile.TemporaryDirectory(prefix="material-button-ripple-") as directory:
        work = Path(directory)
        entry = ("import createButton from " + json.dumps(str(source / "src/components/button/button.ts")) + ";\n"
                 "import {createRipple} from " + json.dumps(str(source / "src/core/compose/features/ripple.ts")) + ";\n"
                 "window.referenceCreateButton=createButton;window.referenceCreateRipple=createRipple;\n")
        (work / "entry.ts").write_text(entry)
        run("bun", "build", str(work / "entry.ts"), "--outfile", str(work / "entry.js"))
        run("npx", "--yes", "sass@1.85.1", "--no-source-map", "--style=expanded",
            str(source / "src/styles/main.scss"), str(work / "buttons.css"))
        (work / "index.html").write_text('<!doctype html><meta charset="utf-8"><link rel="stylesheet" href="buttons.css"><script src="entry.js"></script><body></body>')
        handler = functools.partial(http.server.SimpleHTTPRequestHandler, directory=directory)
        server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), handler)
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        try:
            for mode in ("ordinary", "reduced"):
                session = "material-button-ripple-" + uuid.uuid4().hex[:12]
                def browser(*commands):
                    return run("agent-browser", "--session", session, *commands)
                def evaluate(code):
                    return json.loads(run("agent-browser", "--session", session, "eval", "--stdin", input=code))
                try:
                    browser("open", f"http://127.0.0.1:{server.server_port}")
                    if mode == "reduced":
                        browser("set", "media", "light", "reduced-motion")
                    browser("mouse", "move", "900", "700")
                    captured = evaluate(sampler)
                    if captured["reduced_motion"] != (mode == "reduced"):
                        raise ValueError("Motion preference mismatch")
                    evaluate("window.preparePaintProbe(); true")
                    evaluate("document.querySelector('.mtrl-ripple').style.visibility='hidden'; true")
                    browser("screenshot", str(work / "paint-before.png"))
                    captured["paint_before"] = evaluate("window.readPaintProbe('/paint-before.png')")
                    evaluate("document.querySelector('.mtrl-ripple').style.visibility='visible'; true")
                    browser("screenshot", str(work / "paint.png"))
                    captured["paint_order"] = evaluate("window.readPaintProbe('/paint.png')")
                    validate(captured, mode)
                    data["modes"][mode] = captured
                    if mode == "ordinary":
                        dynamic = {}
                        dynamic["held_before"] = evaluate("window.startDynamicProbe()")
                        browser("set", "media", "light", "reduced-motion")
                        dynamic["held_reduced"] = evaluate("window.dynamicSnapshot()")
                        browser("set", "media", "light")
                        dynamic["held_restored"] = evaluate("window.dynamicSnapshot()")
                        dynamic["held_restored_100ms"] = evaluate("window.seekDynamicProbe(100)")
                        dynamic["held_media_events"] = evaluate("window.dynamicMediaSnapshots()")
                        dynamic["fade_before"] = evaluate("window.startDynamicProbe(); window.releaseDynamicProbe()")
                        browser("set", "media", "light", "reduced-motion")
                        dynamic["fade_reduced"] = evaluate("window.dynamicSnapshot()")
                        browser("set", "media", "light")
                        dynamic["fade_restored"] = evaluate("window.dynamicSnapshot()")
                        dynamic["fade_restored_100ms"] = evaluate("window.seekDynamicProbe(100)")
                        dynamic["fade_media_events"] = evaluate("window.dynamicMediaSnapshots()")
                        browser("set", "media", "light", "reduced-motion")
                        dynamic["release_while_reduced"] = evaluate("window.startDynamicProbe(); window.releaseDynamicProbe()")
                        browser("set", "media", "light")
                        dynamic["reduced_release_restored"] = evaluate("window.dynamicSnapshot()")
                        dynamic["reduced_release_restored_100ms"] = evaluate("window.seekDynamicProbe(100)")
                        validate_dynamic(dynamic)
                        data["dynamic_media"] = dynamic
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
