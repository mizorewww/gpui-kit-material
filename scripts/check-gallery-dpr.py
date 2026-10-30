#!/usr/bin/env python3
"""Check a served WASM gallery's canvas at real device scale factors.

Usage: python3 scripts/check-gallery-dpr.py http://127.0.0.1:8080 /tmp/gallery-dpr
Requires agent-browser and Chromium (Playwright's headless shell by default, or
--browser PATH). Each sample launches its own browser with
--force-device-scale-factor. CDP device-metrics emulation is deliberately not used:
it reports a device-pixel box that disagrees with devicePixelRatio, which no real
display does. Does not assert full component pixel conformance.
"""
import argparse
import glob
import json
import os
from pathlib import Path
import shutil
import socket
import subprocess
import tempfile
import time

SAMPLES = [
    (1, 960, 720),
    (2, 960, 720),
    (1.25, 960, 720),
    (1.5, 1280, 720),
    (2, 1280, 800),
    (1, 1280, 800),
]


def default_browser():
    pattern = os.path.expanduser(
        "~/Library/Caches/ms-playwright/chromium_headless_shell-*/*/chrome-headless-shell"
    )
    pattern_linux = os.path.expanduser(
        "~/.cache/ms-playwright/chromium_headless_shell-*/*/chrome-headless-shell"
    )
    found = sorted(glob.glob(pattern) + glob.glob(pattern_linux))
    return found[-1] if found else None


def free_port():
    with socket.socket() as sock:
        sock.bind(("127.0.0.1", 0))
        return sock.getsockname()[1]


def agent(port, *args, timeout=120):
    result = subprocess.run(
        ["agent-browser", "--cdp", str(port), *args],
        text=True,
        capture_output=True,
        timeout=timeout,
    )
    if result.returncode:
        raise RuntimeError(f"agent-browser {args[0]}: {result.stdout}{result.stderr}")
    return result.stdout.strip()


READ = """(() => {
  const canvas = document.querySelector('canvas');
  if (!canvas) return JSON.stringify({pending: true});
  const rect = canvas.getBoundingClientRect();
  return JSON.stringify({dpr: devicePixelRatio, viewport: [innerWidth, innerHeight],
    css: [rect.width, rect.height], backing: [canvas.width, canvas.height],
    ua: navigator.userAgent});
})()"""


def read_settled(port, deadline=240):
    """Short reads, so a slow software-rendered page cannot time out one command.
    Settled means the canvas is allocated and unchanged across two reads."""
    started = time.monotonic()
    previous = None
    while time.monotonic() - started < deadline:
        time.sleep(3)
        try:
            result = agent(port, "eval", READ)
        except RuntimeError:
            continue
        while isinstance(result, str):
            result = json.loads(result)
        if result.get("pending") or result["backing"][0] <= 300:
            continue
        if result == previous:
            return result
        previous = result
    raise ValueError(f"canvas did not settle; last read {previous}")


def sample(browser, url, output, scale, width, height):
    port = free_port()
    profile = tempfile.mkdtemp(prefix="gallery-dpr-")
    process = subprocess.Popen(
        [browser, f"--remote-debugging-port={port}", f"--user-data-dir={profile}",
         "--no-sandbox", "--no-first-run", f"--force-device-scale-factor={scale}",
         f"--window-size={width},{height}", "--use-angle=swiftshader",
         "--enable-unsafe-swiftshader", "--ignore-gpu-blocklist", "about:blank"],
        stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
    )
    try:
        for _ in range(100):
            try:
                socket.create_connection(("127.0.0.1", port), 0.2).close()
                break
            except OSError:
                time.sleep(0.1)
        agent(port, "open", url)
        result = read_settled(port)
        image = output / f"dpr-{scale}-{width}x{height}.png"
        for attempt in range(4):
            try:
                agent(port, "screenshot", str(image.resolve()))
                break
            except RuntimeError as error:
                print(f"screenshot attempt {attempt + 1}: {error}")
                time.sleep(3)
        result.update(requested=[scale, width, height], screenshot=image.name if image.exists() else None)
        problems = []
        if result["dpr"] != scale:
            problems.append("devicePixelRatio")
        if result["css"] != [result["viewport"][0], result["viewport"][1]]:
            problems.append("canvas does not fill the viewport")
        expected = [round(result["css"][0] * scale), round(result["css"][1] * scale)]
        if any(abs(a - b) > 1 for a, b in zip(result["backing"], expected)):
            problems.append(f"backing {result['backing']}, expected {expected}")
        result["problems"] = problems
        return result
    finally:
        subprocess.run(["agent-browser", "--cdp", str(port), "close"], capture_output=True, timeout=30)
        process.terminate()
        try:
            process.wait(10)
        except subprocess.TimeoutExpired:
            process.kill()
        shutil.rmtree(profile, ignore_errors=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("url")
    parser.add_argument("output", type=Path)
    parser.add_argument("--browser", default=default_browser())
    args = parser.parse_args()
    if not args.browser:
        parser.error("no Chromium found; pass --browser")
    args.output.mkdir(parents=True, exist_ok=True)
    samples = []
    for entry in SAMPLES:
        samples.append(sample(args.browser, args.url, args.output, *entry))
        last = samples[-1]
        print(last["requested"], "css", last["css"], "backing", last["backing"], last["problems"] or "ok", flush=True)
    result = {
        "scope": "WASM canvas allocation at real device scale factors; not component pixel conformance",
        "samples": samples,
    }
    (args.output / "results.json").write_text(json.dumps(result, indent=2) + "\n")
    if any(entry["problems"] for entry in samples):
        raise SystemExit(1)


if __name__ == "__main__":
    main()
