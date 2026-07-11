# Dépôts APT GRUL

## Structure des dépôts

```
https://repo.grul.org/debian/
├── stable/          → pin Debian (miroir ou redirect)
├── grul-core/       → métapaquets + outils grul-*
├── grul-current/    → rebuilds apps/pilotes
└── grul-edge/       → outils développeur (opt-in)
```

## Fichier sources.list.d/grul.list (exemple)

```deb
# GRUL Core — outils système
deb [signed-by=/usr/share/keyrings/grul-archive-keyring.gpg] https://repo.grul.org/debian grul-core main

# GRUL Current — activé par défaut sur une install desktop
deb [signed-by=/usr/share/keyrings/grul-archive-keyring.gpg] https://repo.grul.org/debian grul-current main

# GRUL Edge — désactivé par défaut ; debcomment pour activer
# deb [signed-by=/usr/share/keyrings/grul-archive-keyring.gpg] https://repo.grul.org/debian grul-edge main
```

## Priorités APT (pinning)

Fichier `/etc/apt/preferences.d/grul` :

```
Package: *
Pin: release o=GRUL,a=grul-core
Pin-Priority: 500

Package: *
Pin: release o=GRUL,a=grul-current
Pin-Priority: 450

Package: *
Pin: release o=GRUL,a=grul-edge
Pin-Priority: 400

Package: *
Pin: release o=Debian,a=stable
Pin-Priority: 500
```

Les paquets **Current** ne doivent **pas** écraser les bibliothèques système critiques (`libc6`, `libssl3`, etc.) — seulement les paquets listés dans la manifeste GRUL Current.

## Manifeste GRUL Current (exemple)

| Paquet | Version cible | Rebuild depuis |
|--------|---------------|----------------|
| firefox-esr ou firefox | dernière stable | Mozilla tarball / Debian sid source |
| libreoffice | backport testé | Debian bookworm-backports |
| linux-image-amd64 | 6.12.x LTS | kernel.org + config GRUL |
| mesa | récent stable | Debian sid source |
| firmware-linux | non-free current | Debian non-free |

Chaque entrée = **pipeline CI** : build → test sur VM → publish.

## Métapaquets

| Métapaquet | Contenu |
|------------|---------|
| `grul-core` | `grul-detect`, `grul-tune`, `grul-update`, `grul-doctor`, config par défaut |
| `grul-desktop` | `grul-core` + GNOME/Xfce + apps Current |
| `grul-dev` | `grul-desktop` + activation Edge + outils de base |
| `grul-server` | `grul-core` + profil minimal, pas de Current sauf kernel option |

## Construction des dépôts (outline)

```bash
# 1. Chroot Debian Stable
debootstrap bookworm ./chroot

# 2. Installer grul-core dans le chroot
dpkg -i grul-core_*.deb

# 3. Rebuild paquet Current (exemple kernel)
# ... pipeline dans ci/build-kernel.yml

# 4. Publier
reprepro -b ./repo includedeb grul-current ./artifacts/*.deb
```

Voir `repos/README.md` pour les scripts futurs.
