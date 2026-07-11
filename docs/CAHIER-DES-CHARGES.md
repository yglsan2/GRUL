# Cahier des charges — GRUL

**Version 1.0 – Vision du projet**

> **GRUL** est une marque — le nom du projet et de ses outils sur Debian.  
> Ce document ne définit pas d'acronyme public.

---

## 1. Présentation

GRUL est une distribution GNU/Linux basée sur Debian, pensée pour :

- les machines virtuelles ;
- les environnements de développement ;
- les laboratoires ;
- les postes de travail des développeurs ;
- les étudiants en informatique.

GRUL n'a pas vocation à remplacer Debian.  
C'est une surcouche légère : les mêmes paquets, les mêmes habitudes, avec moins de friction pour l'admin courante.

## 2. Vision

Créer un système :

- extrêmement simple à installer ;
- rapide à démarrer ;
- rapide à mettre à jour ;
- rapide à restaurer ;
- rapide à supprimer ;
- optimisé automatiquement ;
- totalement compatible Debian.

## 3. Objectifs

Le temps passé à administrer le système doit être réduit au minimum.

L'utilisateur doit passer son temps à **développer**, **tester**, **apprendre** — et non à maintenir son système.

## 4. Public cible

**Priorité :** développeurs, DevOps, administrateurs système, étudiants, enseignants, laboratoires.

**Secondaire :** utilisateurs avancés.

**Plus tard :** entreprises.

## 5. Philosophie — six principes

| Principe | Signification |
|----------|---------------|
| **Simplicité** | Une seule façon de faire les choses |
| **Rapidité** | Opérations courtes, peu d'étapes |
| **Automatisation** | Le système configure ce qui peut l'être |
| **Compatibilité** | Un tutoriel Debian fonctionne sur GRUL |
| **Transparence** | Chaque optimisation documentée |
| **Sécurité** | Aucune optimisation ne compromet la sécurité |

## 6. Ce que GRUL n'est pas

GRUL n'est **pas** : Arch Linux, Ubuntu modifiée, distro gaming, distro expérimentale.

GRUL ne **remplacera jamais** : bash, apt, systemd. Il les **enrichit**.

## 7. Architecture cible

```
Debian Stable
        │
        ▼
    GRUL Core
        │
        ├───────────────┐
        │               │
   grul-cli        grul-daemon
        │               │
        └───────┬───────┘
                │
          grul-center
```

## 8. Technologies (implémentation interne)

| Langage | Usage |
|---------|--------|
| **Rust** | Outils système, bibliothèques, sécurité, performance |
| **Go** | Services, réseau, API, CLI lourde (installateur, center) |
| **Bash** | Compatibilité uniquement |

## 9. Rolling release partiel

Base : **Debian Stable**.

Mises à jour rapides (canal **Current**) pour : kernel, Mesa, Firefox, Chromium, Docker, Podman, Go, Rust, LLVM, GCC, Git.

## 10. Installation

- **Objectif :** moins de 5 minutes.
- **Mode :** minimal.
- **Questions :** nom, utilisateur, mot de passe, usage (VM / Dev / Bureau / Serveur).
- **Tout le reste :** automatique.

## 11. Détection automatique (premier démarrage)

Détection : CPU, RAM, SSD, GPU, batterie, UEFI, hyperviseur, connexion.

| Hyperviseur | Action automatique |
|-------------|-------------------|
| VirtualBox | Guest Additions |
| VMware | open-vm-tools |
| KVM | VirtIO |
| Hyper-V | Services adaptés |

## 12. GRUL CLI — `grul`

```bash
grul <commande>
```

| Commande | Rôle |
|----------|------|
| `doctor` | Diagnostic et score santé |
| `update` | Mise à jour système complète |
| `upgrade` | Alias upgrade |
| `repair` | Réparation apt/grub/services |
| `benchmark` | Mesures performance |
| `backup` / `restore` | Snapshots |
| `optimize` | Tuning automatique |
| `drivers` | Pilotes / guest agents |
| `security` | État sécurité |
| `logs` | Journaux utiles |
| `services` | Services système |
| `packages` | Paquets / canaux |
| `vm` | Outils VM |
| `clean` | Nettoyage |
| `status` | État global |
| `info` | Informations système |
| `help` | Aide contextualisée |

## 13. `grul doctor`

Analyse : disque, mémoire, CPU, GPU, température, réseau, sécurité, journaux, services.

Sortie : **score /100** + **3 optimisations recommandées**.

## 14. `grul optimize`

Optimise : SSD, RAM, CPU, swap, scheduler, TRIM, zram, PipeWire, journal.

## 15. `grul update`

Une commande = `apt update` + `upgrade` + `autoremove` + `clean` + vérification + snapshot (optionnel) + reboot si nécessaire.

## 16. `grul repair`

Répare : apt, grub, services, permissions, journaux, paquets cassés.

## 17. `grul vm`

| Sous-commande | Rôle |
|---------------|------|
| `detect` | Détection hyperviseur |
| `optimize` | Tuning VM |
| `compact` | Réduction image disque |
| `clone` | Préparation clone |
| `export` | Export image |
| `clean` | Nettoyage VM |

## 18. Optimisations VM

Objectifs : moins de RAM, CPU, disque ; boot et arrêt plus rapides.

## 19. Snapshots

Snapshot automatique avant grosses mises à jour. Rollback : `grul rollback`.

## 20. Benchmarks

`grul benchmark` — CPU, RAM, SSD, réseau, temps boot/shutdown.

## 21. Interface graphique — GRUL Center

Mises à jour, utilisateurs, services, snapshots, VM, réseau, pilotes, logs, performances.

## 22. Dépôts officiels

- **Stable** (Core — Debian + outils GRUL)
- **Current** (apps et outils dev récents)

## 23. Documentation

Chaque commande : `grul help <cmd>`.  
Chaque erreur : explication + solution + commande à exécuter.

## 24. Objectifs de performance

| Métrique | Cible |
|----------|-------|
| Installation | < 5 min |
| Premier boot (SSD) | < 10 s |
| Mise à jour | une commande |
| Création VM | < 2 min |
| Réparation | une commande |
| Suppression | aucun fichier orphelin |

## 25. Organisation du code

| Composant | Rôle |
|-----------|------|
| `grul-common` | Types, profils, détection VM |
| `grul-cli` | CLI principale `grul` |
| `grul-center` | Interface graphique |
| `grul-daemon` | Services arrière-plan |
| `grul-update` | Canaux et apt |
| `grul-doctor` | Diagnostic |
| `grul-vm` | Outils VM |
| `grul-repair` | Réparation |
| `grul-security` | Sécurité |
| `grul-benchmark` | Benchmarks |
| `grul-backup` | Snapshots |
| `grul-installer` | Installateur |

Chaque composant majeur : dépôt Git indépendant (cible long terme).

## 26. Feuille de route produit

| Version | Contenu |
|---------|---------|
| **v0.1 — Bootstrap** | Debian + dépôt + doc + ISO minimale |
| **v0.2 — Installation** | Installateur, first boot, détection |
| **v0.3 — CLI** | `grul update`, `doctor`, `info`, `help` |
| **v0.4 — Optimisation** | `optimize`, `repair`, `clean` |
| **v0.5 — VM** | Hyperviseurs, compact, export |
| **v0.6 — Center** | GRUL Center v1 |
| **v1.0 — Stable** | Doc complète, dépôts Stable/Current, dev officiel |

## 27. Vision long terme

- **GRUL Cloud** — déploiement cloud simplifié
- **GRUL Containers** — Docker / Podman intégrés
- **GRUL SDK** — bibliothèque commune
- **GRUL Builder** — images custom (VM, cloud, RPi…)
- **GRUL AI Assistant** — aide locale, erreurs, optimisations

---

*Document de référence — aligné sur l'implémentation dans `docs/ROADMAP.md` et `docs/CLI.md`.*
