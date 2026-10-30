// Buttons page: per-size geometry from Type=Round, and disabled colors from
// every button and toggle set at Type=Round, Size=Medium, State=Disabled.
async () => {
  const page = figma.root.children.find(p => p.name === 'Buttons');
  await page.loadAsync();
  const variableName = async (a) => {
    if (!a) return null;
    const v = await figma.variables.getVariableByIdAsync(a.id);
    return v ? v.name : null;
  };
  const fill = async (n) => {
    if (!n || !Array.isArray(n.fills)) return null;
    for (const f of n.fills) {
      if (f.visible !== false && f.type === 'SOLID') {
        return { variable: await variableName(f.boundVariables && f.boundVariables.color), opacity: Math.round((f.opacity ?? 1) * 100) / 100 };
      }
    }
    return null;
  };
  const stroke = async (n) => {
    if (!n || !n.strokes || !n.strokes.length) return null;
    const f = n.strokes.find(s => s.visible !== false);
    if (!f) return null;
    return { variable: await variableName(f.boundVariables && f.boundVariables.color), width: n.strokeWeight };
  };
  const set = (name) => page.findOne(n => n.type === 'COMPONENT_SET' && n.name === name);
  const variant = (s, props) => s.children.find(c => Object.entries(props).every(([k, v]) => c.variantProperties[k] === v));
  const button = set('Button');
  const outline = set('Button - outline');
  const out = { sizes: {}, disabled: {} };
  for (const size of ['XSmall', 'Small', 'Medium', 'Large', 'XLarge']) {
    const round = variant(button, { Type: 'Round', Size: size, State: 'Enabled' });
    const square = variant(button, { Type: 'Square', Size: size, State: 'Enabled' });
    const pressed = variant(button, { Type: 'Round', Size: size, State: 'Pressed' });
    const layer = round.findOne(n => n.name === 'State-layer');
    const icon = round.findOne(n => n.name === 'Icon');
    const label = round.findOne(n => n.type === 'TEXT');
    const outlined = variant(outline, { Type: 'Round', Size: size, State: 'Enabled' });
    out.sizes[size] = {
      node: round.id,
      container_height: round.findOne(n => n.name === 'Content').height,
      touch_target_height: round.height,
      padding: [layer.paddingTop, layer.paddingRight, layer.paddingBottom, layer.paddingLeft],
      icon_label_gap: layer.itemSpacing,
      icon_size: icon.width,
      label_style: (await figma.getStyleByIdAsync(label.textStyleId)).name,
      shape_square: square.findOne(n => n.name === 'Content').cornerRadius,
      shape_pressed: pressed.findOne(n => n.name === 'Content').cornerRadius,
      shape_round: round.findOne(n => n.name === 'Content').cornerRadius,
      outline_width: outlined.findOne(n => n.name === 'Content').strokeWeight,
    };
  }
  const sets = {
    'filled': ['Button', {}],
    'text': ['Button - text', {}],
    'elevated': ['Button - elevated', {}],
    'outlined': ['Button - outline', {}],
    'tonal': ['Button - tonal', {}],
    'toggle-filled-unselected': ['Toggle button', { Selected: 'False' }],
    'toggle-filled-selected': ['Toggle button', { Selected: 'True' }],
    'toggle-elevated-unselected': ['Toggle button - elevated', { Selected: 'False' }],
    'toggle-elevated-selected': ['Toggle button - elevated', { Selected: 'True' }],
    'toggle-outlined-unselected': ['Toggle button - outline', { Selected: 'False' }],
    'toggle-outlined-selected': ['Toggle button - outline', { Selected: 'True' }],
    'toggle-tonal-unselected': ['Toggle button - tonal', { Selected: 'False' }],
    'toggle-tonal-selected': ['Toggle button - tonal', { Selected: 'True' }],
  };
  for (const [key, [name, extra]] of Object.entries(sets)) {
    const s = set(name);
    const c = variant(s, { Type: 'Round', Size: 'Medium', State: 'Disabled', ...extra });
    const content = c.findOne(n => n.name === 'Content');
    const layer = c.findOne(n => n.name === 'State-layer');
    const label = c.findOne(n => n.type === 'TEXT');
    const icon = c.findOne(n => n.name === 'Icon');
    out.disabled[key] = {
      set: name + ' (' + s.id + ')',
      variant: c.name,
      node: c.id,
      container: (await fill(content)) || (await fill(layer)),
      content: {
        variable: (await fill(label)).variable,
        opacity: Math.round(layer.opacity * label.opacity * 100) / 100,
        icon_opacity: Math.round(layer.opacity * icon.opacity * 100) / 100,
      },
      outline: await stroke(content),
    };
  }
  return JSON.stringify(out);
}
