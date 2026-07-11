#!/usr/bin/env bash
# Premier démarrage GRUL — VM → vm-setup, bare metal → tune --auto
set -euo pipefail

if [[ -f /var/lib/grul/firstboot-done ]]; then
  exit 0
fi

if grul-detect --json 2>/dev/null | grep -q '"is_virtual": true'; then
  grul-doctor vm-setup --yes
else
  grul-tune apply --auto --yes
fi

mkdir -p /var/lib/grul
date -Iseconds > /var/lib/grul/firstboot-done
