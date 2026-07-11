# Installation GRUL — deux chemins, au choix de l'utilisateur

GRUL ne force **ni** les snapshots **ni** l'installation manuelle. Deux voies équivalentes en fonctionnalités de base ; seul le rollback Btrfs diffère.

## Comparaison rapide

| | **Voie A — Paquets .deb** | **Voie B — + grul-snap** |
|---|---------------------------|---------------------------|
| Public | Tout le monde | Utilisateurs avancés, Btrfs |
| Rollback | `apt` / backups classiques | `grul-snap rollback` |
| Prérequis | Debian Stable | + partition `/` en Btrfs |
| Installation | `apt install grul-core` | `apt install grul-core grul-snap` |
| Obligatoire ? | Non (snap est **Recommends**, pas Depends) | Non |

## Voie A — Classique (comme Ubuntu)

Pour la majorité des utilisateurs : mises à jour via `grul-update`, sans snapshot.

```bash
# Construire les paquets (sur une machine de build)
bash scripts/build-debs.sh

# Installer sur la cible
sudo dpkg -i dist/debs/grul-detect_*.deb \
             dist/debs/grul-tune_*.deb \
             dist/debs/grul-update_*.deb \
             dist/debs/grul-core_*.deb

# Ou sans les Recommends (sans grul-snap)
sudo apt install ./dist/debs/grul-core_*.deb --no-install-recommends
```

Premier usage :

```bash
sudo grul-tune apply --auto
grul-update status
sudo grul-update upgrade
```

## Voie B — Avec snapshots Btrfs (opt-in)

Pour ceux qui veulent un rollback en un redémarrage, à la Silverblue / Snapper :

```bash
# Prérequis : / installé en Btrfs (grul-install le recommandera)
sudo dpkg -i dist/debs/grul-*.deb

# Mode auto (défaut) : snapshots si Btrfs détecté
grul-snap status

# Ou forcer l'activation
sudo grul-snap enable

# Test manuel
sudo grul-snap create --dry-run

# Les upgrades Current déclenchent un snapshot automatique
sudo grul-update upgrade
```

Rollback après une mauvaise mise à jour :

```bash
grul-snap list
sudo grul-snap rollback --last
sudo reboot
```

Désactiver les snapshots à tout moment :

```bash
sudo grul-snap disable
# grul-update continue de fonctionner normalement
```

## Voie interactive — grul install (v0.2)

Installateur en 4 questions (nom, utilisateur, mot de passe, usage) — objectif < 5 min :

```bash
sudo grul install
# ou
sudo bash scripts/grul-install.sh
```

Usages : VM, Dev, Bureau, Serveur — profil et guest agents appliqués automatiquement.

## Voie développement — install-tools.sh

Sans paquets .deb, pour tester sur une VM :

```bash
sudo bash scripts/install-tools.sh          # sans grul-snap
sudo bash scripts/install-tools.sh --snap   # avec grul-snap
```

## Configuration

| Fichier | Rôle |
|---------|------|
| `/etc/grul/channel.toml` | Canaux APT (installé par grul-core) |
| `/etc/grul/snap.toml` | Mode snapshots : `disabled` / `auto` / `enabled` |
| `[update] snapshot_before_current` | Demander un snapshot avant upgrades Current |

**fail_open = true** (défaut) : si le snapshot échoue, `grul-update` continue quand même.

## Philosophie

- **grul-core** = expérience GRUL complète sans contrainte filesystem
- **grul-snap** = bonus sécurité pour Btrfs — jamais une dépendance dure
- L'utilisateur choisit ; aucun chemin n'est « inférieur »
