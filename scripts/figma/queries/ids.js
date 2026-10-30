// File identity plus the collection, style, page and component-set ids the
// fixtures cite, so a later Kit release shows up as a changed id.
async () => {
  const collections = await figma.variables.getLocalVariableCollectionsAsync();
  const out = {
    fileKey: figma.fileKey,
    root: figma.root.name,
    colls: collections.map(c => ({ name: c.name, id: c.id, key: c.key, modes: c.modes })),
  };
  const styles = await figma.getLocalTextStylesAsync();
  out.text = styles.slice(0, 2).map(s => ({ name: s.name, id: s.id, key: s.key }));
  const effects = await figma.getLocalEffectStylesAsync();
  out.effect = effects.map(s => ({ name: s.name, id: s.id }));
  const buttons = figma.root.children.find(p => p.name === 'Buttons');
  await buttons.loadAsync();
  const sets = buttons.findAll(n => n.type === 'COMPONENT_SET' && /^(Button|Toggle button)/.test(n.name));
  out.buttonSets = sets.map(s => ({ name: s.name, id: s.id }));
  out.stylesPage = figma.root.children.find(p => p.name === 'Styles').id;
  return JSON.stringify(out);
}
