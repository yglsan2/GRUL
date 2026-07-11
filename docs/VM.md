# GRUL pour machines virtuelles

GRUL est surtout pensé pour les **VMs** (Proxmox, KVM, cloud, labos) : Debian Stable en dessous, quelques outils GRUL au-dessus.

## Pourquoi GRUL en VM ?

| Besoin | GRUL |
|--------|------|
| Install rapide | `vm-bootstrap.sh` ou cloud-init |
| Config auto | `grul-doctor vm-setup` (1 commande) |
| Minimal | Profil `vm-minimal` + canal Core only |
| Mises à jour | `grul-update upgrade -y` |
| Sécurité auto | `grul-update-security.timer` |
| Désinstall propre | `uninstall-grul.sh` — Debian intact |
| Optimisation | Détection KVM/QEMU/cloud + tuning |

## Démarrage en 2 minutes (Proxmox, libvirt, VirtualBox)

### 1. Créer une VM Debian minimale

- Debian 12/13 netinst, **sans bureau**
- 1–2 vCPU, 1–2 Go RAM, 8 Go disque suffisent
- SSH activé

### 2. Bootstrap GRUL

```bash
git clone https://github.com/grul-project/grul.git /opt/grul
cd /opt/grul
sudo bash scripts/vm-bootstrap.sh
```

Ou avec paquets `.deb` :

```bash
bash scripts/build-debs.sh
sudo dpkg -i dist/debs/grul-*.deb
sudo grul-doctor vm-setup
```

### 3. Vérifier

```bash
grul-doctor
grul-detect
grul-update status
```

## Cloud-init (AWS, GCP, Azure, Proxmox)

Fichier : `packaging/cloud-init/grul-vm.yaml`

Montez le repo GRUL dans `/opt/grul/GRUL` ou copiez `vm-bootstrap.sh` dans `/tmp/` avant le premier boot.

## Profil `vm-minimal`

Appliqué automatiquement sur les VMs détectées :

- Services bureau masqués (bluetooth, cups, avahi…)
- `swappiness` adaptée aux petites VMs
- `fstrim` activé
- Canal **Current/Edge désactivés** (stabilité max)
- **Sécurité Debian auto** via timer

## Commandes essentielles

```bash
grul doctor              # santé + score /100
grul doctor quick        # check rapide
sudo grul vm optimize    # setup VM (ou grul doctor vm-setup)
sudo grul update -y      # mettre à jour tout
sudo grul optimize       # tuning auto
grul status              # état global
grul info                # version + profil
grul help                # aide
sudo bash /usr/share/grul/scripts/uninstall-grul.sh
```

## Premier démarrage automatique

```bash
sudo systemctl enable grul-firstboot.service
```

Au premier boot : VM → `vm-setup`, bare metal → `grul-tune --auto`.

## Templates Proxmox / libvirt

1. Installez GRUL une fois sur une VM « golden »
2. `grul-doctor` → tout vert
3. Arrêtez la VM, convertissez en template
4. Chaque clone hérite de GRUL ; `grul-detect` adapte le profil si besoin

## Ce que GRUL ne fait pas (volontairement)

- Ne remplace pas Debian par une image immuable (bootc/rpm-ostree)
- N’impose pas Btrfs (snapshots = opt-in)
- Ne force pas Current sur les VMs (modernité = opt-in via canal)

Debian reste **100 % compatible** — GRUL est une couche amovible.
