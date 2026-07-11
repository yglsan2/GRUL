# Construction des dépôts APT GRUL (Phase 2+)

Prérequis sur une machine Debian :

- `reprepro`
- `debsign` / clé GPG projet
- CI (GitHub Actions ou GitLab CI) pour rebuilds Current

## Arborescence cible

```
repos/
├── conf/
│   └── distributions      # config reprepro
├── incoming/
└── published/             # miroir public
```

## Étapes (outline)

1. Signer les `.deb` GRUL Core (`grul-detect`, etc.)
2. `reprepro includedeb grul-core grul-detect_*.deb`
3. Rebuild paquet Current dans chroot isolé
4. Tests d'intégration : `apt upgrade` sur VM Debian + repo GRUL

Scripts à venir dans `repos/scripts/`.
