(() => {
  const icon = '<svg viewBox="0 0 24 24"><path d="M2 12h20"/></svg>';
  const rect = el => {
    if (!el) return null;
    const {x,y,width,height} = el.getBoundingClientRect();
    return {x,y,width,height};
  };
  window.mountCircular = config => {
    window.circularButton?.destroy();
    const button = window.referenceCreateButton({
      text:'Circular action', icon, variant:'filled', size:'s', shape:'round',
      class:'mtrl-button--circular', ripple:false, progress:true,
      ariaLabel:'Circular action', toggleOnClick:false, ...config,
    });
    Object.assign(button.element.style, {position:'absolute',left:'20px',top:'20px'});
    document.body.replaceChildren(button.element);
    window.circularButton = button;
    return true;
  };
  window.snapshotCircular = () => {
    const button = window.circularButton, el = button.element;
    const style = getComputedStyle(el);
    // Record settled endpoints of actual CSS transitions without changing CSS.
    void style.borderRadius;
    el.getAnimations({subtree:true}).filter(a => Number.isFinite(a.effect.getTiming().iterations)).forEach(a=>a.finish());
    const child = selector => {
      const element = el.querySelector(selector);
      return element && {rect:rect(element),display:getComputedStyle(element).display,opacity:getComputedStyle(element).opacity};
    };
    return {rect:rect(el),classes:el.className,variant:button.getVariant(),shape:button.getShape(),
      disabled:el.disabled,selected:el.getAttribute('aria-pressed'),busy:el.getAttribute('aria-busy'),
      active:el.matches(':active'),label:button.getText(),aria_label:el.getAttribute('aria-label'),
      css:{width:style.width,height:style.height,min_width:style.minWidth,padding:[style.paddingTop,style.paddingRight,style.paddingBottom,style.paddingLeft],
        radius:[style.borderTopLeftRadius,style.borderTopRightRadius,style.borderBottomRightRadius,style.borderBottomLeftRadius],
        flex:style.flex,gap:style.gap,color:style.color,background:style.backgroundColor,border:style.borderWidth},
      text:child('.mtrl-button__text'),icon:child('.mtrl-button__icon'),progress:child('.mtrl-button__progress'),canvas:child('canvas'),
      touch_target:{width:getComputedStyle(el,'::after').width,height:getComputedStyle(el,'::after').height},
    };
  };
  window.circularContentMatrix = async () => {
    const rows=[];
    for(const variant of ['filled','elevated','tonal','outlined','text']) {
      for(const size of ['xs','s','m','l','xl']) {
        for(const shape of ['round','square']) {
          for(const content of ['text','icon','icon-text']) {
            mountCircular({variant,size,shape,text:content==='icon'?'':'Circular action',icon:content==='text'?undefined:icon});
            const before=snapshotCircular();
            await circularButton.showProgress();
            const progress=snapshotCircular();
            await circularButton.setLoading(true,'Working');
            const loading=snapshotCircular();
            await circularButton.setLoading(false);
            rows.push({variant,size,shape,content,before,progress,loading,restored:snapshotCircular()});
          }
        }
      }
    }
    mountCircular({shape:'circular',class:undefined});
    const shape_only=snapshotCircular();
    circularButton.setShape('square');
    const shape_then_square=snapshotCircular();
    mountCircular({variant:'circular',class:undefined});
    const variant_only=snapshotCircular();
    mountCircular({class:undefined}); circularButton.addClass('mtrl-button--circular');
    const add_class=snapshotCircular();
    circularButton.element.style.width='200px';
    const inline_width=snapshotCircular();
    circularButton.element.style.setProperty('width','200px','important');
    const inline_important_width=snapshotCircular();
    return {rows,api:{shape_only,shape_then_square,variant_only,add_class,inline_width,inline_important_width}};
  };
  return true;
})()
