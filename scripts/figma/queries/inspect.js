// Dumps the first node on `params.page` whose name contains `params.query`:
// sizes, fills and strokes with resolved variable names, radius, auto layout,
// opacity, effects, typography and component properties.
async (params) => {
  const page = figma.root.children.find(p => p.name === params.page);
  if (!page) return 'page not found';
  await page.loadAsync();
  const node = page.findOne(x => x.name.includes(params.query));
  if (!node) return 'node not found';
  const variableName = async (a) => {
    if (!a) return '';
    try {
      const v = await figma.variables.getVariableByIdAsync(a.id);
      return v ? v.name : '';
    } catch (e) {
      return '';
    }
  };
  const hex = c => '#' + [c.r, c.g, c.b].map(x => Math.round(x * 255).toString(16).padStart(2, '0')).join('');
  const paints = async (ps) => {
    if (!ps || ps === figma.mixed) return '';
    const o = [];
    for (const f of ps) {
      if (f.visible === false) continue;
      let s = f.type === 'SOLID' ? hex(f.color) + (f.opacity !== undefined && f.opacity < 1 ? '@' + f.opacity.toFixed(2) : '') : f.type;
      const v = await variableName(f.boundVariables && f.boundVariables.color);
      if (v) s += '{' + v + '}';
      o.push(s);
    }
    return o.join(',');
  };
  const lines = [];
  const walk = async (n, d) => {
    if (d > params.depth || lines.length > 2500) return;
    let s = '  '.repeat(d) + n.type + ' "' + n.name + '" ' + Math.round(n.width) + 'x' + Math.round(n.height);
    if ('fills' in n) { const f = await paints(n.fills); if (f) s += ' fill=' + f; }
    if ('strokes' in n && n.strokes.length) s += ' stroke=' + (await paints(n.strokes)) + '/' + (n.strokeWeight === figma.mixed ? 'mixed' : n.strokeWeight) + (n.strokeAlign ? ':' + n.strokeAlign : '');
    if ('cornerRadius' in n && n.cornerRadius !== 0) s += ' radius=' + (n.cornerRadius === figma.mixed ? [n.topLeftRadius, n.topRightRadius, n.bottomRightRadius, n.bottomLeftRadius].join('/') : n.cornerRadius);
    if (n.layoutMode && n.layoutMode !== 'NONE') s += ' layout=' + n.layoutMode + ' pad=' + [n.paddingTop, n.paddingRight, n.paddingBottom, n.paddingLeft].join('/') + ' gap=' + n.itemSpacing + ' align=' + n.primaryAxisAlignItems + '/' + n.counterAxisAlignItems;
    if ('opacity' in n && n.opacity < 1) s += ' opacity=' + n.opacity.toFixed(2);
    if ('effects' in n && n.effects.length) s += ' effects=' + n.effects.filter(e => e.visible !== false).map(e => e.type + (e.offset ? '(' + e.offset.x + ',' + e.offset.y + ' r' + e.radius + ' s' + (e.spread || 0) + ' ' + hex(e.color) + '@' + e.color.a.toFixed(2) + ')' : '')).join(';');
    if (n.type === 'TEXT') {
      const font = n.fontName === figma.mixed ? 'mixed' : n.fontName.family + ' ' + n.fontName.style;
      const style = n.textStyleId && n.textStyleId !== figma.mixed ? (await figma.getStyleByIdAsync(n.textStyleId))?.name : '';
      s += ' text="' + n.characters.slice(0, 60).replace(/\n/g, ' ') + '" font=' + font + ' size=' + String(n.fontSize) + ' lh=' + JSON.stringify(n.lineHeight) + ' ls=' + JSON.stringify(n.letterSpacing) + (style ? ' style=' + style : '');
    }
    if (n.type === 'INSTANCE' && n.mainComponent) s += ' ->' + n.mainComponent.name;
    if (n.type === 'COMPONENT_SET') s += ' props=' + JSON.stringify(n.componentPropertyDefinitions);
    lines.push(s);
    if ('children' in n) for (const c of n.children) await walk(c, d + 1);
  };
  await walk(node, 0);
  return lines.join('\n');
}
