# Feuille de route GRUL

Alignée sur le [Cahier des charges v1.0](CAHIER-DES-CHARGES.md).

## v0.1 — Bootstrap (en cours)

- [x] Vision & cahier des charges
- [x] Outils Rust (detect, tune, update, snap, doctor)
- [x] CLI unifiée `grul`
- [x] Profils dont `vm-minimal`
- [x] Paquets `.deb`, scripts VM bootstrap / uninstall
- [ ] ISO minimale Debian + GRUL
- [ ] Dépôt APT public

## v0.2 — Installation

- [x] First boot (`grul-firstboot.service`)
- [x] Détection matérielle + hyperviseur
- [ ] Installateur < 5 min (questions : nom, user, usage)
- [ ] Guest agents auto (VirtualBox, VMware, KVM, Hyper-V)

## v0.3 — CLI

- [x] `grul update` / `upgrade`
- [x] `grul doctor` (+ score /100)
- [x] `grul info` / `help`
- [x] `grul status` / `optimize`
- [x] `grul vm detect` / `optimize`

## v0.4 — Optimisation

- [x] `grul optimize` (via tune)
- [ ] `grul repair` complet (grub, journaux, permissions)
- [x] `grul clean`
- [ ] `grul benchmark`

## v0.5 — VM

- [x] Profil vm-minimal, cloud-init
- [ ] `grul vm compact` / `clone` / `export`
- [ ] Optimisations VirtualBox / VMware / Hyper-V dédiées

## v0.6 — GRUL Center

- [ ] Interface graphique (mises à jour, snapshots, VM, logs)

## v1.0 — Stable

- [ ] Documentation complète par commande
- [ ] Dépôts Stable + Current en production
- [ ] Support officiel environnements dev

## Critères de succès

1. Tutoriel Debian = fonctionne sur GRUL
2. `grul update` = une commande pour être à jour
3. VM template < 2 min après Debian netinst
4. `uninstall-grul.sh` = couche GRUL retirée sans orphelins
5. Aucune optimisation opaque — tout documenté
