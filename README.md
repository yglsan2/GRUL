# GRUL

**La meilleure Debian pour développer, expérimenter et administrer.**

GRUL est une surcouche sur **Debian Stable** — pas un remplacement. Bash, apt et systemd restent au cœur du système.

> *Une commande. Zéro friction.*

```bash
grul doctor      # diagnostic + score /100
sudo grul update # tout mettre à jour
sudo grul optimize
grul status
```

## Pour qui ?

VMs · dev · labos · étudiants · DevOps — voir [Cahier des charges](docs/CAHIER-DES-CHARGES.md).

## Modèle

| Couche | Rôle |
|--------|------|
| **Debian Stable** | Base, sécurité, compatibilité |
| **GRUL Core** | Outils, politique, profils |
| **GRUL Current** | Apps & outils dev récents (opt-in) |

## Commande unique : `grul`

| Commande | Action |
|----------|--------|
| `grul update` | Mise à jour complète |
| `grul doctor` | Santé système |
| `grul optimize` | Tuning auto |
| `grul vm optimize` | Setup VM |
| `grul help` | Aide |

Référence complète : [docs/CLI.md](docs/CLI.md)

## VM en 2 minutes

```bash
sudo bash scripts/vm-bootstrap.sh
grul doctor
sudo grul update -y
```

[Guide VM](docs/VM.md)

## Installation

```bash
bash scripts/build-debs.sh
sudo dpkg -i dist/debs/grul-*.deb
# ou dev :
sudo bash scripts/install-tools.sh --vm
```

[Désinstallation propre](docs/INSTALL.md) · [Feuille de route](docs/ROADMAP.md)

## Licence

GPL-3.0-or-later (outils GRUL). Paquets Debian : licences respectives.
