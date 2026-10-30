import { defineConfig } from "cf/config";

// An assets-only Worker: no entrypoint runs, and the custom domain is its only route.
export default defineConfig({
  worker: {
    name: "gpui-kit-material",
    compatibilityDate: "2026-10-23",
    workersDev: false,
    previewUrls: false,
    domains: ["material.mizore.blog"],
    assets: { htmlHandling: "auto-trailing-slash", notFoundHandling: "404-page" },
  },
});
