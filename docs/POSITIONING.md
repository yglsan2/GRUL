# Positionnement GRUL

## Message public

**GRUL** est une marque — un nom pour une couche d'outils sur Debian.

**Dire** : « Debian Stable, avec des outils en plus pour le dev, les VMs et l'admin — la commande `grul`. »

**Ne pas dire** : acronyme technique du nom GRUL dans la communication utilisateur.

## Usage courant : machines virtuelles

GRUL convient bien aux **VMs** (Proxmox, KVM, cloud, CI) :

- Install : `vm-bootstrap.sh` ou `apt install grul-vm`
- Config : `grul-doctor vm-setup`
- Update : `grul-update upgrade -y`
- Uninstall : `uninstall-grul.sh` sans casser Debian

Voir [docs/VM.md](VM.md).

## Ce qu'on propose (3 idées simples)

### 1. Stabilité Debian

- Même `apt`, mêmes paquets Core, mêmes garanties sécurité
- Pas de rolling release intégral — pas de surprise sur `libc`

### 2. Paquets plus récents, si besoin

- **Current** : navigateur, bureau, pilotes graphiques
- **Edge** : stack dev récente, activable à la main

### 3. Moins de bricolage

- Détection matérielle → profil adapté
- Snapshots avant mises à jour risquées (opt-in)
- Une CLI (`grul`) au lieu de jongler entre plusieurs outils

## Public cible

| Persona | Profil GRUL | Canaux |
|---------|-------------|--------|
| Utilisateur bureau | `desktop-balanced` | Core + Current |
| Développeur | `dev-performance` | Core + Current + Edge |
| Admin serveur | `server-minimal` | Core (+ kernel Current option) |
| **VM / cloud / CI** | **`vm-minimal`** | **Core only, sécurité auto** |
| Gamer Linux | `gaming-latency` | Core + Current (Mesa, kernel) |

## Go / Rust — pourquoi ces langages

Les langages servent à **fiabiliser les outils GRUL**, pas à remplacer le userspace Debian :

- **Rust** : `grul-update`, `grul-snap`, `grul-tune` — outils sensibles, peu de surprises à l'exécution
- **Go** : `grul-install`, `grul-config` — installateur et centre de config (prévu)

Le **démarrage plus rapide** vient surtout de :

- services allégés selon profil
- réglages SSD/NVMe raisonnables
- pas de snaps/flatpak imposés au boot

Pas de réécriture du noyau.

## Nom et identité

- **GRUL** — marque produit (pas d'acronyme public)
- Formulation sobre : **« Debian, un peu simplifié pour le quotidien. »**
- CLI : **`grul`**
