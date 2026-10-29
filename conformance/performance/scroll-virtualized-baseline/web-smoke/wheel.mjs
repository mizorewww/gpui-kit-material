const pages = await (await fetch('http://127.0.0.1:65292/json/list')).json();
const page = pages.find(p => p.url.startsWith('http://127.0.0.1:8094'));
const ws = new WebSocket(page.webSocketDebuggerUrl);
await new Promise((resolve,reject)=>{ws.onopen=resolve;ws.onerror=reject});
ws.send(JSON.stringify({id:1,method:'Input.dispatchMouseEvent',params:{type:'mouseWheel',x:700,y:350,deltaX:0,deltaY:Number(process.argv[2]),pointerType:'mouse'}}));
await new Promise((resolve,reject)=>{ws.onmessage=e=>{const m=JSON.parse(e.data);if(m.id===1){if(m.error)reject(m.error);else resolve();}};ws.onerror=reject});
ws.close();
