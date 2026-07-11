# grul-update — guide utilisateur

## Philosophie

GRUL ne remplace pas `apt`. Il **orchestre** les mises à jour avec trois canaux explicites :

| Canal | Contenu | Par défaut |
|-------|---------|------------|
| **Core** | Debian Stable + outils `grul-*` | Toujours actif |
| **Current** | Firefox, LibreOffice, Mesa, kernel option | Actif (desktop) |
| **Edge** | Go, Rust, Docker, kubectl | Opt-in |

L'objectif : des mises à jour **guidées** — un outil, un résumé clair, confirmation avant upgrade, sécurité automatisable.

## Commandes

### Statut (sans root)

```bash
grul-update status
```

Affiche la version GRUL, les canaux actifs, le nombre de mises à jour par canal et les correctifs Debian Security.

### Mise à jour complète

```bash
sudo grul-update upgrade
```

1. `apt update`
2. Synchronise les fichiers `sources.list.d` GRUL
3. Résumé par canal
4. Confirmation `[O/n]`
5. `apt upgrade` (ou `--full` pour `full-upgrade`)

### Sans interaction

```bash
sudo grul-update upgrade -y
```

### Sécurité uniquement

```bash
sudo grul-update upgrade --security-only -y
```

Utilisé par le timer systemd `grul-update-security.timer` (quotidien).

### Canal Edge (opt-in)

```bash
grul-update edge status
sudo grul-update edge enable
sudo grul-update upgrade --edge
sudo grul-update edge disable
```

### Nouvelle version GRUL (do-release-upgrade)

Quand une version majeure GRUL est publiée, un fichier `/var/lib/grul/release-available` signale la disponibilité :

```bash
grul-update release-check
sudo grul-update release-upgrade
```

## Configuration

Fichier : `/etc/grul/channel.toml`

```toml
[channel]
current_enabled = true
edge_enabled = false

[packages]
firefox-esr = "current"
docker.io = "edge"
```

Variable d'environnement dev : `GRUL_CHANNEL_CONFIG=configs/grul-channel.toml`

## Sécurité automatique

```bash
sudo systemctl enable --now grul-update-security.timer
```

Équivalent léger d'`unattended-upgrades` : uniquement les paquets Debian Security.

## Fichiers système

| Fichier | Rôle |
|---------|------|
| `/etc/grul/channel.toml` | Canaux et overrides paquets |
| `/etc/grul/release` | Version GRUL installée |
| `/etc/apt/sources.list.d/grul-*.list` | Dépôts GRUL |
| `/etc/apt/preferences.d/grul` | Pinning APT |
| `/var/lib/grul/release-available` | Nouvelle version GRUL signalée |

## Intégration grul-snap (optionnel)

Si `grul-snap` est installé et `snapshot_before_current = true`, un snapshot Btrfs est tenté avant les upgrades **Current**. Sinon, message informatif — pas d'échec (`fail_open`).

```bash
sudo grul-snap enable
sudo grul-update upgrade
sudo grul-snap rollback --last && sudo reboot
```

Voir [INSTALL.md](INSTALL.md) pour le choix entre paquets seuls ou avec snapshots.
