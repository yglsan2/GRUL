//! Aide contextualisée — grul help [commande]

pub fn print_help(topic: Option<&str>) {
    match topic {
        None => print_general(),
        Some("doctor") => print_doctor(),
        Some("update") | Some("upgrade") => print_update(),
        Some("optimize") => print_optimize(),
        Some("vm") => print_vm(),
        Some("rollback") | Some("backup") | Some("restore") => print_snap(),
        Some("repair") => print_repair(),
        Some(other) => {
            println!("Commande « {other} » — voir docs/CLI.md");
            print_general();
        }
    }
}

fn print_general() {
    println!(
        r#"GRUL — aide

Une seule commande pour administrer votre Debian :

  grul doctor          Diagnostic et score santé
  grul update          Mise à jour complète (recommandé)
  grul optimize        Tuning automatique matériel
  grul status          État mises à jour + santé
  grul info            Version et profil
  grul vm detect       Détection hyperviseur
  grul vm optimize     Setup VM optimal
  grul rollback        Restaurer snapshot (Btrfs)
  grul repair          Réparer apt/paquets
  grul clean           Nettoyage apt
  grul uninstall       Retirer la couche GRUL

  grul help <cmd>      Aide détaillée

GRUL n remplace pas apt ni systemd — il les orchestre.
Cahier des charges : docs/CAHIER-DES-CHARGES.md
"#
    );
}

fn print_doctor() {
    println!(
        r#"grul doctor — diagnostic système

  grul doctor              Rapport complet + score
  grul doctor quick        Vérification rapide
  grul doctor vm-setup     Configuration VM (sudo)

Analyse : disque, RAM, profil, mises à jour, guest agent, timer sécurité.
"#
    );
}

fn print_update() {
    println!(
        r#"grul update — mise à jour en une commande

  grul update              Interactive
  grul update -y           Sans confirmation
  grul update --dry-run    Simulation

Équivalent : refresh + upgrade + autoremove + clean (via grul-update + apt).
Canaux : Core (Stable) + Current (opt-in) — voir /etc/grul/channel.toml
"#
    );
}

fn print_optimize() {
    println!(
        r#"grul optimize — tuning automatique

  sudo grul optimize         Applique le profil recommandé (grul-detect → grul-tune)
  sudo grul optimize -y      Sans confirmation

Profils : vm-minimal, desktop-balanced, dev-performance, server-minimal, gaming-latency
Chaque réglage est réversible : sudo grul-tune reset
"#
    );
}

fn print_vm() {
    println!(
        r#"grul vm — machines virtuelles

  grul vm detect           Hyperviseur + profil recommandé
  sudo grul vm optimize    Profil vm-minimal + guest agent + sécurité auto
  grul vm clean            Nettoyage (apt clean)

Prévu v0.5 : compact, clone, export
Guide : docs/VM.md
"#
    );
}

fn print_snap() {
    println!(
        r#"Snapshots (optionnel — paquet grul-snap, Btrfs)

  sudo grul backup         Créer un snapshot
  sudo grul rollback       Revenir au dernier snapshot (+ reboot)
  sudo grul restore --last Idem rollback

Désactivé par défaut sur VMs (canal Core only). Activer : grul-snap enable
"#
    );
}

fn print_repair() {
    println!(
        r#"grul repair — réparation système

  sudo grul repair              dpkg --configure -a + apt -f install
  sudo grul repair --dry-run

Prévu v0.4 : grub, permissions, journaux corrompus
"#
    );
}
