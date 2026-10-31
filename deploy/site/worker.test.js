// node --test deploy/site/
import assert from "node:assert/strict";
import { test } from "node:test";
import { acceptsBrotli, contentType } from "./encoding.js";
import worker from "./worker.js";

const ASSETS = new Map([
  ["/gallery/wasm/gallery.abc.wasm", "raw"],
  ["/gallery/wasm/gallery.abc.wasm.br", "compressed"],
  ["/gallery/wasm/only-raw.wasm", "raw"],
  ["/gallery/wasm/gallery.abc.css", "raw"],
]);
const env = {
  ASSETS: {
    async fetch(input) {
      const { pathname } = new URL(input.url ?? input);
      const body = ASSETS.get(pathname);
      return body ? new Response(body) : new Response("not found", { status: 404 });
    },
  },
};
const get = (path, encoding, method = "GET") =>
  worker.fetch(
    new Request(`https://example.test${path}`, { method, headers: { "Accept-Encoding": encoding } }),
    env,
  );

test("Accept-Encoding", () => {
  assert.ok(acceptsBrotli("gzip, deflate, br"));
  assert.ok(acceptsBrotli("BR;q=0.5"));
  assert.ok(!acceptsBrotli("gzip"));
  assert.ok(!acceptsBrotli("br;q=0"));
  assert.ok(!acceptsBrotli(""));
  assert.ok(!acceptsBrotli(null));
});

test("content types", () => {
  assert.equal(contentType("/a.b.wasm"), "application/wasm");
  assert.equal(contentType("/a.ttf"), "font/ttf");
  assert.equal(contentType("/a.br"), undefined);
  assert.equal(contentType("/constructor"), undefined);
});

test("a client that accepts Brotli gets the copy under the asset's type", async () => {
  const response = await get("/gallery/wasm/gallery.abc.wasm", "gzip, br");
  assert.equal(await response.text(), "compressed");
  assert.equal(response.headers.get("Content-Encoding"), "br");
  assert.equal(response.headers.get("Content-Type"), "application/wasm");
  assert.equal(response.headers.get("Vary"), "Accept-Encoding");
  assert.match(response.headers.get("Cache-Control"), /immutable, no-transform/);
});

test("a client without Brotli, or an asset without a copy, gets the asset", async () => {
  for (const [path, encoding] of [
    ["/gallery/wasm/gallery.abc.wasm", "gzip"],
    ["/gallery/wasm/only-raw.wasm", "br"],
    ["/gallery/wasm/gallery.abc.css", "br"],
  ]) {
    const response = await get(path, encoding);
    assert.equal(await response.text(), "raw", path);
    assert.equal(response.headers.get("Content-Encoding"), null, path);
  }
});

test("HEAD has the headers and no body; a missing asset stays a 404", async () => {
  const head = await get("/gallery/wasm/gallery.abc.wasm", "br", "HEAD");
  assert.equal(head.headers.get("Content-Encoding"), "br");
  assert.equal(await head.text(), "");
  assert.equal((await get("/gallery/wasm/none.wasm", "br")).status, 404);
});
