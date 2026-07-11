# Positionnement GRUL

## Message public

**GRUL** est une marque — qualité, simplicité, confiance sur Debian.

**Dire** : « La meilleure Debian pour développer, expérimenter et administrer — une commande : `grul`. »

**Ne pas dire** : acronyme technique du nom GRUL dans la communication utilisateur.

## Cas d'usage phare : machines virtuelles

GRUL cible en priorité les **VMs** (Proxmox, KVM, cloud, CI) :

- Install : `vm-bootstrap.sh` ou `apt install grul-vm`
- Config : `grul-doctor vm-setup` (une commande)
- Update : `grul-update upgrade -y`
- Uninstall : `uninstall-grul.sh` sans casser Debian

Voir [docs/VM.md](VM.md).

## Proposition de valeur (3 piliers)

### 1. Stabilité Debian

- Même `apt`, mêmes paquets Core, mêmes garanties sécurité
- Pas de rolling release intégral — pas de surprise sur `libc`

### 2. Modernité contrôlée

- **Current** : navigateur, bureau bureautique, pilotes graphiques récents
- **Edge** : stack dev à la pointe, activable en un clic

### 3. Administration intelligente

- Détection matérielle → profil adapté
- Snapshots avant mises à jour risquées
- Un seul outil (`grul-config` / `grul-doctor`) au lieu de 15 tutoriels

## Public cible

| Persona | Profil GRUL | Canaux |
|---------|-------------|--------|
| Utilisateur bureau | `desktop-balanced` | Core + Current |
| Développeur | `dev-performance` | Core + Current + Edge |
| Admin serveur | `server-minimal` | Core (+ kernel Current option) |
| **VM / cloud / CI** | **`vm-minimal`** | **Core only, sécurité auto** |
| Gamer Linux | `gaming-latency` | Core + Current (Mesa, kernel) |

## Go / Rust — argument honnête

Les langages servent à **qualité du code GRUL**, pas à remplacer le userspace Debian :

- **Rust** : `grul-update`, `grul-snap`, `grul-tune` — fiabilité, pas de segfault sur une MAJ système
- **Go** : `grul-install`, `grul-config` — rapidité de dev, binaires portables pour l'installateur

Le **démarrage plus rapide** vient surtout de :

- services allégés selon profil
- SSD/NVMe tuning
- pas de snaps/flatpak imposés au boot

Pas de la réécriture du noyau en Rust.

## Nom et identité

- **GRUL** — marque produit (pas d'acronyme public)
- Slogan : **« La meilleure Debian pour développer. »**
- CLI : **`grul`** — une seule commande
