#!/usr/bin/env bash
# Construit les paquets .deb GRUL (Linux + dpkg-deb + Rust).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
VERSION="${GRUL_VERSION:-0.1.0}"
ARCH="${GRUL_ARCH:-amd64}"
OUT="${ROOT}/dist/debs"
STAGE="${ROOT}/dist/staging"

echo "==> GRUL build-debs v${VERSION} (${ARCH})"
cd "$ROOT/tools"
cargo build --release

rm -rf "$STAGE" "$OUT"
mkdir -p "$OUT"

install_share_files() {
  local dir="$1"
  mkdir -p "$dir/usr/share/grul/scripts"
  install -m644 "$ROOT/configs/grul-channel-vm.toml" "$dir/usr/share/grul/"
  install -m755 "$ROOT/scripts/grul-firstboot.sh" "$dir/usr/share/grul/scripts/"
  install -m755 "$ROOT/scripts/uninstall-grul.sh" "$dir/usr/share/grul/scripts/"
  install -m755 "$ROOT/scripts/vm-bootstrap.sh" "$dir/usr/share/grul/scripts/"
  install -m644 "$ROOT/packaging/cloud-init/grul-vm.yaml" "$dir/usr/share/grul/" 2>/dev/null || true
}

build_binary_deb() {
  local name="$1"
  local desc="$2"
  local depends="${3:-}"
  local bin="${4:-$name}"
  local dir="${STAGE}/${name}_${VERSION}_${ARCH}"

  mkdir -p "$dir/DEBIAN" "$dir/usr/bin"
  install -Dm755 "$ROOT/tools/target/release/${bin}" "$dir/usr/bin/${bin}"

  cat >"$dir/DEBIAN/control" <<EOF
Package: ${name}
Version: ${VERSION}
Section: admin
Priority: optional
Architecture: ${ARCH}
Depends: ${depends:-libc6 (>= 2.31)}
Maintainer: GRUL Project <grul@localhost>
Description: ${desc}
EOF

  dpkg-deb --build "$dir" "$OUT/${name}_${VERSION}_${ARCH}.deb"
  echo "    → $OUT/${name}_${VERSION}_${ARCH}.deb"
}

build_grul_core_deb() {
  local dir="${STAGE}/grul-core_${VERSION}_${ARCH}"
  mkdir -p "$dir/DEBIAN"
  mkdir -p "$dir/etc/grul/profiles"
  mkdir -p "$dir/etc/apt/sources.list.d"
  mkdir -p "$dir/etc/apt/preferences.d"
  mkdir -p "$dir/lib/systemd/system"
  mkdir -p "$dir/usr/share/doc/grul-core"

  install -m644 "$ROOT/configs/grul-channel.toml" "$dir/etc/grul/channel.toml"
  install -m644 "$ROOT/configs/grul-snap.toml" "$dir/etc/grul/snap.toml"
  install -m644 "$ROOT/packaging/etc/grul/release" "$dir/etc/grul/release"
  install -m644 "$ROOT/configs/profiles/"*.toml "$dir/etc/grul/profiles/"
  install -m644 "$ROOT/packaging/apt/preferences.d/grul" "$dir/etc/apt/preferences.d/grul"
  install -m644 "$ROOT/packaging/apt/sources.list.d/grul-core.list" "$dir/etc/apt/sources.list.d/"
  install -m644 "$ROOT/packaging/apt/sources.list.d/grul-current.list" "$dir/etc/apt/sources.list.d/"
  install -m644 "$ROOT/packaging/systemd/grul-firstboot.service" "$dir/lib/systemd/system/"
  install -m644 "$ROOT/packaging/systemd/grul-update-security.service" "$dir/lib/systemd/system/"
  install -m644 "$ROOT/packaging/systemd/grul-update-security.timer" "$dir/lib/systemd/system/"
  install_share_files "$dir"

  cat >"$dir/DEBIAN/control" <<EOF
Package: grul-core
Version: ${VERSION}
Section: admin
Priority: optional
Architecture: ${ARCH}
Depends: grul-cli (= ${VERSION}), grul-detect (= ${VERSION}), grul-tune (= ${VERSION}), grul-update (= ${VERSION}), grul-doctor (= ${VERSION})
Recommends: grul-snap (= ${VERSION}), btrfs-progs
Suggests: grul-vm, grul-desktop
Maintainer: GRUL Project <grul@localhost>
Description: GRUL Core — configuration et métapaquet
EOF

  dpkg-deb --build "$dir" "$OUT/grul-core_${VERSION}_${ARCH}.deb"
  echo "    → $OUT/grul-core_${VERSION}_${ARCH}.deb"
}

build_grul_vm_deb() {
  local dir="${STAGE}/grul-vm_${VERSION}_${ARCH}"
  mkdir -p "$dir/DEBIAN" "$dir/etc/grul" "$dir/usr/share/grul"
  install -m644 "$ROOT/configs/grul-channel-vm.toml" "$dir/etc/grul/channel.toml"
  install -m644 "$ROOT/configs/grul-channel-vm.toml" "$dir/usr/share/grul/grul-channel-vm.toml"

  cat >"$dir/DEBIAN/control" <<EOF
Package: grul-vm
Version: ${VERSION}
Section: admin
Priority: optional
Architecture: ${ARCH}
Depends: grul-core (= ${VERSION})
Recommends: qemu-guest-agent, cloud-init
Maintainer: GRUL Project <grul@localhost>
Description: GRUL VM — métapaquet machines virtuelles
 Installe le canal VM (Core only) et recommande guest agent + cloud-init.
 Après install : sudo grul-doctor vm-setup
EOF

  dpkg-deb --build "$dir" "$OUT/grul-vm_${VERSION}_${ARCH}.deb"
  echo "    → $OUT/grul-vm_${VERSION}_${ARCH}.deb"
}

echo "==> Paquets binaires"
build_binary_deb "grul-cli" "GRUL — interface CLI unifiée (commande grul)" "" "grul"
build_binary_deb "grul-detect" "GRUL — détection matérielle, VM et profil"
build_binary_deb "grul-tune" "GRUL — profils d'optimisation" "grul-detect (= ${VERSION})"
build_binary_deb "grul-update" "GRUL — mises à jour canaux Core/Current/Edge"
build_binary_deb "grul-doctor" "GRUL — diagnostics et setup VM" "grul-detect (= ${VERSION})"
build_binary_deb "grul-snap" "GRUL — snapshots Btrfs optionnels" "btrfs-progs"

echo "==> Métapaquets"
build_grul_core_deb
build_grul_vm_deb

echo ""
echo "OK — paquets dans ${OUT}/"
echo ""
echo "VM (recommandé) :"
echo "  sudo dpkg -i ${OUT}/grul-*.deb"
echo "  sudo grul-doctor vm-setup"
echo ""
echo "Désinstaller :"
echo "  sudo bash /usr/share/grul/scripts/uninstall-grul.sh"
