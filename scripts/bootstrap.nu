# bootstrap.nu
#
# Nushell script to (re)create the Rust workspace skeleton for fsrs.
#
# Usage (from repo root):
#
#   use scripts/bootstrap.nu *
#   bootstrap
#
# This script is intentionally idempotent; it creates files if they don't
# exist, but will not overwrite existing Cargo.toml or src files.

export def bootstrap [] {
  let root = (pwd)

  # Ensure rust/ directory exists
  let rust_dir = ($root | path join "rust")
  if not ($rust_dir | path exists) {
    mkdir $rust_dir
  }

  cd $rust_dir

  # Create workspace Cargo.toml if missing
  let ws_toml = "Cargo.toml"
  if not ($ws_toml | path exists) {
    $"[workspace]
members = [
  \"crates/fsrs-frontend\",
  \"crates/fsrs-vm\",
  \"crates/fsrs-demo\",
]
resolver = \"2\"
" | save $ws_toml
  }

  # Create crates directory
  let crates_dir = ($rust_dir | path join "crates")
  if not ($crates_dir | path exists) {
    mkdir $crates_dir
  }

  # Helper to create crate if missing
  def ensure_crate [name: string, kind: string] {
    let crate_dir = ($crates_dir | path join $name)
    if not ($crate_dir | path exists) {
      mkdir $crate_dir
    }

    let cargo_file = ($crate_dir | path join "Cargo.toml")
    if not ($cargo_file | path exists) {
      if $kind == "lib" {
        $"[package]
name = \"($name)\"
version = \"0.1.0\"
edition = \"2021\"

[lib]
path = \"src/lib.rs\"

[dependencies]
" | save $cargo_file
      } else if $kind == "bin" {
        $"[package]
name = \"($name)\"
version = \"0.1.0\"
edition = \"2021\"

[dependencies]
fsrs-frontend = { path = \"../fsrs-frontend\" }
fsrs-vm = { path = \"../fsrs-vm\" }
" | save $cargo_file
      }
    }

    let src_dir = ($crate_dir | path join "src")
    if not ($src_dir | path exists) {
      mkdir $src_dir
    }

    if $kind == "lib" {
      let lib_file = ($src_dir | path join "lib.rs")
      if not ($lib_file | path exists) {
        $"// (fsrs) crate: ($name)
// This is a stub file. See docs/CLAUDE_CODE_NOTES.md for implementation steps.

pub fn placeholder() {
    println!(\"($name) placeholder\");
}
" | save $lib_file
      }
    } else if $kind == "bin" {
      let main_file = ($src_dir | path join "main.rs")
      if not ($main_file | path exists) {
        $"// fsrs-demo: demo host for the Mini-F# VM
// For now, this just prints a stub message and exits.

fn main() {
    println!(\"fsrs-demo stub. See docs/CLAUDE_CODE_NOTES.md for next steps.\");
}
" | save $main_file
      }
    }
  }

  ensure_crate "fsrs-frontend" "lib"
  ensure_crate "fsrs-vm" "lib"
  ensure_crate "fsrs-demo" "bin"

  cd $root

  print \"fsrs Rust workspace bootstrapped (or already present) under ./rust\"
}
