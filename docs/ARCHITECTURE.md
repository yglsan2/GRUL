# Architecture GRUL

## Principe fondateur

GRUL **ne remplace pas** Debian. Il **orchestre** Debian :

```
┌─────────────────────────────────────────────────────────┐
│  GRUL Edge (opt-in)     — dev tools, rolling contrôlé   │
├─────────────────────────────────────────────────────────┤
│  GRUL Current           — apps & pilotes récents        │
├─────────────────────────────────────────────────────────┤
│  GRUL Core              — outils GRUL, politique APT │
├─────────────────────────────────────────────────────────┤
│  Debian Stable          — base, sécurité, dpkg/apt      │
└─────────────────────────────────────────────────────────┘
```

## Partial rolling release — règles

| Type de paquet | Canal | Politique |
|----------------|-------|-----------|
| `libc`, `openssl`, `systemd`, noyau (option) | **Core** | Debian Stable + security |
| Firefox, LibreOffice, Mesa, firmware | **Current** | Rebuild GRUL, testés sur Stable |
| `golang`, `rustc`, Docker, kubectl | **Edge** | Versions récentes, opt-in |
| Paquets utilisateur (`grul-*`) | **GRUL repo** | Livrés par GRUL |

**Sécurité** : les CVE critiques passent **toujours** par Debian Security en priorité. GRUL Current ne remplace pas `stable-security`.

## Pourquoi Go et Rust ?

**Ce que ça apporte vraiment :**

- Binaires **statiques** ou quasi — déploiement simple, peu de dépendances runtime
- **Mémoire sûre** (Rust) pour les outils système sensibles (`grul-update`, `grul-snap`)
- **Concurrence** propre pour l'installateur et le centre de config (Go)
- **Tests unitaires** et CI modernes

**Ce que ça ne fait pas :**

- Réécrire `apt`, `systemd`, `glibc` ou le noyau — hors scope et contraire à la philosophie GRUL
- Rendre l'OS « hyper léger » à lui seul — le poids vient surtout de GNOME/KDE, LibreOffice, etc.

Le gain perçu vient surtout de :

1. **Moins de scripts shell fragiles** → outils uniques, testés
2. **Profils adaptés** → services inutiles désactivés, I/O adapté au SSD
3. **Canaux clairs** → pas de mélange stable/backports/snap flatpak au hasard

## Composants détaillés

### `grul-detect` (Rust)

Au premier boot (ou à la demande) :

- CPU (cœurs, flags, governor)
- RAM
- Disque (SSD/HDD/NVMe via `/sys/block/*/queue/rotational`)
- GPU (PCI, driver chargé)
- Type d'usage déclaré ou inféré (bureau / dev / serveur / gaming)

Produit un **profil** : `desktop-balanced`, `dev-performance`, `server-minimal`, `gaming-latency`.

### `grul-tune` (Rust)

Applique le profil sans casser Debian :

- `sysctl.d` (swappiness, network buffers)
- `systemd` — services optionnels masqués
- `fstrim.timer`, `noatime` sur SSD (avec avertissement)
- **Jamais** de modification opaque de paquets Debian

Tout est **réversible** via `grul-tune --reset`.

### `grul-update` (Rust)

Wrapper autour d'`apt` avec **politique de canaux** :

```toml
[channel]
core = "debian-stable"
current = "grul-current"
edge = "grul-edge"   # opt-in

[packages]
firefox = "current"
linux-image-amd64 = "current"  # option GRUL
docker.io = "edge"
```

- `grul-update` → met à jour Core + Current
- `grul-update --edge` → inclut Edge
- Snapshot automatique si `grul-snap` actif

### `grul-snap` (Rust)

- Btrfs subvolumes ou ZFS snapshots avant upgrade
- Rollback : `grul-snap rollback`

### `grul-install` (Go)

Installateur calqué sur Debian, avec :

- Choix du profil (bureau / dev / minimal)
- Partitionnement Btrfs recommandé
- Installation des métapaquets `grul-core`, `grul-desktop`, `grul-dev`

### `grul-config` (Go)

Centre de configuration unifié :

- Canaux Current/Edge
- Profil performance / batterie
- Pilotes propriétaires (NVIDIA) — avec avertissements
- Diagnostics (`grul-doctor` intégré)

## Différenciation vs existant

| Distribution | Proche de GRUL ? | Différence GRUL |
|--------------|------------------|-----------------|
| Debian + backports | Oui | Canaux explicites + auto-tuning |
| Ubuntu LTS | Partiel | Reste 100 % Debian, pas de Snap imposé |
| Linux Mint Debian | Partiel | Outils Rust/Go + Edge dev |
| Fedora Silverblue | Non | GRUL reste apt/dpkg classique |
| NixOS | Non | Pas de store Nix — compat Debian totale |

## Stack technique cible

- **Base** : Debian Stable (amd64 en premier, arm64 plus tard)
- **Init** : systemd (Debian default)
- **FS recommandé** : Btrfs (snapshots) ou ext4 (simple)
- **Desktop** : GNOME ou Xfce (metapackage GRUL, pas fork)
- **Build** : `debos` / `live-build` + `reprepro` pour les dépôts
