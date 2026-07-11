//! Informations système — grul info

use grul_common::{load_applied_state, vm::detect_vm};
use std::fs;

pub fn print_info() {
    println!("GRUL");
    println!("====");
    println!("Version outils : {}", env!("CARGO_PKG_VERSION"));
    print_file_line("/etc/grul/release", "Release GRUL");
    print_debian();
    print_profile();
    print_vm();
    println!();
    println!("Documentation : docs/CAHIER-DES-CHARGES.md");
    println!("Aide          : grul help");
}

fn print_file_line(path: &str, label: &str) {
    if let Ok(s) = fs::read_to_string(path) {
        let line = s.lines().next().unwrap_or("").trim();
        if !line.is_empty() {
            println!("{label} : {line}");
        }
    }
}

fn print_debian() {
    if let Ok(s) = fs::read_to_string("/etc/os-release") {
        for line in s.lines() {
            if line.starts_with("PRETTY_NAME=") {
                let v = line.trim_start_matches("PRETTY_NAME=").trim_matches('"');
                println!("Système       : {v}");
                break;
            }
        }
    }
}

fn print_profile() {
    match load_applied_state() {
        Some(s) => println!("Profil actif  : {}", s.profile_id),
        None => println!("Profil actif  : (aucun — grul optimize)"),
    }
}

fn print_vm() {
    let vm = detect_vm();
    if vm.is_virtual {
        println!("Environnement : VM — {}", vm.kind.label());
    } else {
        println!("Environnement : bare metal");
    }
}
