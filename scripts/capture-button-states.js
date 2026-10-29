(async () => {
  window.reference = 'c910072f3fc5b660276731d8a935d7186f288364';
  const css = document.createElement('link');
  css.rel = 'stylesheet'; css.href = '/buttons.css';
  await new Promise((resolve, reject) => {
    css.onload = resolve; css.onerror = reject; document.head.append(css);
  });
  document.body.innerHTML = '<button id="sentinel" style="position:absolute;left:600px;top:600px">Focus start</button><button id="probe" class="mtrl-button mtrl-button--filled mtrl-button--s" style="position:absolute;left:20px;top:20px;width:120px">Probe</button>';
  const el = document.getElementById('probe');
  for (const [role, value] of Object.entries({primary:'#6750a4','on-primary':'#ffffff','on-surface':'#1d1b20','on-surface-variant':'#49454f'})) el.style.setProperty('--mtrl-sys-color-' + role, value);
  const animations = () => el.getAnimations({subtree:true});
  const flush = () => getComputedStyle(el, '::before').opacity;
  // Pause actual CSS transitions as they are created, before CLI round trips
  // can let the 100ms layer transition finish. No styles or durations change.
  el.addEventListener('transitionrun', () => {
    animations().forEach(a => { a.pause(); a.currentTime = 0; });
  });
  window.finishState = () => {
    flush(); animations().forEach(a => a.finish()); flush(); return true;
  };
  window.sampleState = () => {
    flush();
    const active = animations();
    active.forEach(a => {
      a.pause();
      a.currentTime = a.transitionProperty === 'opacity' ? 50 : 125;
    });
    return {
      hovered:el.matches(':hover'), focused:el.matches(':focus-visible'),
      pressed:el.matches(':active'), disabled:el.disabled,
      animations:active.filter(a => ['opacity','box-shadow'].includes(a.transitionProperty)).map(a => {
        const pseudo = a.effect.pseudoElement || null;
        return {property:a.transitionProperty,pseudo,time:a.currentTime,
          duration:a.effect.getTiming().duration,easing:a.effect.getTiming().easing,
          frames:a.effect.getKeyframes(),
          value:getComputedStyle(el,pseudo).getPropertyValue(a.transitionProperty)};
      }),
      opacity:getComputedStyle(el,'::before').opacity,
      shadow:getComputedStyle(el).boxShadow,
    };
  };
  flush(); return true;
})()
