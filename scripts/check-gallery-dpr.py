#!/usr/bin/env python3
"""Exercise a served WASM gallery across DPR changes and record real canvas sizes.

Usage: python3 scripts/check-gallery-dpr.py http://127.0.0.1:8080 /tmp/gallery-dpr
Requires agent-browser and a freshly built/served gallery. Does not assert full
component pixel conformance. Each screenshot uses the recorded emulated DPR.
"""
import argparse
import json
from pathlib import Path

import reference_capture as capture


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("url")
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    samples = []
    with capture.browser_session("gallery-dpr-") as browser:
        evaluate = browser.evaluate
        browser("open", args.url)
        for index, (width, height, dpr) in enumerate([
            (960, 720, 1), (960, 720, 2), (960, 720, 1.25),
            (1280, 720, 1.5), (960, 720, 1),
        ]):
            # CDP can change devicePixelRatio without resize or media-query events.
            # Produce a real CSS resize to exercise the backend observer as well.
            browser("set", "viewport", str(width + 1), str(height), str(dpr))
            browser("set", "viewport", str(width), str(height), str(dpr))
            settled = evaluate("""new Promise((resolve, reject) => {
                const started=performance.now();
                const check=()=>{
                    const canvas=document.querySelector('canvas');
                    const rect=canvas?.getBoundingClientRect();
                    if(canvas && rect && devicePixelRatio===DPR && rect.width===WIDTH && rect.height===HEIGHT
                        && canvas.width===Math.round(WIDTH*DPR) && canvas.height===Math.round(HEIGHT*DPR)) {
                        resolve(performance.now()-started); return;
                    }
                    if(performance.now()-started>5000) { reject(new Error(JSON.stringify({expected:[WIDTH,HEIGHT,DPR],observed:{dpr:devicePixelRatio,css:rect&&[rect.width,rect.height],backing:canvas&&[canvas.width,canvas.height]}}))); return; }
                    requestAnimationFrame(check);
                }; requestAnimationFrame(check);
            })""".replace("DPR", str(dpr)).replace("WIDTH", str(width)).replace("HEIGHT", str(height)))
            sample = evaluate("""(() => {
                const canvas=document.querySelector('canvas');
                if(!canvas) throw new Error('Gallery canvas not mounted');
                const rect=canvas.getBoundingClientRect();
                return {ua:navigator.userAgent,dpr:devicePixelRatio,zoom:visualViewport.scale,
                    viewport:[innerWidth,innerHeight],css:[rect.width,rect.height],
                    backing:[canvas.width,canvas.height]};
            })()""")
            if sample["dpr"] != dpr or sample["css"] != [width, height]:
                raise ValueError(f"Emulation or gallery CSS dimensions differ: {sample}")
            expected = [round(width*dpr), round(height*dpr)]
            if sample["backing"] != expected:
                raise ValueError(f"Expected backing {expected}, observed {sample}")
            sample["settle_ms"] = settled
            image = args.output / f"{index}-dpr-{dpr}.png"
            browser("screenshot", str(image.resolve()))
            sample["screenshot"] = image.name
            samples.append(sample)
        result = {"scope": "WASM canvas allocation across DPR and viewport changes; not component pixel conformance",
                  "protocol": "Each emulated DPR change includes a 1 CSS pixel viewport resize because CDP DPR-only changes may emit neither resize nor media-query events.",
                  "samples": samples}
        (args.output / "results.json").write_text(json.dumps(result, indent=2)+"\n")


if __name__ == "__main__":
    main()
