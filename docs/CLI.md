# GRUL CLI — `grul`

CLI principale, décrite dans le [Cahier des charges](CAHIER-DES-CHARGES.md).

## Principe

```bash
grul <commande> [options]
```

Une façon d'administrer le système. Les outils internes (`grul-detect`, `grul-tune`, …) restent disponibles pour les scripts et le packaging.

## État d'implémentation (v0.3)

| Commande | Statut | Implémentation |
|----------|--------|----------------|
| `grul doctor` | ✅ | `grul-doctor` + score /100 |
| `grul doctor quick` | ✅ | |
| `grul doctor vm-setup` | ✅ | |
| `grul update` | ✅ | refresh + upgrade + autoremove + clean |
| `grul upgrade` | ✅ | alias de update |
| `grul optimize` | ✅ | `grul-tune apply --auto` |
| `grul status` | ✅ | update status + doctor quick |
| `grul info` | ✅ | version, Debian, profil, VM |
| `grul help` | ✅ | aide par commande |
| `grul vm detect` | ✅ | `grul-detect` |
| `grul vm optimize` | ✅ | `grul-doctor vm-setup` |
| `grul vm clean` | ✅ | apt clean |
| `grul rollback` | ✅ | `grul-snap rollback` (opt-in) |
| `grul backup` / `restore` | ✅ | via grul-snap |
| `grul clean` | ✅ | autoremove + clean |
| `grul repair` | ✅ | `grul-doctor repair` (dpkg, apt -f, check) |
| `grul drivers` | ✅ | statut VM + `grul drivers install` |
| `grul install` | ✅ | `scripts/grul-install.sh` interactif |
| `grul security` | 🔶 | aperçu sécurité |
| `grul logs` | 🔶 | journalctl err |
| `grul services` | 🔶 | tune status + failed units |
| `grul packages` | ✅ | update status |
| `grul uninstall` | ✅ | guide + script |
| `grul benchmark` | 📋 | v0.4 |
| `grul vm compact/clone/export` | 📋 | v0.5 |

Légende : ✅ disponible · 🔶 partiel · 📋 planifié

## Exemples VM (usage fréquent)

```bash
sudo bash scripts/vm-bootstrap.sh
grul info
grul vm detect
sudo grul vm optimize
grul doctor
sudo grul update -y
```

## Désinstallation

```bash
grul uninstall          # guide
sudo bash /usr/share/grul/scripts/uninstall-grul.sh
```

Debian reste intact — aucun fichier orphelin côté couche GRUL (sauvegarde dans `/var/lib/grul/backup/`).
