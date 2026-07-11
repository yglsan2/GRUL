#!/usr/bin/env bash
# Désinstalle GRUL proprement — Debian reste intact.
# Usage : sudo bash scripts/uninstall-grul.sh [--purge]
set -euo pipefail

PURGE=false
for arg in "$@"; do
  case "$arg" in
    --purge) PURGE=true ;;
  esac
done

if [[ "${EUID:-$(id -u)}" -ne 0 ]]; then
  echo "Root requis : sudo bash $0"
  exit 1
fi

BACKUP="/var/lib/grul/backup/uninstall-$(date +%Y%m%d-%H%M%S)"
mkdir -p "$BACKUP"

echo "GRUL — désinstallation"
echo "======================"
echo ""

backup_if_exists() {
  local f="$1"
  if [[ -e "$f" ]]; then
    cp -a "$f" "$BACKUP/" 2>/dev/null || cp "$f" "$BACKUP/"
    echo "  sauvegarde : $f"
  fi
}

echo "==> Sauvegarde configuration"
backup_if_exists /etc/grul
backup_if_exists /var/lib/grul/applied.json
backup_if_exists /var/lib/grul/snapshots.json

echo "==> Annulation du tuning"
if command -v grul-tune >/dev/null 2>&1; then
  grul-tune reset --dry-run 2>/dev/null | head -5 || true
  grul-tune reset 2>/dev/null || echo "  (reset ignoré ou déjà neutre)"
fi

echo "==> Arrêt timers systemd GRUL"
systemctl disable --now grul-update-security.timer 2>/dev/null || true
systemctl disable grul-firstboot.service 2>/dev/null || true

echo "==> Suppression binaires"
for bin in grul-detect grul-tune grul-update grul-snap grul-doctor; do
  rm -f "/usr/local/bin/$bin" "/usr/bin/$bin"
done

echo "==> Suppression configuration APT GRUL"
rm -f /etc/apt/sources.list.d/grul-core.list
rm -f /etc/apt/sources.list.d/grul-current.list
rm -f /etc/apt/sources.list.d/grul-edge.list
rm -f /etc/apt/preferences.d/grul

echo "==> Suppression unités systemd"
rm -f /etc/systemd/system/grul-firstboot.service
rm -f /etc/systemd/system/grul-update-security.service
rm -f /etc/systemd/system/grul-update-security.timer
systemctl daemon-reload 2>/dev/null || true

if $PURGE; then
  echo "==> Purge complète /etc/grul et /var/lib/grul"
  rm -rf /etc/grul
  rm -rf /var/lib/grul
else
  echo "==> Retrait /etc/grul (état dans $BACKUP)"
  rm -rf /etc/grul
  mkdir -p /var/lib/grul
  echo "Config sauvegardée dans $BACKUP" > /var/lib/grul/uninstalled.txt
fi

echo "==> Paquets .deb (si présents)"
if command -v dpkg >/dev/null 2>&1; then
  apt-get remove -y grul-core grul-detect grul-tune grul-update grul-doctor grul-snap 2>/dev/null || true
fi

echo ""
echo "✓ GRUL désinstallé — Debian inchangé sous la couche GRUL."
echo "  Sauvegarde : $BACKUP"
echo "  Réinstaller : bash scripts/vm-bootstrap.sh"
