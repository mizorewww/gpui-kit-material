#!/usr/bin/env bash
# Publish target/site (from scripts/build-site.sh) to material.mizore.blog with
# the Cloudflare CLI, then check the live URLs. Run by hand after `cf auth login`;
# no credentials live in this repository and CI does not deploy.
set -euo pipefail

workspace_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$workspace_dir"
origin="https://material.mizore.blog"
target_dir="$(cargo metadata --no-deps --format-version 1 | python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])')"
site_dir="$target_dir/site"
if [[ ! -f "$site_dir/index.html" || ! -f "$site_dir/gallery/index.html" || ! -f "$site_dir/api/index.html" ]]; then
  echo "Run ./scripts/build-site.sh first: $site_dir is incomplete." >&2
  exit 1
fi
# Cloudflare rejects a static asset over 25 MiB.
oversized="$(find "$site_dir" -type f -size +26214400c)"
if [[ -n "$oversized" ]]; then
  echo "Over the 25 MiB per-file limit:" >&2
  echo "$oversized" >&2
  exit 1
fi
# The CLI caches account details in .cloudflare/ under its working directory.
if [[ -n "$(find "$site_dir" -name .cloudflare -print -quit)" ]]; then
  echo "Refusing to publish a .cloudflare directory from $site_dir." >&2
  exit 1
fi
commit="$(git rev-parse --short HEAD)"

(
  cd deploy/site
  npm ci --no-audit --no-fund --loglevel=error
  SITE_DIR="$site_dir" npx cf deploy --message "$commit"
)

status=0
for path in / /gallery/ /api/; do
  code="$(curl -sS -o /dev/null -w '%{http_code}' --retry 5 --retry-delay 5 --retry-all-errors "$origin$path")"
  echo "$code $origin$path"
  [[ "$code" == 200 ]] || status=1
done
exit "$status"
