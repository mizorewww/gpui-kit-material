// Names, variants and text of `params.page`, depth-limited.
async (params) => {
  const page = figma.root.children.find(p => p.name === params.page);
  await page.loadAsync();
  const lines = [];
  const walk = (n, d) => {
    if (d > 5 || lines.length > 1500) return;
    const text = n.type === 'TEXT' ? ' "' + n.characters.slice(0, 80).replace(/\n/g, ' ') + '"' : '';
    const variants = n.type === 'COMPONENT_SET' ? ' variants=' + Object.keys(n.componentPropertyDefinitions || {}).join(',') : '';
    const main = n.type === 'INSTANCE' && n.mainComponent ? ' ->' + n.mainComponent.name : '';
    lines.push('  '.repeat(d) + n.type + ' ' + n.name + ' ' + Math.round(n.width) + 'x' + Math.round(n.height) + text + variants + main);
    if ('children' in n && n.type !== 'INSTANCE') n.children.forEach(c => walk(c, d + 1));
  };
  page.children.forEach(c => walk(c, 0));
  return lines.join('\n');
}
