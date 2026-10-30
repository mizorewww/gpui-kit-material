// Local text and effect styles, plus every variable collection except M3
// (m3-colors.js resolves that one per mode).
async () => {
  const out = {};
  const variableName = async (alias) => {
    if (!alias) return null;
    try {
      const v = await figma.variables.getVariableByIdAsync(alias.id);
      return v ? v.name : null;
    } catch (e) {
      return null;
    }
  };
  out.textStyles = [];
  for (const s of await figma.getLocalTextStylesAsync()) {
    const bound = {};
    for (const k of Object.keys(s.boundVariables || {})) bound[k] = await variableName(s.boundVariables[k]);
    out.textStyles.push({ name: s.name, id: s.id, key: s.key, family: s.fontName.family, style: s.fontName.style, size: s.fontSize, lh: s.lineHeight, ls: s.letterSpacing, bound });
  }
  const hex = c => '#' + [c.r, c.g, c.b].map(x => Math.round(x * 255).toString(16).padStart(2, '0')).join('');
  out.effectStyles = (await figma.getLocalEffectStylesAsync()).map(s => ({
    name: s.name,
    id: s.id,
    key: s.key,
    effects: s.effects.map(e => ({ type: e.type, x: e.offset && e.offset.x, y: e.offset && e.offset.y, r: e.radius, s: e.spread, c: e.color && (hex(e.color) + '@' + e.color.a.toFixed(3)) })),
  }));
  out.vars = {};
  for (const c of await figma.variables.getLocalVariableCollectionsAsync()) {
    if (c.name === 'M3') continue;
    const rows = [];
    for (const id of c.variableIds) {
      const v = await figma.variables.getVariableByIdAsync(id);
      const vals = {};
      for (const m of c.modes) {
        let x = v.valuesByMode[m.modeId];
        if (x && x.type === 'VARIABLE_ALIAS') x = 'alias:' + (await variableName(x));
        vals[m.name] = x;
      }
      rows.push({ name: v.name, type: v.resolvedType, vals });
    }
    out.vars[c.name] = rows;
  }
  return JSON.stringify(out);
}
