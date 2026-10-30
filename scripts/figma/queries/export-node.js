// One top-level node of `params.page` as "name|WxH|scale|base64 PNG".
async (params) => {
  const page = figma.root.children.find(p => p.name === params.page);
  await page.loadAsync();
  const c = page.children[params.index];
  const scale = Math.min(2, params.maxDim / Math.max(c.width, c.height));
  const png = await c.exportAsync({ format: 'PNG', constraint: { type: 'SCALE', value: scale } });
  return c.name + '|' + Math.round(c.width) + 'x' + Math.round(c.height) + '|' + scale.toFixed(3) + '|' + figma.base64Encode(png);
}
