# Use the pinned nixpkgs that has newer Rust
{ pkgs ? import (fetchTarball "https://github.com/NixOS/nixpkgs/archive/nixpkgs-unstable.tar.gz") {} }:
pkgs.mkShell {
  buildInputs = with pkgs; [
    rustc
    cargo
    rust-analyzer
    rustfmt
    clippy
    nodejs_20
    pnpm
    git
    tmux
    jq
    sqlite
    pkg-config
    openssl
  ];

  RUST_BACKTRACE = "1";
  RUSTFLAGS = "-C target-feature=-crt-static";

  shellHook = ''
    echo "HOOP development environment loaded"
    echo "Rust version: $(rustc --version)"
    echo "Node version: $(node --version)"
    echo "pnpm version: $(pnpm --version)"
  '';
}
