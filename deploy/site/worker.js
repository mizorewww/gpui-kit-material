// Serves the Brotli copies scripts/build-site.py writes beside the Gallery's module and
// fonts: Cloudflare's own compression of them is worse (7.5 MB against 6.6 MB for the
// module). `encodeBody: "manual"` passes them through as they are. `no-transform` keeps
// the edge from compressing them again. Only /gallery/wasm/* runs this Worker
// (`runWorkerFirst` in cloudflare.config.ts); every other path stays a plain asset.
import { acceptsBrotli, contentType } from "./encoding.js";

const CACHE = "public, max-age=31536000, immutable, no-transform";

export default {
  async fetch(request, env) {
    const url = new URL(request.url);
    const type = contentType(url.pathname);
    if (type && (request.method === "GET" || request.method === "HEAD")
        && acceptsBrotli(request.headers.get("Accept-Encoding"))) {
      const copy = await env.ASSETS.fetch(new URL(`${url.pathname}.br`, url));
      if (copy.ok) {
        const headers = new Headers(copy.headers);
        headers.set("Content-Type", type);
        headers.set("Content-Encoding", "br");
        headers.set("Cache-Control", CACHE);
        headers.set("Vary", "Accept-Encoding");
        const body = request.method === "HEAD" ? null : copy.body;
        return new Response(body, { status: 200, headers, encodeBody: "manual" });
      }
    }
    return env.ASSETS.fetch(request);
  },
};
