#!/usr/bin/env bash
# Bootstrap GRUL sur une VM Debian minimale — une commande.
# Usage : curl -fsSL .../vm-bootstrap.sh | sudo bash
#     ou : sudo bash scripts/vm-bootstrap.sh
set -euo pipefail

GRUL_CHANNEL_VM="${GRUL_CHANNEL_VM:-true}"
INSTALL_SNAP="${INSTALL_SNAP:-false}"

if [[ "${EUID:-$(id -u)}" -ne 0 ]]; then
  echo "Exécutez en root : sudo bash $0"
  exit 1
fi

ROOT="$(cd "$(dirname "$0")/.." && pwd)"

echo "╔══════════════════════════════════════╗"
echo "║     GRUL VM Bootstrap (Debian)       ║"
echo "╚══════════════════════════════════════╝"
echo ""

echo "==> Dépendances Debian"
export DEBIAN_FRONTEND=noninteractive
apt-get update -qq
apt-get install -y -qq \
  curl ca-certificates git build-essential pkg-config \
  libssl-dev qemu-guest-agent cloud-init 2>/dev/null || \
apt-get install -y curl ca-certificates git build-essential pkg-config libssl-dev

echo "==> Rust (si absent)"
if ! command -v cargo >/dev/null 2>&1; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  # shellcheck disable=SC1091
  source "$HOME/.cargo/env" 2>/dev/null || true
  export PATH="$HOME/.cargo/bin:/root/.cargo/bin:$PATH"
fi

echo "==> Installation GRUL"
if [[ -f "$ROOT/scripts/install-tools.sh" ]]; then
  bash "$ROOT/scripts/install-tools.sh" ${INSTALL_SNAP:+--snap}
else
  echo "Clonez d'abord le repo GRUL ou utilisez les paquets .deb"
  exit 1
fi

if [[ "$GRUL_CHANNEL_VM" == "true" && -f "$ROOT/configs/grul-channel-vm.toml" ]]; then
  install -m644 "$ROOT/configs/grul-channel-vm.toml" /etc/grul/channel.toml
  echo "    Canal VM appliqué (Core only, sécurité auto)"
fi

echo "==> Configuration VM (profil vm-minimal)"
grul-doctor vm-setup --yes

echo ""
echo "✓ GRUL VM prête"
echo ""
grul-doctor quick
echo ""
echo "Mises à jour : sudo grul update -y"
echo "Santé       : grul doctor"
echo "Désinstaller: sudo bash scripts/uninstall-grul.sh"
