(async () => {
  if (!document.querySelector('link[data-probe]')) {
    const css = document.createElement('link'); css.rel = 'stylesheet'; css.href = '/buttons.css'; css.dataset.probe = '1';
    await new Promise((resolve, reject) => {
      css.onload = resolve;
      css.onerror = () => reject(new Error('Failed to load compiled reference styles'));
      document.head.append(css);
    });
  }
  document.body.innerHTML = '<button id="probe" class="mtrl-button mtrl-button--filled mtrl-button--s">Probe</button>';
  const el = document.getElementById('probe');
  for (const [role, value] of Object.entries({primary:'#6750a4','on-primary':'#ffffff','secondary-container':'#e8def8','on-secondary-container':'#1d192b','on-surface':'#1d1b20','on-surface-variant':'#49454f'})) el.style.setProperty('--mtrl-sys-color-' + role, value);
  const flush = () => getComputedStyle(el).backgroundColor;
  const finish = () => { flush(); el.getAnimations().forEach(a => a.finish()); flush(); };
  const sample = (time) => {
    flush(); const animations = el.getAnimations(); animations.forEach(a => {a.pause(); a.currentTime=time;});
    return Object.fromEntries(animations.map(a => [a.transitionProperty, {time:a.currentTime, duration:a.effect.getTiming().duration, easing:a.effect.getTiming().easing, frames:a.effect.getKeyframes(), value:getComputedStyle(el).getPropertyValue(a.transitionProperty)}]));
  };
  finish(); el.classList.replace('mtrl-button--filled','mtrl-button--tonal');
  const variant = sample(125); finish(); el.classList.replace('mtrl-button--tonal','mtrl-button--filled'); finish();
  el.disabled = true; const disabled = sample(125);
  el.disabled = false; const reversed = sample(25);
  return {reference:'c910072f3fc5b660276731d8a935d7186f288364',ua:navigator.userAgent,variant,disabled,reversed};
})()
