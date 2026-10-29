#!/usr/bin/env python3
"""Freeze shared probe fallback fonts: uv run --no-project --with fonttools==4.60.1 python scripts/freeze-conformance-fonts.py."""
from fontTools.ttLib import TTFont
from fontTools.varLib.instancer import instantiateVariableFont
from pathlib import Path
import json,hashlib,fontTools
root=Path(__file__).resolve().parents[1]/'crates/material/examples/gallery/fonts/conformance'
p=root/'SOURCES.json';data=json.loads(p.read_text());derived=[]
for folder,name in [('notosansarabic','NotoSansArabic'),('notosanshebrew','NotoSansHebrew'),('notoemoji','NotoEmoji')]:
 source=next(path for path in (root/folder).glob('*.ttf') if '[' in path.name)
 font=TTFont(source);font.recalcTimestamp=False
 axes={axis.axisTag:400 if axis.axisTag=='wght' else axis.defaultValue for axis in font['fvar'].axes}
 static=instantiateVariableFont(font,axes,inplace=False,updateFontNames=True)
 out=root/folder/(name+'-Regular.ttf');static.save(out)
 derived.append({'file':str(out.relative_to(root)),'source':str(source.relative_to(root)),'axes':axes,'sha256':hashlib.sha256(out.read_bytes()).hexdigest(),'tool':'fonttools '+fontTools.version,'recalcTimestamp':False})
data['derived']=derived;p.write_text(json.dumps(data,indent=2)+'\n')
