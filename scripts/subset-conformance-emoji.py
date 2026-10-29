#!/usr/bin/env python3
"""Reproduce the color-emoji probe subset: uv run --no-project --with fonttools==4.60.1 python scripts/subset-conformance-emoji.py."""
from fontTools.ttLib import TTFont
from fontTools import subset
import fontTools
from pathlib import Path
import json, hashlib, urllib.request, io
root = Path(__file__).resolve().parents[1] / 'crates/material/examples/gallery/fonts/conformance/notocoloremoji'
p = root / 'SOURCES.json'
metadata = json.loads(p.read_text())
source = next(row for row in metadata['files'] if row['file'] == 'NotoColorEmoji.ttf')
data = urllib.request.urlopen(source['source']).read()
assert hashlib.sha256(data).hexdigest() == source['sha256']
font = TTFont(io.BytesIO(data))
options = subset.Options()
options.recalc_timestamp = False
sub = subset.Subsetter(options=options)
text = '👩\u200d👧👦 family☀️👍🏽'
sub.populate(text=text)
sub.subset(font)
out = root / 'NotoColorEmoji-Probe.ttf'
font.save(out)
metadata['subset'] = {'file': out.name, 'sha256': hashlib.sha256(out.read_bytes()).hexdigest(), 'tool': 'fonttools ' + fontTools.version, 'text': text, 'recalc_timestamp': False, 'source_file': 'NotoColorEmoji.ttf', 'purpose': 'Probe sequence coverage only. Preserves CBDT/CBLC/GSUB and original PostScript name.'}
p.write_text(json.dumps(metadata, indent=2) + '\n')
print(out, out.stat().st_size)
