#!/usr/bin/env bash
# Deploy the Disk Doctor landing page to Cloudflare Pages.
#
# One-step ship:   ./site/deploy.sh
# Or from anywhere: bash ~/Documents/rik-code/projects/disk-doctor/site/deploy.sh
#
# Requires:
#   - wrangler installed and logged in (`wrangler whoami` to verify)
#   - Cloudflare Pages project "disk-doctor" already exists on this account

set -euo pipefail

cd "$(dirname "$0")"

echo "→ Committing any pending changes in /site …"
cd ..
if [ -n "$(git status --porcelain site/)" ]; then
  git add site/
  git commit -m "site: update $(date +%Y-%m-%d-%H%M)" || true
  git push origin main
else
  echo "  (no changes to commit)"
fi

echo ""
echo "→ Deploying to Cloudflare Pages …"
cd site
wrangler pages deploy . --project-name=disk-doctor --branch=main --commit-dirty=true

echo ""
echo "✓ Live at: https://diskdoctor.midcurved.com"
echo "  Preview: https://disk-doctor.pages.dev"
