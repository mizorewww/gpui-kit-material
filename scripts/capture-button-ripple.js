(async () => {
  const sleep = ms => new Promise(resolve => setTimeout(resolve, ms));
  const reduced = matchMedia('(prefers-reduced-motion: reduce)').matches;
  const result = {ua:navigator.userAgent,reduced_motion:reduced};
  let component;
  const make = (border=0, duration=450) => {
    if (component) component.destroy();
    document.body.innerHTML = '';
    component = window.referenceCreateButton({text:'MMMM',variant:'filled',rippleConfig:{duration}});
    const button = component.element;
    button.id = 'probe';
    Object.assign(button.style,{position:'absolute',left:'40px',top:'40px',width:'120px',height:'40px',
      border: `${border}px solid transparent`,backgroundColor:'black',color:'white'});
    document.body.append(button);
    return button;
  };
  const down = (button, pointerId=1, buttonNumber=0) => {
    const rect = button.getBoundingClientRect();
    const point = {x:rect.x+30,y:rect.y+15};
    button.dispatchEvent(new PointerEvent('pointerdown',{bubbles:true,pointerId,button:buttonNumber,clientX:point.x,clientY:point.y}));
    return point;
  };
  const release = (type='pointerup',pointerId=1) => document.dispatchEvent(new PointerEvent(type,{pointerId}));
  const wave = button => button.querySelector('.mtrl-ripple-wave');
  const snapshot = w => {
    const s = getComputedStyle(w); const r=w.getBoundingClientRect();
    return {opacity:Number(s.opacity),transform:s.transform,color:s.backgroundColor,
      width:parseFloat(s.width),height:parseFloat(s.height),left:parseFloat(s.left),top:parseFloat(s.top),
      center:{x:r.x+r.width/2,y:r.y+r.height/2}};
  };
  const animationInfo = a => ({type:a.constructor.name,name:a.animationName||a.transitionProperty,
    timing:a.effect.getTiming(),keyframes:a.effect.getKeyframes()});
  const seek = (w,time) => {
    w.getAnimations().forEach(a=>{a.pause();a.currentTime=time;});
    return {time,...snapshot(w)};
  };
  const b=make(); down(b); const w=wave(b);
  result.initial=snapshot(w);
  result.expansion={animations:w.getAnimations().map(animationInfo),samples:[0,40,100,200,300,400,600].map(t=>seek(w,t))};
  // Keep the expansion held at its final value, then seek the real opacity transition.
  release(); getComputedStyle(w).opacity;
  result.release={animations:w.getAnimations().map(animationInfo),samples:[0,40,100,200,300,400].map(t=>{
    w.getAnimations().forEach(a=>{a.pause();a.currentTime=a.transitionProperty==='opacity'?t:400;});
    return {time:t,...snapshot(w)};
  })};
  // A short press: expansion and fade run together, without minimum hold.
  const short=make();down(short);const sw=wave(short);seek(sw,40);release();getComputedStyle(sw).opacity;
  result.short_press={hold:40,samples:[0,100,200].map(t=>{
    sw.getAnimations().forEach(a=>{a.pause();a.currentTime=a.transitionProperty==='opacity'?t:40+t;});
    return {release_time:t,...snapshot(sw)};
  })};
  result.geometry=[];
  for(const border of [0,1,3]){
    const button=make(border);const pointer=down(button);const current=wave(button);seek(current,400);
    const bounds=button.getBoundingClientRect();const container=button.querySelector('.mtrl-ripple').getBoundingClientRect();
    const state=snapshot(current);
    result.geometry.push({border,pointer,button:{x:bounds.x,y:bounds.y,width:bounds.width,height:bounds.height},
      container:{x:container.x,y:container.y,width:container.width,height:container.height},wave:state,
      center_offset:{x:state.center.x-pointer.x,y:state.center.y-pointer.y}});
  }
  const multi=make();down(multi,1);down(multi,2);release('pointerup',99);
  result.multiple_unrelated_release=[...multi.querySelectorAll('.mtrl-ripple-wave')].map(x=>x.className);
  const leave=make();down(leave);leave.dispatchEvent(new MouseEvent('mouseleave'));
  result.element_mouseleave=wave(leave).className;document.dispatchEvent(new MouseEvent('mouseleave'));
  result.document_mouseleave=wave(leave).className;
  const cancel=make();down(cancel);release('pointercancel');result.pointercancel=wave(cancel).className;
  const disabled=make();down(disabled);component.disable();result.disabled_mid_press=wave(disabled).className;
  const right=make();down(right,1,2);result.right_pointerdown_wave_count=right.querySelectorAll('.mtrl-ripple-wave').length;
  const key=make();key.dispatchEvent(new KeyboardEvent('keydown',{key:'Enter',bubbles:true}));key.click();
  result.keyboard_and_click_wave_count=key.querySelectorAll('.mtrl-ripple-wave').length;
  const immediate=make();down(immediate);release();result.same_task_release_opacity=Number(getComputedStyle(wave(immediate)).opacity);
  const timed=make();down(timed);const tw=wave(timed);getComputedStyle(tw).opacity;const start=performance.now();release();
  let lastConnected=0;
  while(tw.isConnected && performance.now()-start<2000){lastConnected=performance.now()-start;await sleep(5);}
  result.cleanup={configured_ms:450,last_connected_ms:lastConnected,first_disconnected_ms:performance.now()-start,removed:!tw.isConnected};
  if(!result.cleanup.removed || result.cleanup.first_disconnected_ms<440) throw new Error('Unexpected cleanup');
  // Raster paint-order probe. Change text color only to distinguish label from
  // ripple (normally they have the same ink); preserve layout/stacking styles.
  window.preparePaintProbe=()=>{
    const button=make();const label=button.querySelector('.mtrl-button__text');
    Object.assign(label.style,{color:'black',backgroundColor:'rgb(200,0,0)'});
    down(button);seek(wave(button),400);
    const rect=label.getBoundingClientRect();
    window.paintProbe={label:{x:rect.x,y:rect.y,width:rect.width,height:rect.height},
      label_position:getComputedStyle(label).position,label_z_index:getComputedStyle(label).zIndex,
      ripple_z_index:getComputedStyle(button.querySelector('.mtrl-ripple')).zIndex};
  };
  window.readPaintProbe=async(url)=>{
    const image=new Image();image.src=url;await image.decode();
    const canvas=document.createElement('canvas');canvas.width=image.width;canvas.height=image.height;
    const ctx=canvas.getContext('2d');ctx.drawImage(image,0,0);
    const r=window.paintProbe.label;const sx=image.width/innerWidth,sy=image.height/innerHeight;
    const pixels=ctx.getImageData(Math.ceil(r.x*sx),Math.ceil(r.y*sy),Math.floor(r.width*sx),Math.floor(r.height*sy)).data;
    const counts={};for(let i=0;i<pixels.length;i+=4){const color=[...pixels.slice(i,i+4)].join(',');counts[color]=(counts[color]||0)+1;}
    const common=Object.entries(counts).sort((a,b)=>b[1]-a[1]).slice(0,8);
    return {...window.paintProbe,pixel_counts:common,
      interpretation:'Black glyph interior painted as gray means the white 10% ripple is above label; red label backing becomes (205,26,26).'};
  };
  // Keep real CSS animations/transitions; only extend disposal so CLI media
  // round trips cannot remove the probe before its style is inspected.
  const media = matchMedia('(prefers-reduced-motion: reduce)');
  const animationIds = new WeakMap(); let nextAnimationId = 1;
  const animationId = animation => {
    if (!animationIds.has(animation)) animationIds.set(animation,nextAnimationId++);
    return animationIds.get(animation);
  };
  let dynamicWave; let initialAnimation; let mediaSnapshots=[];
  const dynamicSnapshot=()=>({reduced_motion:media.matches,connected:dynamicWave.isConnected,
    ...snapshot(dynamicWave),transition:{property:getComputedStyle(dynamicWave).transitionProperty,
      duration:getComputedStyle(dynamicWave).transitionDuration},animations:dynamicWave.getAnimations().map(a=>({
      id:animationId(a),time:a.currentTime,play_state:a.playState,playback_rate:a.playbackRate,...animationInfo(a)})),
    initial_animation:initialAnimation?{id:animationId(initialAnimation),time:initialAnimation.currentTime,
      play_state:initialAnimation.playState}:null});
  media.addEventListener('change',()=>{
    if (!dynamicWave || !dynamicWave.isConnected) return;
    getComputedStyle(dynamicWave).transform;
    dynamicWave.getAnimations().filter(a=>a.animationName).forEach(a=>{a.pause();a.currentTime=0;});
    mediaSnapshots.push(dynamicSnapshot());
  });
  window.startDynamicProbe=()=>{
    const button=make(0,60000);down(button);dynamicWave=wave(button);
    getComputedStyle(dynamicWave).transform;
    initialAnimation=dynamicWave.getAnimations().find(a=>a.animationName);
    dynamicWave.getAnimations().forEach(a=>{a.pause();a.currentTime=100;});
    mediaSnapshots=[];
    return {disposal_duration_ms:60000,...dynamicSnapshot()};
  };
  window.dynamicSnapshot=dynamicSnapshot;
  window.dynamicMediaSnapshots=()=>mediaSnapshots;
  window.seekDynamicProbe=time=>{seek(dynamicWave,time);return dynamicSnapshot();};
  window.releaseDynamicProbe=()=>{
    release();getComputedStyle(dynamicWave).opacity;
    dynamicWave.getAnimations().forEach(a=>{a.pause();a.currentTime=100;});
    // Freeze the running transition at exact t=100 without CLI latency drift.
    // The full stylesheet's reduced-motion reset keeps opacity in the
    // transition-property list (!important), so an existing fade survives.
    dynamicWave.getAnimations().filter(a=>a.transitionProperty==='opacity').forEach(a=>{
      a.playbackRate=0;a.play();
    });
    return dynamicSnapshot();
  };
  return result;
})()
