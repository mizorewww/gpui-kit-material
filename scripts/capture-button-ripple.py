#!/usr/bin/env python3
"""Capture pinned reference button ripple in Chrome at exact WAAPI animation times.

Usage: python3 scripts/capture-button-ripple.py /path/to/material output.json
Captures ordinary and reduced motion. Requires git, bun, npx, agent-browser.
Uses original createButton/createRipple and Sass 1.85.1; no reference source edits.
"""
from pathlib import Path
import tempfile

import reference_capture as capture
from reference_capture import REVISION


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
    parser = capture.parser(__doc__)
    args = parser.parse_args()
    source = args.source.resolve()
    # All imports of the actual button and ripple must match the pinned revision.
    capture.require_reference(parser, source, "src", "Reference HEAD must be ",
                              "Reference src (including styles and ripple) must be clean")
    sampler = capture.sampler(__file__)
    data = {"reference": REVISION, "tools": capture.tool_versions(), "modes": {}}
    with tempfile.TemporaryDirectory(prefix="material-button-ripple-") as directory:
        work = Path(directory)
        capture.bundle(work, "import createButton from " + capture.module(source / "src/components/button/button.ts") + ";\n"
                       "import {createRipple} from " + capture.module(source / "src/core/compose/features/ripple.ts") + ";\n"
                       "window.referenceCreateButton=createButton;window.referenceCreateRipple=createRipple;\n")
        capture.compile_sass(source / "src/styles/main.scss", work / "buttons.css")
        (work / "index.html").write_text('<!doctype html><meta charset="utf-8"><link rel="stylesheet" href="buttons.css"><script src="entry.js"></script><body></body>')
        with capture.serve(directory) as port:
            for mode in ("ordinary", "reduced"):
                with capture.browser_session("material-button-ripple-") as browser:
                    evaluate = browser.evaluate
                    browser("open", f"http://127.0.0.1:{port}")
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
            capture.write_json(args.output, data)


if __name__ == "__main__":
    main()
