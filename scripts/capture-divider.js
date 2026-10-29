(() => {
  const rect = el => {
    const {x,y,width,height}=el.getBoundingClientRect();
    return {x,y,width,height};
  };
  let component, host;
  function mount(config, {direction='ltr',layout='block',context='plain',scale=1,mode='light'}={}) {
    component?.destroy();
    document.documentElement.style.fontSize=`${16*scale}px`;
    document.documentElement.dataset.theme='baseline';
    document.documentElement.dataset.themeMode=mode;
    host=document.createElement('div');
    host.dir=direction;
    if(context!=='plain') host.className=`mtrl-${context}`;
    // Pin only the parent's layout. Leave every component declaration untouched.
    Object.assign(host.style,{position:'absolute',left:'20px',top:'20px',width:'240px',height:'120px',
      padding:'0',border:'0',boxSizing:'border-box',display:layout==='block'?'block':'flex',
      flexDirection:layout==='column'?'column':'row',alignItems:layout==='row-center'?'center':'stretch',gap:'0'});
    component=referenceCreateDivider(config);
    host.append(component.element);
    document.body.replaceChildren(host);
  }
  function sample() {
    const el=component.element,s=getComputedStyle(el),bounds=rect(el),parent=rect(host);
    bounds.x-=parent.x; bounds.y-=parent.y;
    return {rect:bounds,parent_rect:parent,inline_style:el.getAttribute('style'),css_size:[s.width,s.height],margin:[s.marginTop,s.marginRight,s.marginBottom,s.marginLeft],
      background:s.backgroundColor,display:s.display,box_sizing:s.boxSizing,
      aria_orientation:el.getAttribute('aria-orientation'),role:el.getAttribute('role'),tag:el.tagName,
      variant:component.getVariant(),orientation:component.getOrientation(),classes:el.className,
      transition_duration:s.transitionDuration,animation_count:el.getAnimations().length};
  }
  window.captureDividerMatrix=()=>{
    const rows=[];
    for(const mode of ['light','dark']) for(const scale of [1,1.25,1.5,2])
    for(const direction of ['ltr','rtl']) for(const layout of ['block','row','row-center','column'])
    for(const context of ['plain','list','card']) for(const orientation of ['horizontal','vertical'])
    for(const variant of ['full-width','inset','middle-inset']) {
      const config={orientation,variant},environment={mode,scale,direction,layout,context};
      mount(config,environment); rows.push({...config,...environment,sample:sample()});
    }
    const custom=[];
    for(const direction of ['ltr','rtl']) for(const orientation of ['horizontal','vertical'])
    for(const variant of ['full-width','inset','middle-inset'])
    for(const config of [{thickness:3,insetStart:24,insetEnd:8},{thickness:0,insetStart:0,insetEnd:0},
      {thickness:0.5,insetStart:4,insetEnd:28},{thickness:2,color:'var(--mtrl-sys-color-primary)',insetEnd:12}]) {
      mount({...config,orientation,variant},{direction,layout:orientation==='vertical'?'row':'column'});
      custom.push({direction,orientation,variant,config,sample:sample()});
    }
    const setters=[];
    for(const direction of ['ltr','rtl']) {
      mount({orientation:'horizontal',variant:'middle-inset',insetStart:24,insetEnd:8},{direction,layout:'row'});
      const steps=[{operation:'initial',sample:sample()}];
      for(const [method,args] of [['setOrientation',['vertical']],['setVariant',['full-width']],
        ['setThickness',[3]],['setVariant',['inset']],['setInset',[0,12]],['setOrientation',['horizontal']],
        ['setVariant',['middle-inset']],['setThickness',[0]],['setColor',['rgba(18, 52, 86, 0.5)']],
        ['setInset',[undefined,0]],['setVariant',['full-width']],['setColor',['']]]) {
        const chainable=component[method](...args)===component;
        steps.push({operation:method,args:args.map(v=>v===undefined?'undefined':v),chainable,sample:sample()});
      }
      setters.push({direction,steps});
    }
    mount({});
    return {rows,custom,setters,defaults:sample()};
  };
  return true;
})()
