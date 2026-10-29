(async () => {
  const rect = el => {
    if (!el) return null;
    const r = el.getBoundingClientRect();
    return { x: r.x, y: r.y, width: r.width, height: r.height };
  };
  const snapshot = button => {
    const el = button.element;
    const css = getComputedStyle(el);
    const progress = el.querySelector('.mtrl-button__progress');
    const canvas = el.querySelector('canvas');
    const icon = el.querySelector('.mtrl-button__icon');
    return {
      rect: rect(el), label: button.getText(), disabled: el.disabled,
      busy: el.getAttribute('aria-busy'), classes: el.className,
      padding: [css.paddingLeft, css.paddingRight], gap: css.gap,
      children: [...el.children].map(child => ({ classes: child.className, display: getComputedStyle(child).display })),
      icon: rect(icon), progress: rect(progress), canvas: rect(canvas),
      progress_opacity: progress && getComputedStyle(progress).opacity,
      canvas_style: canvas && [canvas.style.width, canvas.style.height],
      progress_role: progress && progress.getAttribute('role'),
      progress_value: progress && progress.getAttribute('aria-valuenow')
    };
  };
  const rows = [];
  for (const variant of ['filled', 'elevated', 'tonal', 'outlined', 'text']) {
    for (const size of ['xs', 's', 'm', 'l', 'xl']) {
      for (const placement of ['none', 'start', 'end']) {
        const config = { text: 'Save changes', variant, size, progress: true, ripple: false };
        if (placement !== 'none') Object.assign(config, {icon: '<svg viewBox="0 0 24 24"><path d="M2 12h20"/></svg>', iconPosition: placement});
        const button = window.referenceCreateButton(config);
        document.body.replaceChildren(button.element);
        const before = snapshot(button);
        await button.showProgress();
        const shown = snapshot(button);
        await button.setLoading(true, 'Saving changes');
        for (const animation of document.getAnimations()) animation.finish();
        const loading = snapshot(button);
        await button.setLoading(true, 'Ignored second call');
        const repeated = snapshot(button);
        await button.setLoading(false);
        const restored = snapshot(button);
        rows.push({variant, size, placement, before, shown, loading, repeated, restored});
        button.destroy();
      }
    }
  }
  const initiallyDisabled = window.referenceCreateButton({text:'Save', disabled:true, progress:true});
  document.body.replaceChildren(initiallyDisabled.element);
  await initiallyDisabled.setLoading(true);
  await initiallyDisabled.setLoading(false);
  const afterInitiallyDisabled = snapshot(initiallyDisabled);
  initiallyDisabled.destroy();
  return { reduced_motion: matchMedia('(prefers-reduced-motion: reduce)').matches,
    rows, after_initially_disabled: afterInitiallyDisabled,
    motion: [0,75,150,300,750,1499,1500,1650,1800,3000,4500,5999,6000].map(time => ({time, ...window.referenceCircularFrame(time)})) };
})()
