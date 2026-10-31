import { bindings, defineConfig } from "cf/config";

// Static assets on the custom domain, its only route. The Worker runs only for the
// Gallery's module and fonts, which it serves from their Brotli copies; see worker.js.
export default defineConfig({
  worker: {
    name: "gpui-kit-material",
    entrypoint: "./worker.js",
    compatibilityDate: "2026-10-23",
    workersDev: false,
    previewUrls: false,
    domains: ["material.mizore.blog"],
    env: { ASSETS: bindings.assets() },
    assets: {
      htmlHandling: "auto-trailing-slash",
      notFoundHandling: "404-page",
      runWorkerFirst: ["/gallery/wasm/*"],
    },
  },
});
