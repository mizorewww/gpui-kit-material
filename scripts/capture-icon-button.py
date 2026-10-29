#!/usr/bin/env python3
"""Capture pinned reference icon-button geometry, interactions, accessibility, and motion.

Usage: python3 scripts/capture-icon-button.py /path/to/material output.json
Captures actual CSS pointer/keyboard states, API behavior, touch boundaries, and transition tracks. Requires git, bun, npx, agent-browser.
Uses original createIconButton and complete main.scss compiled with Sass 1.85.1; no reference source edits.
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
    result = subprocess.run(args, text=True, capture_output=True, **kwargs)
    if result.returncode:
        raise RuntimeError(f"{args[0]} failed ({result.returncode}): {result.stderr or result.stdout}")
    return result.stdout


def unpack_fixture(fixture):
    """Expand losslessly interned computed styles; also accepts raw checkpoints."""
    if "style_table" not in fixture:
        return fixture
    table=fixture["style_table"]
    def expand(value):
        if isinstance(value,list): return [expand(item) for item in value]
        if isinstance(value,dict):
            if set(value)=={"$style"}: return dict(table[value["$style"]])
            return {key:expand(item) for key,item in value.items()}
        return value
    return expand({key:value for key,value in fixture.items() if key not in ("schema","style_table")})


def pack_fixture(data):
    """Intern identical computed-style dictionaries in deterministic traversal order."""
    styles=[];indices={}
    def pack(value,key=None):
        if isinstance(value,dict):
            if key in ("css","state_layer","touch_target"):
                canonical=json.dumps(value,sort_keys=True,separators=(",",":"))
                if canonical not in indices:
                    indices[canonical]=len(styles)
                    styles.append(dict(sorted(value.items())))
                return {"$style":indices[canonical]}
            return {name:pack(item,name) for name,item in sorted(value.items())}
        if isinstance(value,list):return [pack(item) for item in value]
        return value
    result=pack(data)
    result["schema"]={"version":1,"encoding":"Computed-style dictionaries at css/state_layer/touch_target are replaced by {$style: index} references into style_table. All other values are verbatim. Traversal and style keys are sorted; list order is preserved.","expand":"python3 scripts/capture-icon-button.py --expand conformance/reference/icon-button.json /tmp/icon-button-expanded.json"}
    result["style_table"]=styles
    return result


def validate(data):
    heights={"xs":32,"s":40,"m":56,"l":96,"xl":136}
    widths={"xs":[28,32,40],"s":[32,40,52],"m":[48,56,72],"l":[64,96,128],"xl":[104,136,184]}
    icons={"xs":20,"s":24,"m":24,"l":32,"xl":40}
    square={"xs":12,"s":12,"m":16,"l":28,"xl":28}
    pressed={"xs":8,"s":8,"m":12,"l":16,"xl":16}
    assert data["theme"]["name"] == "baseline" and data["theme"]["mode"] == "light"
    assert data["dark_theme"]["primary"] != data["theme"]["primary"], "dark mode did not change actual palette"
    assert len(data["states"]) == 120
    assert sum(len(row)-1 for row in data["states"]) == 1800
    for row in data["states"]:
        c=row["config"];size=c["size"];height=heights[size];width=widths[size][["narrow","default","wide"].index(c["width"])]
        for state,sample in row.items():
            if state=="config":continue
            assert sample["rect"]["width"]==width and sample["rect"]["height"]==height,(c,state,"geometry")
            assert sample["icon"]["rect"]["width"]==icons[size] and sample["icon"]["rect"]["height"]==icons[size],(c,state,"icon")
            assert sample["css"]["borderTopWidth"] == (str({"xs":1,"s":1,"m":1,"l":2,"xl":3}[size])+"px" if c["variant"]=="outlined" else "0px"), (c,state,"outline thickness")
            expected=pressed[size] if "pressed" in state else (square[size] if c["shape"]=="round" else height/2) if state.startswith("selected") else height/2 if c["shape"]=="round" else square[size]
            assert sample["css"]["borderTopLeftRadius"]==f"{expected:g}px",(c,state,"radius",sample["css"]["borderTopLeftRadius"],expected)
            if "pressed" in state: assert sample["active"],(c,state,"real active")
            if "focus" in state: assert sample["focus_visible"] and sample["state_layer"]["opacity"]=="0.1",(c,state,"keyboard focus")
            if "hover" in state: assert sample["hover"] and sample["state_layer"]["opacity"]=="0.08",(c,state,"real hover")
            if "disabled" in state: assert sample["disabled"] and sample["state_layer"]["display"]=="none",(c,state,"disabled")
        assert row["plain"]["aria_pressed"] is None and row["toggle_off"]["aria_pressed"]=="false" and row["selected"]["aria_pressed"]=="true"
        assert row["plain"]["svg"]["identity"]=="off" and row["selected"]["svg"]["identity"]=="on"
    for row in data["touch"]:
        assert len(row["targets"][0]["snapshot"]["events"])==0,(row["size"],row["width"],"outside hit")
        assert len(row["targets"][1]["snapshot"]["events"])==1,(row["size"],row["width"],"expanded hit")
    for row in data["events"]:
        expected=1 if row["mode"]=="automatic" else 0
        assert len(row["pointer_click"]["events"])==expected,row["mode"]
        if row["mode"]=="automatic":
            assert len(row["keyboard_space"]["events"])==2 and len(row["keyboard_enter"]["events"])==3
    assert len(data["motion"])==80
    for row in data["motion"]:
        assert row["media"]==row["reduced"]
        if not row["reduced"]:
            assert any(t["property"]=="border-top-left-radius" for t in row["selection"]["tracks"]),row["config"]
    assert len(data["disabled_focus_lifecycle"])==8
    for row in data["disabled_focus_lifecycle"]:
        assert row["before"]["focus_visible"] and row["immediate"]["disabled"]
        for phase in ("after_frame","after_blur"):
            assert row[phase]["disabled"] and not row[phase]["focus"] and not row[phase]["focus_visible"]
            assert row[phase]["css"]["outlineStyle"]=="none"
    for row in data["motion"]:
        for phase in ("selection","press"):
            for track in row[phase]["tracks"]:
                if "radius" in (track["property"] or ""):
                    assert not row["reduced"] and track["timing"]["duration"]==50
                    assert track["timing"]["easing"]=="cubic-bezier(0.3, 0, 1, 1)"
    for row in data["dynamic_reduced"]:
        assert row["after"]["css"]["borderTopLeftRadius"]=="16px"
    light_standard=next(row for row in data["states"] if row["config"]==dict(variant="standard",size="s",width="default",shape="round"))
    assert data["dark_endpoints"][0]["selected"]["css"]["color"] != light_standard["selected"]["css"]["color"]
    data["assertions"]={"disabled_focus_lifecycle_cases":8,"state_configurations":120,"state_samples":1800,"motion_configurations":80,"touch_boundary_cases":6,"event_modes":4,"validated":True}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path, nargs="?")
    parser.add_argument("output", type=Path, nargs="?")
    parser.add_argument("--resume", action="store_true", help="Reuse compatible completed samples and finish missing phases")
    parser.add_argument("--expand", nargs=2, type=Path, metavar=("INPUT", "OUTPUT"), help="Expand a compact fixture without running a browser")
    args = parser.parse_args()
    if args.expand:
        original=unpack_fixture(json.loads(args.expand[0].read_text()))
        args.expand[1].write_text(json.dumps(original,indent=2)+"\n")
        return
    if args.source is None or args.output is None: parser.error("source and output are required for capture")
    source = args.source.resolve()
    if run("git", "-C", str(source), "rev-parse", "HEAD").strip() != REVISION:
        parser.error("Reference HEAD must be " + REVISION)
    # All imports of the actual icon button must match the pinned revision.
    if run("git", "-C", str(source), "status", "--porcelain", "--", "src"):
        parser.error("Reference src (including shared features and styles) must be clean")
    sampler = Path(__file__).with_suffix(".js").read_text()
    data = {"reference": REVISION, "tools": {
        "bun": run("bun", "--version").strip(),
        "sass": run("npx", "--yes", "sass@1.85.1", "--version").strip(),
        "agent_browser": run("agent-browser", "--version").strip(),
    }}
    with tempfile.TemporaryDirectory(prefix="material-icon-button-") as directory:
        work = Path(directory)
        entry = ("import createIconButton from " + json.dumps(str(source / "src/components/icon-button/icon-button.ts")) + ";\n"
                 "window.referenceCreateIconButton=createIconButton;\n")
        (work / "entry.ts").write_text(entry)
        run("bun", "build", str(work / "entry.ts"), "--outfile", str(work / "entry.js"))
        run("npx", "--yes", "sass@1.85.1", "--no-source-map", "--style=expanded",
            str(source / "src/styles/main.scss"), str(work / "buttons.css"))
        data["stylesheet_sha256"] = hashlib.sha256((work / "buttons.css").read_bytes()).hexdigest()
        (work / "index.html").write_text('<!doctype html><html data-theme="baseline" data-theme-mode="light"><meta charset="utf-8"><link rel="stylesheet" href="buttons.css"><script src="entry.js"></script><body></body>')
        handler = functools.partial(http.server.SimpleHTTPRequestHandler, directory=directory)
        server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), handler)
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        try:
            session = "material-icon-button-" + uuid.uuid4().hex[:12]
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
                data["provenance"] = {"factory": "src/components/icon-button/icon-button.ts", "stylesheet": "src/styles/main.scss", "source_clean": True,
                    "sampler_sha256": hashlib.sha256(sampler.encode()).hexdigest(),
                    "sources": {name: hashlib.sha256((source / name).read_bytes()).hexdigest() for name in (
                        "src/components/icon-button/icon-button.ts", "src/components/icon-button/config.ts", "src/components/icon-button/api.ts",
                        "src/components/icon-button/features/toggle.ts", "src/styles/components/_icon-button.scss", "src/styles/base/_reset.scss")}}
                data["notes"] = ["Actual createIconButton and full main.scss. The main matrix changes only absolute test placement; separately named API override probes set the recorded shape custom properties.",
                    "State endpoints finish finite browser CSS animations. Motion samples pause and seek actual CSS transitions at the recorded shared elapsed milliseconds.",
                    "Main state matrix disables ripple to isolate state-layer opacity; separate ripple samples retain the reference ripple.",
                    "Computed CSS radius is the specified value; Chromium applies CSS corner-overlap scaling at paint time on narrow containers."]
                def mount(config):
                    browser("mouse", "move", "500", "400")
                    return evaluate("mountIcon(" + json.dumps(config) + ")")
                def move(point):
                    browser("mouse", "move", str(point["x"]), str(point["y"]))
                def snap():
                    return evaluate("snapshotIcon()")
                def focus():
                    evaluate("prepareIconFocus()")
                    browser("press", "Tab")
                    return snap()
                data["theme"] = evaluate("({name:document.documentElement.dataset.theme,mode:document.documentElement.dataset.themeMode,primary:getComputedStyle(document.documentElement).getPropertyValue('--mtrl-sys-color-primary').trim()})")
                data["states"] = []
                previous={}
                if args.resume and args.output.exists():
                    previous=unpack_fixture(json.loads(args.output.read_text()))
                    identity_keys=("reference","provenance","stylesheet_sha256","tools","ua","media","theme")
                    if any(previous.get(key) != data.get(key) for key in identity_keys):
                        raise ValueError("Checkpoint source/tools/browser/media/theme differs; do a fresh capture")
                    data["states"]=previous.get("states", [])
                def save():
                    args.output.parent.mkdir(parents=True, exist_ok=True)
                    args.output.write_text(json.dumps(pack_fixture(data), indent=2) + "\n")
                def batch(commands):
                    run("agent-browser", "--session", session, "batch", "--bail", "--json", input=json.dumps(commands))
                completed=[row["config"] for row in data["states"]]
                for variant in ("standard", "filled", "tonal", "outlined"):
                    for size in ("xs", "s", "m", "l", "xl"):
                        for width in ("narrow", "default", "wide"):
                            for shape in ("round", "square"):
                                config = dict(variant=variant, size=size, width=width, shape=shape)
                                if config in completed: continue
                                point = mount(config)
                                def ev(code): return ["eval", code + ";true"]
                                commands=[ev("window.iconRow={config:"+json.dumps(config)+",plain:snapshotIcon()}"),
                                    ["mouse","move",str(point["x"]),str(point["y"])],ev("iconRow.hover=snapshotIcon()"),
                                    ["mouse","down","left"],ev("iconRow.pressed=snapshotIcon()"),["mouse","up","left"],["mouse","move","500","400"],
                                    ev("prepareIconFocus()"),["press","Tab"],ev("iconRow.focus=snapshotIcon();iconButton.disable();iconRow.disabled=snapshotIcon()"),
                                    ev("mountIcon("+json.dumps(dict(config,toggle=True))+");iconRow.toggle_off=snapshotIcon();iconButton.disable();iconRow.toggle_off_disabled=snapshotIcon();iconButton.enable();iconButton.select();iconRow.selected=snapshotIcon()"),
                                    ["mouse","move",str(point["x"]),str(point["y"])],ev("iconRow.selected_hover=snapshotIcon()"),
                                    ["mouse","down","left"],ev("iconRow.selected_pressed=snapshotIcon()"),["mouse","up","left"],["mouse","move","500","400"],
                                    ev("prepareIconFocus()"),["press","Tab"],ev("iconRow.selected_focus=snapshotIcon();iconButton.disable();iconRow.selected_disabled=snapshotIcon()")]
                                batch(commands)
                                data["states"].append(evaluate("iconRow"))
                        save()
                        print("State samples ready:", variant, size, flush=True)
                    print("State variant ready:", variant, flush=True)
                if previous.get("assertions",{}).get("validated"):
                    for key in ("api","events","touch","motion","ripple","dynamic_reduced","dark_theme","dark_endpoints"):
                        data[key]=previous[key]
                else:
                    data["api"] = evaluate("iconApiSamples()")
                    data["events"] = []
                    for mode in ("automatic", "controlled", "plain", "disabled"):
                        point = mount(dict(toggle=mode != "plain", toggleOnClick=mode != "controlled", disabled=mode == "disabled", value="reference-value"))
                        row = {"mode": mode, "before": snap()}
                        move(point); browser("mouse", "down", "left"); browser("mouse", "up", "left")
                        row["pointer_click"] = snap()
                        if mode != "disabled":
                            focus(); browser("press", "Space"); row["keyboard_space"] = snap()
                            browser("press", "Enter"); row["keyboard_enter"] = snap()
                        data["events"].append(row)
                    data["touch"] = []
                    for size in ("xs", "s"):
                        for width in ("narrow", "default", "wide"):
                            point = mount(dict(size=size, width=width, toggle=True, toggleOnClick=True))
                            before = snap(); bounds=before["rect"]
                            # One CSS pixel outside the union of the visual and 48px target, then one inside its left edge.
                            left = min(bounds["x"], point["x"] - 24)
                            targets=[]
                            for x in (left-1, left+1):
                                move(dict(x=x,y=point["y"])); browser("mouse", "down", "left"); browser("mouse", "up", "left")
                                targets.append({"x":x,"y":point["y"],"snapshot":snap()})
                            data["touch"].append(dict(size=size,width=width,before=before,targets=targets))
                    data["motion"] = []
                    for reduced in (False, True):
                        browser("set", "media", "light", *( ["reduced-motion"] if reduced else [] ))
                        for variant in ("standard", "filled", "tonal", "outlined"):
                            for size in ("xs", "s", "m", "l", "xl"):
                                for shape in ("round", "square"):
                                    config=dict(variant=variant,size=size,shape=shape,toggle=True)
                                    point=mount(config)
                                    row=dict(config=config,reduced=reduced,media=evaluate("matchMedia('(prefers-reduced-motion: reduce)').matches"),selection=evaluate("selectIconMotion()"))
                                    evaluate("iconButton.deselect();snapshotIcon();armIconPressMotion()")
                                    move(point); browser("mouse", "down", "left")
                                    browser("wait", "--fn", "window.iconPressMotion !== null")
                                    row["press"]=evaluate("iconPressMotion")
                                    browser("mouse", "up", "left")
                                    data["motion"].append(row)
                        print("Captured motion: reduced=", reduced, flush=True)
                    browser("set", "media", "light")
                    data["ripple"] = []
                    for selected in (False, True):
                        point=mount(dict(toggle=True,selected=selected,ripple=True))
                        move(point);browser("mouse","down","left")
                        data["ripple"].append(dict(selected=selected,snapshot=snap(),children=evaluate("Array.from(iconButton.element.children).map(e=>({classes:e.className,style:e.getAttribute('style')}))")))
                        browser("mouse","up","left")
                    data["dynamic_reduced"] = []
                    for variant in ("standard", "filled", "tonal", "outlined"):
                        browser("set","media","light")
                        mount(dict(variant=variant,size="m",toggle=True))
                        before=evaluate("snapshotIcon();iconButton.select();void getComputedStyle(iconButton.element).borderRadius;iconButton.element.getAnimations({subtree:true}).forEach(a=>{a.pause();a.currentTime=25});snapshotIcon(false)")
                        browser("set","media","light","reduced-motion")
                        after=snap()
                        data["dynamic_reduced"].append(dict(variant=variant,before=before,after=after))
                    browser("set","media","light")
                    data["dark_endpoints"]=[]
                    evaluate("document.documentElement.dataset.themeMode='dark'")
                    data["dark_theme"]=evaluate("({name:document.documentElement.dataset.theme,mode:document.documentElement.dataset.themeMode,primary:getComputedStyle(document.documentElement).getPropertyValue('--mtrl-sys-color-primary').trim()})")
                    for variant in ("standard","filled","tonal","outlined"):
                        mount(dict(variant=variant,toggle=True))
                        row=dict(variant=variant,unselected=snap())
                        evaluate("iconButton.select()");row["selected"]=snap()
                        evaluate("iconButton.disable()");row["disabled"]=snap()
                        data["dark_endpoints"].append(row)
                evaluate("document.documentElement.dataset.themeMode='light'")
                browser("set","media","light")
                # Complete all three interaction states for toggle-off as well:
                # filled toggle-off has different colors from a plain button.
                for index,row in enumerate(data["states"]):
                    if "toggle_off_hover" in row: continue
                    point=mount(dict(row["config"],toggle=True))
                    def ev(code): return ["eval", code+";true"]
                    batch([ev("window.iconExtra={}"),["mouse","move",str(point["x"]),str(point["y"])],
                           ev("iconExtra.toggle_off_hover=snapshotIcon()"),["mouse","down","left"],
                           ev("iconExtra.toggle_off_pressed=snapshotIcon()"),["mouse","up","left"],["mouse","move","500","400"],
                           ev("prepareIconFocus()"),["press","Tab"],ev("iconExtra.toggle_off_focus=snapshotIcon()")])
                    row.update(evaluate("iconExtra"))
                    if (index+1)%6==0: save()
                data["disabled_focus_lifecycle"]=[]
                for variant in ("standard","filled","tonal","outlined"):
                    for selected in (False,True):
                        mount(dict(variant=variant,toggle=True,selected=selected))
                        before=focus()
                        immediate=evaluate("iconButton.disable();snapshotIcon()")
                        after_frame=evaluate("(async()=>{await new Promise(requestAnimationFrame);return snapshotIcon()})()")
                        evaluate("iconButton.element.blur()")
                        after_blur=snap()
                        data["disabled_focus_lifecycle"].append(dict(variant=variant,selected=selected,before=before,immediate=immediate,after_frame=after_frame,after_blur=after_blur))
                data["notes"].append("Matrix disabled/selected_disabled snapshots are synchronous after disabling a keyboard-focused element. Consult disabled_focus_lifecycle for the subsequent animation-frame and explicit blur observations; finite CSS transitions alone are settled by snapshotIcon().")
                save()
                validate(data)
                assert unpack_fixture(pack_fixture(data)) == data, "style interning must be lossless"
            finally:
                browser("close")
            args.output.parent.mkdir(parents=True, exist_ok=True)
            args.output.write_text(json.dumps(pack_fixture(data), indent=2) + "\n")
        finally:
            server.shutdown()
            server.server_close()
            thread.join()


if __name__ == "__main__":
    main()
