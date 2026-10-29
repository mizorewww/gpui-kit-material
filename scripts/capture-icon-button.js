(() => {
  const icon = '<svg data-icon="off" viewBox="0 0 24 24"><path d="M2 12h20"/></svg>';
  const selectedIcon = '<svg data-icon="on" viewBox="0 0 24 24"><path d="M12 2v20"/></svg>';
  const rect = el => { if(!el)return null; const {x,y,width,height}=el.getBoundingClientRect();return {x,y,width,height}; };
  const css = (el,pseudo) => {
    const s=getComputedStyle(el,pseudo);
    return Object.fromEntries(['width','height','minWidth','minHeight','boxSizing','paddingTop','paddingRight','paddingBottom','paddingLeft','borderTopLeftRadius','borderTopRightRadius','borderBottomRightRadius','borderBottomLeftRadius','backgroundColor','color','opacity','display','overflow','pointerEvents','cursor','transform','borderTopWidth','borderTopStyle','borderTopColor','outlineWidth','outlineStyle','outlineColor','outlineOffset','transitionProperty','transitionDuration','transitionTimingFunction','transitionDelay','content','top','left'].map(k=>[k,s[k]]));
  };
  window.mountIcon = config => {
    window.iconButton?.destroy();
    window.iconEvents=[];
    const b=window.referenceCreateIconButton({icon,selectedIcon,ariaLabel:'Reference icon action',ripple:false,toggleOnClick:false,...config});
    b.on('change',event=>iconEvents.push({selected:event.selected,value:event.value}));
    b.element.id='sample';
    Object.assign(b.element.style,{position:'absolute',left:'80px',top:'80px'});
    const sentinel=document.createElement('button');sentinel.id='sentinel';sentinel.textContent='Focus sentinel';Object.assign(sentinel.style,{position:'absolute',left:'-1000px'});
    document.body.replaceChildren(sentinel,b.element);window.iconButton=b;
    const r=rect(b.element);return {x:r.x+r.width/2,y:r.y+r.height/2};
  };
  window.snapshotIcon = (settle=true) => {
    const b=iconButton,e=b.element;void getComputedStyle(e).borderRadius;
    if(settle)e.getAnimations({subtree:true}).filter(a=>Number.isFinite(a.effect.getTiming().iterations)).forEach(a=>a.finish());
    const i=b.icon.getElement(),svg=i?.querySelector('svg');
    return {rect:rect(e),css:css(e),state_layer:css(e,'::before'),touch_target:css(e,'::after'),
      classes:e.className,aria_label:e.getAttribute('aria-label'),aria_pressed:e.getAttribute('aria-pressed'),aria_disabled:e.getAttribute('aria-disabled'),
      disabled:e.disabled,selected:b.isSelected(),toggle:b.isToggle(),variant:b.getVariant(),size:b.getSize(),width:b.getWidth(),shape:b.getShape(),
      active:e.matches(':active'),hover:e.matches(':hover'),focus:e===document.activeElement,focus_visible:e.matches(':focus-visible'),tab_index:e.tabIndex,
      icon:i&&{rect:rect(i),css:css(i),html:i.innerHTML},svg:svg&&{rect:rect(svg),css:css(svg),identity:svg.dataset.icon},events:iconEvents.slice()};
  };
  window.prepareIconFocus=()=>{document.getElementById('sentinel').focus();return true;};
  window.captureIconMotion = () => {
    const e=iconButton.element;void getComputedStyle(e).borderRadius;
    const animations=e.getAnimations({subtree:true}).filter(a=>Number.isFinite(a.effect.getTiming().iterations));
    animations.forEach(a=>a.pause());
    const tracks=animations.map(a=>({property:a.transitionProperty??null,pseudo:a.effect.pseudoElement??null,timing:a.effect.getTiming(),keyframes:a.effect.getKeyframes()}));
    const duration=Math.max(0,...animations.map(a=>Number(a.effect.getTiming().duration)));
    const frames=[0,.125,.25,.375,.5,.75,1].map(fraction=>{const time_ms=duration*fraction;animations.forEach(a=>a.currentTime=time_ms);const s=snapshotIcon(false);return {time_ms,css:s.css,state_layer:s.state_layer};});
    animations.forEach(a=>a.finish());
    return {tracks,frames};
  };
  window.selectIconMotion=()=>{snapshotIcon();iconButton.select();return captureIconMotion();};
  window.armIconPressMotion=()=>{window.iconPressMotion=null;iconButton.element.addEventListener('mousedown',()=>{requestAnimationFrame(()=>window.iconPressMotion=captureIconMotion());},{once:true});return true;};
  window.iconApiSamples=()=>{
    const result={};
    mountIcon({});result.defaults=snapshotIcon();iconButton.select();result.non_toggle_select=snapshotIcon();
    mountIcon({selected:true});result.selected_without_toggle=snapshotIcon();
    mountIcon({toggle:true,selected:true});result.selected_initial=snapshotIcon();
    iconButton.setShape('square');result.shape_square=snapshotIcon();iconButton.setShape('round');result.shape_round_after_setter=snapshotIcon();
    iconButton.setSize('m');result.size_m=snapshotIcon();iconButton.setSize('s');result.size_s_after_setter=snapshotIcon();
    iconButton.setWidth('wide');result.width_wide=snapshotIcon();iconButton.setWidth('default');result.width_default_after_setter=snapshotIcon();
    iconButton.setIcon('<svg data-icon="updated" viewBox="0 0 24 24"/>');result.set_icon_while_selected=snapshotIcon();iconButton.deselect();result.deselect_after_set_icon=snapshotIcon();
    iconButton.disable();iconButton.select();result.programmatic_select_disabled=snapshotIcon();
    mountIcon({toggle:true});iconButton.element.style.setProperty('--mtrl-button-shape','7px');iconButton.element.style.setProperty('--mtrl-button-shape-selected','9px');result.override_rest=snapshotIcon();iconButton.select();result.override_selected=snapshotIcon();
    return result;
  };
  return true;
})()
