#!/usr/bin/env bash
# Installe les outils GRUL en local (Linux, root pour /usr/local).
# Usage : install-tools.sh [--snap] [--vm]
set -euo pipefail

INSTALL_SNAP=false
INSTALL_VM=false
for arg in "$@"; do
  case "$arg" in
    --snap) INSTALL_SNAP=true ;;
    --vm) INSTALL_VM=true ;;
  esac
done

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT/tools"

echo "==> Compilation release"
cargo build --release

echo "==> Installation binaires"
install -Dm755 target/release/grul /usr/local/bin/grul
install -Dm755 target/release/grul-detect /usr/local/bin/grul-detect
install -Dm755 target/release/grul-tune /usr/local/bin/grul-tune
install -Dm755 target/release/grul-update /usr/local/bin/grul-update
install -Dm755 target/release/grul-doctor /usr/local/bin/grul-doctor

if $INSTALL_SNAP; then
  install -Dm755 target/release/grul-snap /usr/local/bin/grul-snap
  install -m644 "$ROOT/configs/grul-snap.toml" /etc/grul/snap.toml
  echo "    grul-snap installé"
else
  echo "    grul-snap non installé — ajoutez --snap si souhaité"
fi

echo "==> Configuration GRUL"
install -dm755 /etc/grul /usr/share/grul/scripts
if $INSTALL_VM && [[ -f "$ROOT/configs/grul-channel-vm.toml" ]]; then
  install -m644 "$ROOT/configs/grul-channel-vm.toml" /etc/grul/channel.toml
  install -m644 "$ROOT/configs/grul-channel-vm.toml" /usr/share/grul/grul-channel-vm.toml
else
  install -m644 "$ROOT/configs/grul-channel.toml" /etc/grul/channel.toml
fi
install -m644 "$ROOT/configs/grul-channel-vm.toml" /usr/share/grul/grul-channel-vm.toml 2>/dev/null || true
install -m644 "$ROOT/packaging/etc/grul/release" /etc/grul/release
if [[ ! -f /etc/grul/snap.toml ]]; then
  install -m644 "$ROOT/configs/grul-snap.toml" /etc/grul/snap.toml
fi

echo "==> Profils système"
install -dm755 /etc/grul/profiles
install -m644 "$ROOT/configs/profiles/"*.toml /etc/grul/profiles/

echo "==> Scripts GRUL"
install -m755 "$ROOT/scripts/grul-firstboot.sh" /usr/share/grul/scripts/
install -m755 "$ROOT/scripts/grul-install.sh" /usr/share/grul/scripts/
install -m755 "$ROOT/scripts/uninstall-grul.sh" /usr/share/grul/scripts/
install -m755 "$ROOT/scripts/vm-bootstrap.sh" /usr/share/grul/scripts/

echo "==> APT GRUL (sources + pinning)"
install -dm755 /etc/apt/preferences.d /etc/apt/sources.list.d
install -m644 "$ROOT/packaging/apt/preferences.d/grul" /etc/apt/preferences.d/grul
install -m644 "$ROOT/packaging/apt/sources.list.d/grul-core.list" /etc/apt/sources.list.d/
install -m644 "$ROOT/packaging/apt/sources.list.d/grul-current.list" /etc/apt/sources.list.d/

echo "==> systemd (optionnel)"
if [[ -d /etc/systemd/system ]]; then
  install -Dm644 "$ROOT/packaging/systemd/grul-firstboot.service" \
    /etc/systemd/system/grul-firstboot.service
  install -Dm644 "$ROOT/packaging/systemd/grul-update-security.service" \
    /etc/systemd/system/grul-update-security.service
  install -Dm644 "$ROOT/packaging/systemd/grul-update-security.timer" \
    /etc/systemd/system/grul-update-security.timer
  systemctl daemon-reload
  if $INSTALL_VM; then
    systemctl enable --now grul-update-security.timer 2>/dev/null || true
    echo "    VM : timer sécurité activé"
  else
    echo "    systemctl enable --now grul-update-security.timer  # optionnel"
  fi
  echo "    systemctl enable grul-firstboot.service  # premier démarrage"
fi

mkdir -p /var/lib/grul

echo ""
if $INSTALL_VM; then
  echo "OK — VM : sudo grul-doctor vm-setup"
else
  echo "OK — grul-update upgrade | grul-doctor"
fi
