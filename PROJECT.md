# Emplacement du projet

**Répertoire officiel GRUL :**

```
C:\Users\Super\Projects\GRUL
```

Tout le développement GRUL se fait dans ce dépôt — indépendamment des autres projets Cursor.

## Structure

```
GRUL/
├── tools/          # Rust workspace (grul, grul-doctor, …)
├── configs/        # Profils et canaux
├── scripts/        # bootstrap VM, build-debs, uninstall
├── packaging/      # .deb, systemd, cloud-init
├── docs/           # Cahier des charges, VM, CLI
└── repos/          # Dépôts APT (futur)
```

## Démarrer

```bash
cd C:\Users\Super\Projects\GRUL
```

Sur Linux / VM :

```bash
cd ~/Projects/GRUL   # ou clone git vers ~/GRUL
sudo bash scripts/install-tools.sh --vm
grul doctor
```
