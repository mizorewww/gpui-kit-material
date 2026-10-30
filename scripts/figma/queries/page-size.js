// How many top-level nodes `params.page` has.
async (params) => {
  const page = figma.root.children.find(p => p.name === params.page);
  await page.loadAsync();
  return page.children.length;
}
