#!/usr/bin/env python3
"""Rebuild gallery TTFs from the pinned md3.io WOFF2.

Run: uv run --with 'fonttools[woff]==4.60.1' python scripts/build-reference-fonts.py
"""
import hashlib
import json
from pathlib import Path

from fontTools.ttLib import TTFont
from fontTools.varLib.instancer import instantiateVariableFont

root = Path(__file__).resolve().parents[1] / 'crates/material/examples/gallery/fonts'
sources = json.loads((root / 'SOURCES.json').read_text())
source = root / 'roboto/Roboto-latin-wght.woff2'
expected = sources['roboto/Roboto-latin-wght.woff2']['sha256']
if hashlib.sha256(source.read_bytes()).hexdigest() != expected:
    raise SystemExit('Reference WOFF2 hash mismatch')
for weight, name in [(400, 'Regular'), (500, 'Medium'), (600, 'SemiBold')]:
    font = TTFont(source, recalcTimestamp=False)
    font = instantiateVariableFont(font, {'wght': weight}, inplace=True, updateFontNames=True)
    font.flavor = None
    target = root / f'roboto/Roboto-{name}.ttf'
    font.save(target)
    digest = hashlib.sha256(target.read_bytes()).hexdigest()
    if digest != sources[f'roboto/Roboto-{name}.ttf']['sha256']:
        raise SystemExit(f'{target.name}: unexpected output hash {digest}')
    print(f'{target.name}: {digest}')
