// The M3 collection's variables in the six baseline modes, with aliases
// resolved per mode into 8-bit sRGB hex ("@alpha" when translucent).
async () => {
  const collection = (await figma.variables.getLocalVariableCollectionsAsync()).find(c => c.name === 'M3');
  const hex = x => '#' + [x.r, x.g, x.b].map(v => Math.round(v * 255).toString(16).padStart(2, '0')).join('') + (x.a !== undefined && x.a < 1 ? '@' + x.a.toFixed(3) : '');
  const want = ['Light', 'Dark', 'Light High Contrast', 'Light Medium Contrast', 'Dark High Contrast', 'Dark Medium Contrast'];
  const modes = collection.modes.filter(m => want.includes(m.name));
  const rows = [];
  for (const id of collection.variableIds) {
    const v = await figma.variables.getVariableByIdAsync(id);
    const vals = {};
    for (const m of modes) {
      let x = v.valuesByMode[m.modeId];
      for (let depth = 0; x && x.type === 'VARIABLE_ALIAS' && depth < 6; depth++) {
        const target = await figma.variables.getVariableByIdAsync(x.id);
        const owner = await figma.variables.getVariableCollectionByIdAsync(target.variableCollectionId);
        const mode = owner.modes.find(q => q.name === m.name) || owner.modes[0];
        x = target.valuesByMode[mode.modeId];
      }
      vals[m.name] = (x && typeof x === 'object' && 'r' in x) ? hex(x) : x;
    }
    rows.push({ name: v.name, type: v.resolvedType, vals });
  }
  return JSON.stringify(rows);
}
