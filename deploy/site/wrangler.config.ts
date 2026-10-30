import { defineWranglerConfig } from "wrangler/experimental-config";

// scripts/deploy-site.sh passes the directory that scripts/build-site.sh wrote.
export default defineWranglerConfig({
  assetsDirectory: process.env.SITE_DIR ?? "../../target/site",
});
