(() => {
const roles = ["error", "error-container", "info", "inverse-on-surface", "inverse-primary", "inverse-surface", "on-error", "on-error-container", "on-info", "on-primary", "on-primary-container", "on-primary-fixed", "on-primary-fixed-variant", "on-secondary", "on-secondary-container", "on-secondary-fixed", "on-secondary-fixed-variant", "on-success", "on-surface", "on-surface-variant", "on-tertiary", "on-tertiary-container", "on-tertiary-fixed", "on-tertiary-fixed-variant", "on-warning", "outline", "outline-variant", "primary", "primary-container", "primary-fixed", "primary-fixed-dim", "scrim", "secondary", "secondary-container", "secondary-fixed", "secondary-fixed-dim", "shadow", "success", "surface", "surface-bright", "surface-container", "surface-container-high", "surface-container-highest", "surface-container-low", "surface-container-lowest", "surface-dim", "surface-variant", "tertiary", "tertiary-container", "tertiary-fixed", "tertiary-fixed-dim", "warning"], names = ["autumn", "baseline", "brownbeige", "content", "desert", "expressive", "fidelity", "forest", "fruit-salad", "highcontrast", "monochrome", "neutral", "ocean", "rainbow", "sageivory", "spring", "summer", "sunset", "tealcaramel", "vibrant"], rows = [];
const root=document.documentElement;
for (const theme of names) for (const mode of ['light','dark']) for (const contrast of ['standard','medium','high']) {
 root.dataset.theme=theme; root.dataset.themeMode=mode; root.dataset.themeContrast=contrast;
 const style=getComputedStyle(root);
 const colors=Object.fromEntries(roles.map(role=>[role,style.getPropertyValue('--mtrl-sys-color-'+role).trim().toLowerCase()]));
 if(Object.values(colors).some(color=>!/^#[0-9a-f]{6}$/.test(color))) throw new Error('Missing or unexpected color '+theme+' '+mode+' '+contrast);
 rows.push({theme,mode,contrast,colors});
}
return {source_commit:'c910072f3fc5b660276731d8a935d7186f288364',sass_version:'1.85.1',browser:navigator.userAgent,roles,rows};
})()
