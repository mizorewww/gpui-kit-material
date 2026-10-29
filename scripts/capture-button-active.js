(() => {
  const snapshot = button => {
    const el = button.element;
    void getComputedStyle(el).borderRadius;
    el.getAnimations({subtree:true}).filter(a => Number.isFinite(a.effect.getTiming().iterations)).forEach(a => a.finish());
    const css = getComputedStyle(el), layer = getComputedStyle(el,'::before');
    return {active_class:el.classList.contains('mtrl-button--active'),pointer_active:el.matches(':active'),
      disabled:el.disabled,selected:el.getAttribute('aria-pressed'),radius:css.borderRadius,
      opacity:layer.opacity,shadow:css.boxShadow,background:css.backgroundColor,color:css.color,
      ripple_container:!!el.querySelector('.mtrl-ripple'),waves:el.querySelectorAll('.mtrl-ripple-wave').length};
  };
  window.captureActiveMatrix = () => {
    const rows=[];
    for(const variant of ['filled','elevated','tonal','outlined','text'])
    for(const size of ['xs','s','m','l','xl'])
    for(const shape of ['round','square'])
    for(const ripple of [false,true])
    for(const mode of ['plain','toggle','selected']) {
      const toggle=mode!=='plain', selected=mode==='selected';
      const button=window.referenceCreateButton({text:'Active action',variant,size,shape,ripple,
        toggle,selected,toggleOnClick:false});
      Object.assign(button.element.style,{position:'absolute',left:'20px',top:'20px'});
      document.body.replaceChildren(button.element);
      const before=snapshot(button);
      button.setActive(true); const active=snapshot(button);
      button.disable(); const disabled_active=snapshot(button);
      button.setActive(false); const disabled_inactive=snapshot(button);
      button.enable(); const restored=snapshot(button);
      rows.push({variant,size,shape,ripple,toggle,selected,before,active,disabled_active,disabled_inactive,restored});
      button.destroy();
    }
    return rows;
  };
  return true;
})();
