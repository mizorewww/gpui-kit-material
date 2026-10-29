#!/usr/bin/env python3
"""Repackage identical probe PNG strikes as sbix for CoreText. Run with fonttools==4.60.1."""
from pathlib import Path
import hashlib,json,fontTools,math
from fontTools.ttLib import TTFont,newTable
from fontTools.ttLib.tables.sbixGlyph import Glyph
from fontTools.ttLib.tables.sbixStrike import Strike
from fontTools.pens.ttGlyphPen import TTGlyphPen
root=Path(__file__).resolve().parents[1]/'crates/material/examples/gallery/fonts/conformance/notocoloremoji'
source=root/'NotoColorEmoji-Probe.ttf';font=TTFont(source);font.recalcTimestamp=False
sbix=newTable('sbix');sbix.version=1;sbix.flags=1;sbix.strikes={}
for index,original in enumerate(font['CBLC'].strikes):
    strike=Strike(ppem=original.bitmapSizeTable.ppemY,resolution=72)
    strike.glyphs={name:Glyph(glyphName=name) for name in font.getGlyphOrder()}
    for name,bitmap in font['CBDT'].strikeData[index].items():
        metrics=bitmap.metrics
        strike.glyphs[name]=Glyph(glyphName=name,originOffsetX=metrics.BearingX,originOffsetY=metrics.BearingY-metrics.height,graphicType='png ',imageData=bitmap.imageData)
    sbix.strikes[strike.ppem]=strike
font['sbix']=sbix
# CoreText expects a TrueType bounds scaffold alongside sbix. It leaves the
# original cmap, GSUB, advances and bitmap pixel data unchanged.
glyf=newTable('glyf');glyf.glyphs={name:TTGlyphPen(None).glyph() for name in font.getGlyphOrder()}
# CoreText obtains glyph bounds from glyf even when it paints sbix pixels.
# A two-point closed contour has zero filled area but supplies the original
# bitmap bounds, avoiding an empty atlas rectangle without adding visible ink.
for index,strike in enumerate(font['CBLC'].strikes):
    factor=font['head'].unitsPerEm/strike.bitmapSizeTable.ppemY
    for name,bitmap in font['CBDT'].strikeData[index].items():
        m=bitmap.metrics;x0=math.floor(m.BearingX*factor);y0=math.floor((m.BearingY-m.height)*factor)
        x1=math.ceil((m.BearingX+m.width)*factor);y1=math.ceil(m.BearingY*factor)
        pen=TTGlyphPen(None);pen.moveTo((x0,y0));pen.lineTo((x1,y1));pen.closePath();glyf.glyphs[name]=pen.glyph()
font['glyf']=glyf;font['loca']=newTable('loca')
font['maxp'].tableVersion=0x10000
for field in ['maxPoints','maxContours','maxCompositePoints','maxCompositeContours','maxTwilightPoints','maxStorage','maxFunctionDefs','maxInstructionDefs','maxStackElements','maxSizeOfInstructions','maxComponentElements','maxComponentDepth']:setattr(font['maxp'],field,0)
font['maxp'].maxZones=1
# One bitmap representation per font makes renderer capability explicit.
del font['CBDT'];del font['CBLC']
out=root/'NotoColorEmoji-Probe-sbix.ttf';font.save(out)
original=TTFont(source);converted=TTFont(out)
assert original.getGlyphOrder()==converted.getGlyphOrder()
assert original.getBestCmap()==converted.getBestCmap()
assert original['hmtx'].metrics==converted['hmtx'].metrics
assert original['GSUB'].compile(original)==converted['GSUB'].compile(converted)
for index,strike in enumerate(original['CBLC'].strikes):
    for name,bitmap in original['CBDT'].strikeData[index].items():
        assert converted['sbix'].strikes[strike.bitmapSizeTable.ppemY].glyphs[name].imageData==bitmap.imageData
p=root/'SOURCES.json';metadata=json.loads(p.read_text());metadata['sbix']={'file':out.name,'sha256':hashlib.sha256(out.read_bytes()).hexdigest(),'source_file':source.name,'source_sha256':hashlib.sha256(source.read_bytes()).hexdigest(),'tool':'fonttools '+fontTools.version,'conversion':'Same PNG payloads and strike ppem; sbix origins = CBDT BearingX, BearingY-height; zero-area TrueType bitmap-bounds scaffold; cmap/GSUB/hmtx retained; recalcTimestamp=False'};p.write_text(json.dumps(metadata,indent=2)+'\n');print(out,out.stat().st_size)
