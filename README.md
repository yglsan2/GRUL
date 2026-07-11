# GRUL

**Debian amélioré, pour en faire une distribution, à la fois stable, rapide et moderne avec des surcouches de Go et de Rust, pour gagner en rapidité d'exécution et en modernité.**

L'objectif de GRUL est de devenir la Distribution Linux la plus facile à utiliser pour les projets d'Administration Linux, de Dev et de DevOps, que ce soit en Desktop, ou en Machine Virtuelle. Pour ce faire, il faut donc que la configuration du système soit la plus rapide et efficace possible, de l'installation à la configuration (qui doivent se faire d'une manière limpide et automatisées), jusqu'aux mises à jour, aux utilisations et à la désinstallation, qui doivent aussi être limpides, faciles et très rapides) . 

GRUL ne remplace pas Debian — il s'appuie dessus. Bash, apt et systemd restent là, comme vous les connaissez. L'idée : moins de friction au quotidien, surtout en VM, en labo ou pour du dev.

```bash
grul doctor       # un coup d'œil sur l'état du système
sudo grul update  # mises à jour en une commande
sudo grul optimize
grul status
```

## C'est quoi, concrètement ?

Une surcouche légère : profils de tuning, détection matérielle/VM, guest agents, une CLI unifiée (`grul`).  
Pas de distro exotique — juste Debian, un peu mieux rangé pour ceux qui enchaînent les installs et les petits serveurs.

Public visé : VMs, machines de dev, labos, étudiants, DevOps.  
Détails : [Cahier des charges](docs/CAHIER-DES-CHARGES.md).

## Comment c'est organisé

| Couche | Rôle |
|--------|------|
| **Debian Stable** | Base, sécurité, compatibilité |
| **GRUL Core** | Outils, profils, politique par défaut |
| **GRUL Current** | Paquets plus récents, si vous le voulez (opt-in) |

## CLI `grul`

| Commande | Action |
|----------|--------|
| `grul update` | Mise à jour complète |
| `grul doctor` | Diagnostic et score santé |
| `grul optimize` | Tuning selon le profil |
| `grul vm optimize` | Setup rapide pour une VM |
| `grul install` | Installateur interactif |
| `grul help` | Aide |

Liste complète : [docs/CLI.md](docs/CLI.md)

## Démarrer sur une VM

```bash
sudo bash scripts/vm-bootstrap.sh
grul doctor
sudo grul update -y
```

Ou l'installateur guidé : `sudo grul install`  
Guide : [docs/VM.md](docs/VM.md)

## Installation

```bash
# Paquets .deb
bash scripts/build-debs.sh
sudo dpkg -i dist/debs/grul-*.deb

# Depuis les sources (dev / test)
sudo bash scripts/install-tools.sh --vm
```

[Désinstallation propre](docs/INSTALL.md) · [Feuille de route](docs/ROADMAP.md)

## Licence

GPL-3.0-or-later (outils GRUL). Les paquets Debian embarqués gardent leurs licences respectives.

---

Projet jeune, en cours de construction — les retours et les PR sont les bienvenus.
