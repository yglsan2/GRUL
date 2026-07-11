# Audit — séparation des projets

**Date :** vérification post-session  
**Répertoire GRUL officiel :** `C:\Users\Super\Projects\GRUL`

## Résultat

| Vérification | Statut |
|--------------|--------|
| Code GRUL uniquement dans `Projects/GRUL` | ✅ |
| Références PloufPlouf dans GRUL (hors doc audit) | ✅ Aucune |
| Références GRUL dans Ploufplouf | ✅ Aucune |
| Git GRUL indépendant | ✅ Dépôt `main` local (`aa7c8d4`) |
| Git Ploufplouf indépendant | ✅ Dépôt séparé |

## Périmètres

```
C:\Users\Super\Projects\GRUL\     ← GRUL uniquement
C:\Users\Super\Ploufplouf\       ← autre projet (aucun fichier grul-*)
```

## Règle

Ne **jamais** copier de fichiers GRUL dans Ploufplouf ou inversement.  
Ouvrir Cursor sur `Projects/GRUL` pour tout travail GRUL.

## Intégrité GRUL (inventaire)

| Composant | Présent |
|-----------|---------|
| `tools/grul-cli` (commande `grul`) | ✅ |
| `tools/grul-detect` | ✅ |
| `tools/grul-tune` | ✅ |
| `tools/grul-update` | ✅ |
| `tools/grul-snap` | ✅ |
| `tools/grul-doctor` (+ drivers, repair) | ✅ |
| `tools/grul-common` (+ drivers.rs) | ✅ |
| Profils (5 dont vm-minimal) | ✅ |
| `scripts/grul-install.sh` | ✅ |
| Scripts bootstrap / uninstall | ✅ |
| Docs (CdC, VM, CLI, ROADMAP) | ✅ |
| Packaging .deb / systemd | ✅ |

## Actions si mélange détecté

1. Supprimer les fichiers hors périmètre
2. Ne pas committer depuis le mauvais dépôt git
3. Relancer `git -C Projects/GRUL status`
