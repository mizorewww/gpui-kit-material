#!/usr/bin/env python3
"""Extract the pinned m3e.kyralabs.dev design tokens into crate fixtures.

Usage: python3 -I scripts/extract-m3e-tokens.py /path/to/m3exp-web

The checkout must be kyra-labs/m3exp-web (the source of m3e.kyralabs.dev) at
``REVISION`` and clean under ``packages/m3e-tokens``. Only committed generated
token files are read; nothing is built or executed. Writes:

- conformance/fixtures/m3e-motion.json (scheme springs, Compose settle
  durations, the CSS linear() stops and reduced-motion overrides)
- crates/material/tests/fixtures/m3e-typography.json (15 roles, base and
  emphasized)
- crates/material/tests/fixtures/m3e-sys-tokens.json (shape, elevation, state
  layers and the button size / disabled tokens)
"""
import argparse
import json
from pathlib import Path
import re
import subprocess

REPOSITORY = "https://github.com/kyra-labs/m3exp-web"
REVISION = "3b1e0047c0021eab7b4c0128fa1bec95c76ca746"
SITE = "https://m3e.kyralabs.dev/"
TOKENS = "packages/m3e-tokens/src/generated/tokens.css"
COMPONENTS = "packages/m3e-tokens/src/generated/components"
ROOT = Path(__file__).resolve().parent.parent

ROLES = [
    "display-large", "display-medium", "display-small",
    "headline-large", "headline-medium", "headline-small",
    "title-large", "title-medium", "title-small",
    "body-large", "body-medium", "body-small",
    "label-large", "label-medium", "label-small",
]
SPEEDS = ["fast", "default", "slow"]
KINDS = ["spatial", "effects"]
BUTTON_SIZES = ["x-small", "small", "medium", "large", "x-large"]
BUTTON_VARIANTS = ["filled-button", "elevated-button", "filled-tonal-button",
                   "tonal-button", "outlined-button", "text-button"]


def git(source, *args):
    return subprocess.run(["git", "-C", str(source), *args], text=True,
                          capture_output=True, check=True, timeout=60).stdout


def declarations(path):
    """Custom properties in declaration order; later blocks override earlier ones."""
    return declarations_in(path.read_text())


def declarations_in(text):
    blocks = []
    for selector, body in re.findall(r"([^{}]+)\{([^{}]*)\}", text):
        values = dict(re.findall(r"(--[a-z0-9-]+):\s*([^;]+);", body))
        if values:
            blocks.append((" ".join(selector.split()), values))
    return blocks


def root_tokens(path):
    """Unconditional :root tokens (the reduced-motion media block is excluded)."""
    text = re.sub(r"@media[^{]*\{\s*:root\s*\{[^{}]*\}\s*\}", "", path.read_text())
    tokens = {}
    for selector, values in declarations_in(text):
        if selector.endswith(":root"):
            tokens.update(values)
    return tokens


def reduced_motion_tokens(path):
    text = path.read_text()
    match = re.search(r"prefers-reduced-motion:\s*reduce\)\s*\{\s*:root\s*\{([^{}]*)\}", text)
    if not match:
        raise ValueError("no reduced-motion block in " + str(path))
    return dict(re.findall(r"(--[a-z0-9-]+):\s*([^;]+);", match.group(1)))


def resolve(tokens, value):
    for _ in range(8):
        match = re.fullmatch(r"var\((--[a-z0-9-]+)\)", value.strip())
        if not match:
            return value.strip()
        value = tokens[match.group(1)]
    raise ValueError("unresolved " + value)


def px(value):
    """CSS length to design px (1rem = 16px in the reference)."""
    value = value.strip()
    if value == "0":
        return 0.0
    if value.startswith("calc(infinity"):
        return "infinity"
    if value.endswith("rem"):
        return round(float(value[:-3]) * 16, 6)
    if value.endswith("px"):
        return float(value[:-2])
    raise ValueError("not a length: " + value)


def ms(value):
    assert value.endswith("ms"), value
    return int(value[:-2])


def linear_stops(value):
    body = re.fullmatch(r"linear\((.*)\)", value.strip()).group(1)
    parts = [part.strip() for part in body.split(",")]
    stops = []
    for index, part in enumerate(parts):
        fields = part.split()
        if len(fields) == 1:
            progress = 0.0 if index == 0 else 1.0
        else:
            progress = round(float(fields[1].rstrip("%")) / 100, 6)
        stops.append([progress, float(fields[0])])
    return stops


def motion(tokens, reduced):
    springs = {}
    for scheme in ("standard", "expressive"):
        entries = []
        for speed in SPEEDS:
            for kind in KINDS:
                prefix = f"--m3e-sys-motion-spring-{scheme}-{speed}-{kind}"
                entries.append({
                    "token": f"{speed}-{kind}",
                    "damping_ratio": float(tokens[prefix + "-damping-ratio"]),
                    "stiffness": float(tokens[prefix + "-stiffness"]),
                    "duration_ms": ms(tokens[prefix + "-duration"]),
                    "linear_stops": linear_stops(tokens[prefix + "-easing"]),
                })
        springs[scheme] = entries
    reduced_motion = {}
    for speed in SPEEDS:
        for kind in KINDS:
            key = f"--m3e-sys-motion-spring-expressive-{speed}-{kind}-duration"
            value = reduced.get(key)
            if value is None:
                reduced_motion[f"{speed}-{kind}"] = {"unchanged": True}
            elif value.startswith("var("):
                token = re.search(r"expressive-([a-z]+-[a-z]+)-duration", value).group(1)
                reduced_motion[f"{speed}-{kind}"] = {"replaced_by": token}
            else:
                reduced_motion[f"{speed}-{kind}"] = {"duration_ms": ms(value)}
    return {
        "unit": "durations in ms; linear_stops are [progress 0..1, value] pairs",
        "duration_definition": "Compose FloatSpringSpec.getDurationNanos for 0 -> 1 at a 0.001 visibility threshold, truncated to whole ms (packages/m3e-motion/src/linear.ts)",
        "springs": springs,
        "reduced_motion": reduced_motion,
        "reduced_motion_scope": "@media (prefers-reduced-motion: reduce) in tokens.css; identical overrides apply to the standard scheme",
    }


def typography(tokens):
    def role(name, suffix):
        prefix = f"--m3e-sys-typescale-{name}{suffix}"
        font = tokens[prefix + "-font"]
        typeface = "brand" if "brand" in font else "plain"
        return {
            "name": name,
            "typeface": typeface,
            "font_size": px(tokens[prefix + "-size"]),
            "line_height": px(tokens[prefix + "-line-height"]),
            "letter_spacing": px(tokens[prefix + "-tracking"]),
            "font_weight": int(resolve(tokens, tokens[prefix + "-weight"])),
        }
    return {
        "unit": "design px (reference 1rem = 16px); weights are CSS font-weight",
        "font_family": {
            "brand": resolve(tokens, "var(--m3e-ref-typeface-brand)"),
            "plain": resolve(tokens, "var(--m3e-ref-typeface-plain)"),
            "note": "Google Sans Flex is m3exp-web decision D1; Compose TypefaceTokens default is FontFamily.SansSerif",
        },
        "roles": [role(name, "") for name in ROLES],
        "emphasized": [role(name, "-emphasized") for name in ROLES],
    }


def component(source, name):
    tokens = {}
    for selector, values in declarations(source / COMPONENTS / f"{name}.css"):
        if selector.endswith(":root"):
            tokens.update(values)
    return tokens


def sys_tokens(source, tokens):
    shape = {key.removeprefix("--m3e-sys-shape-corner-value-"): px(value)
             for key, value in tokens.items() if key.startswith("--m3e-sys-shape-corner-value-")}
    shape["full"] = px(tokens["--m3e-sys-shape-corner-full"])
    elevation = []
    for level in range(6):
        shadow = tokens[f"--m3e-sys-elevation-level{level}-shadow"]
        layers = []
        for x, y, blur, spread, opacity in re.findall(
                r"(-?\d+)(?:px)? (-?\d+)(?:px)? (-?\d+)(?:px)? (-?\d+)(?:px)? color-mix\(in srgb, var\(--m3e-sys-color-shadow\) (\d+)%", shadow):
            layers.append({"x": float(x), "y": float(y), "blur": float(blur),
                           "spread": float(spread), "opacity": int(opacity) / 100})
        elevation.append({"level": level, "dp": px(tokens[f"--m3e-sys-elevation-level{level}"]),
                          "layers": layers})
    state = {key.removeprefix("--m3e-sys-state-").removesuffix("-state-layer-opacity"): float(value)
             for key, value in tokens.items() if key.endswith("-state-layer-opacity")}
    buttons = {}
    for size in BUTTON_SIZES:
        values = component(source, f"button-{size}")
        prefix = f"--m3e-comp-button-{size}-"
        buttons[size] = {
            "container_height": px(values[prefix + "container-height"]),
            "icon_size": px(values[prefix + "icon-size"]),
            "icon_label_space": px(values[prefix + "icon-label-space"]),
            "leading_space": px(values[prefix + "leading-space"]),
            "trailing_space": px(values[prefix + "trailing-space"]),
            "outline_width": px(values[prefix + "outlined-outline-width"]),
            "shape_square": resolve(values | tokens, values[prefix + "container-shape-square"]),
            "shape_pressed": resolve(values | tokens, values[prefix + "container-shape-pressed"]),
        }
    defaults = component(source, "button-defaults")
    disabled = {}
    for variant in BUTTON_VARIANTS:
        values = component(source, variant)
        name = variant.removesuffix("-button")
        prefix = f"--m3e-comp-{variant}-"
        label = "label-" if variant == "text-button" else "label-text-"
        disabled[name] = {
            "container_color": values.get(prefix + "container-color-disabled", "").removeprefix("var(--m3e-sys-color-").rstrip(")") or None,
            "container_opacity": float(values[prefix + "container-opacity-disabled"]),
            "content_color": values[prefix + label + "color-disabled"].removeprefix("var(--m3e-sys-color-").rstrip(")"),
            "content_opacity": float(values[prefix + label + "opacity-disabled"]),
        }
    return {
        "shape_corner_px": dict(sorted(shape.items())),
        "elevation": elevation,
        "state_layer_opacity": state,
        "button": {
            "sizes": buttons,
            "defaults": {key.removeprefix("--m3e-comp-"): px(value)
                         for key, value in defaults.items()},
            "disabled": disabled,
            "variant_mapping": "packages/m3e-css/src/button.css: --tonal uses filled-tonal-button (FilledTonalButtonTokens) except as a toggle (tonal-button); --text and --outlined paint a transparent disabled container",
        },
    }


def provenance(paths):
    return {
        "site": SITE,
        "repository": REPOSITORY,
        "source_commit": REVISION,
        "source_paths": paths,
        "upstream": "Generated by @kyralabs/m3e-codegen from compose/material3 tokens @ androidx 19991f2c0858aadfcb442ce02b2c8e4743c96898 (Compose Material3 1.5.0-beta01); see SOURCE_PIN and SOURCES.md",
        "generator": "scripts/extract-m3e-tokens.py",
    }


def write(path, data):
    path.write_text(json.dumps(data, indent=2) + "\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path)
    args = parser.parse_args()
    source = args.source.resolve()
    if git(source, "rev-parse", "HEAD").strip() != REVISION:
        parser.error("the m3exp-web checkout must be at " + REVISION)
    if git(source, "status", "--porcelain", "--", "packages/m3e-tokens"):
        parser.error("packages/m3e-tokens must be clean")
    tokens = root_tokens(source / TOKENS)
    reduced = reduced_motion_tokens(source / TOKENS)
    write(ROOT / "conformance/fixtures/m3e-motion.json",
          provenance([TOKENS, "packages/m3e-motion/src/linear.ts"]) | motion(tokens, reduced))
    write(ROOT / "crates/material/tests/fixtures/m3e-typography.json",
          provenance([TOKENS]) | typography(tokens))
    component_paths = [f"{COMPONENTS}/button-{size}.css" for size in BUTTON_SIZES]
    component_paths += [f"{COMPONENTS}/button-defaults.css"]
    component_paths += [f"{COMPONENTS}/{variant}.css" for variant in BUTTON_VARIANTS]
    write(ROOT / "crates/material/tests/fixtures/m3e-sys-tokens.json",
          provenance([TOKENS, *component_paths, "packages/m3e-css/src/button.css"])
          | sys_tokens(source, tokens))


if __name__ == "__main__":
    main()
