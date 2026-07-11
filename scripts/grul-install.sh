#!/usr/bin/env bash
# Installateur interactif GRUL v0.2 — CdC §10 (< 5 min, 4 questions).
# Usage : sudo grul install   ou   sudo bash scripts/grul-install.sh
set -euo pipefail

if [[ "${EUID:-$(id -u)}" -ne 0 ]]; then
  echo "Exécutez en root : sudo grul install"
  exit 1
fi

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
export DEBIAN_FRONTEND=noninteractive

echo "╔══════════════════════════════════════╗"
echo "║   GRUL — installation interactive    ║"
echo "╚══════════════════════════════════════╝"
echo ""
echo "Debian reste intact — GRUL ajoute une couche légère d'outils."
echo ""

# --- Question 1 : nom de la machine ---
default_host="$(hostname 2>/dev/null || echo grul-host)"
read -r -p "Nom de la machine [$default_host] : " GRUL_HOST
GRUL_HOST="${GRUL_HOST:-$default_host}"
hostnamectl set-hostname "$GRUL_HOST" 2>/dev/null || echo "$GRUL_HOST" > /etc/hostname

# --- Question 2 : utilisateur ---
default_user="${SUDO_USER:-${USER:-grul}}"
if [[ "$default_user" == "root" ]]; then
  default_user="grul"
fi
read -r -p "Utilisateur principal [$default_user] : " GRUL_USER
GRUL_USER="${GRUL_USER:-$default_user}"

if ! id "$GRUL_USER" &>/dev/null; then
  useradd -m -s /bin/bash -G sudo,adm "$GRUL_USER" 2>/dev/null || \
    useradd -m -s /bin/bash "$GRUL_USER"
  echo "  → utilisateur $GRUL_USER créé"
else
  echo "  → utilisateur $GRUL_USER existant"
fi

# --- Question 3 : mot de passe (nouvel utilisateur ou changement) ---
read -r -s -p "Mot de passe pour $GRUL_USER (Entrée = inchangé) : " GRUL_PASS
echo ""
if [[ -n "$GRUL_PASS" ]]; then
  echo "$GRUL_USER:$GRUL_PASS" | chpasswd
  echo "  → mot de passe défini"
fi

# --- Question 4 : usage ---
echo ""
echo "Usage prévu :"
echo "  1) VM        — légère, guest agents, sécurité auto"
echo "  2) Dev       — performance développement"
echo "  3) Bureau    — équilibré desktop"
echo "  4) Serveur   — minimal, services essentiels"
read -r -p "Choix [1-4] : " GRUL_USAGE
GRUL_USAGE="${GRUL_USAGE:-3}"

case "$GRUL_USAGE" in
  1) PROFILE="vm-minimal"; INSTALL_VM=true ;;
  2) PROFILE="dev-performance"; INSTALL_VM=false ;;
  3) PROFILE="desktop-balanced"; INSTALL_VM=false ;;
  4) PROFILE="server-minimal"; INSTALL_VM=false ;;
  *)
    echo "Choix invalide — bureau par défaut"
    PROFILE="desktop-balanced"
    INSTALL_VM=false
    ;;
esac

# Détection VM : priorité au choix explicite ou auto
if grul-detect --json 2>/dev/null | grep -q '"is_virtual": true'; then
  if [[ "$GRUL_USAGE" != "1" ]]; then
    read -r -p "VM détectée — basculer en profil VM ? [O/n] : " vm_confirm
    if [[ ! "$vm_confirm" =~ ^[Nn] ]]; then
      PROFILE="vm-minimal"
      INSTALL_VM=true
    fi
  fi
fi

echo ""
echo "==> Dépendances Debian"
apt-get update -qq
apt-get install -y -qq curl ca-certificates git build-essential pkg-config libssl-dev sudo

echo "==> Rust (si absent)"
if ! command -v cargo >/dev/null 2>&1; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  export PATH="/root/.cargo/bin:${PATH:-}"
fi

echo "==> Outils GRUL"
INSTALL_ARGS=()
if $INSTALL_VM; then
  INSTALL_ARGS+=(--vm)
fi
bash "$ROOT/scripts/install-tools.sh" "${INSTALL_ARGS[@]}"

echo "==> Profil : $PROFILE"
if $INSTALL_VM; then
  grul-doctor vm-setup --yes
else
  grul-tune apply --profile "$PROFILE" --yes
  grul-doctor drivers install 2>/dev/null || true
fi

mkdir -p /var/lib/grul
date -Iseconds > /var/lib/grul/firstboot-done

echo ""
echo "✓ GRUL installé — machine : $GRUL_HOST, utilisateur : $GRUL_USER"
echo ""
grul-doctor quick
echo ""
echo "Prochaines étapes :"
echo "  sudo grul update -y"
echo "  grul doctor"
echo "  grul status"
