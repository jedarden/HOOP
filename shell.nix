{ pkgs ? import <nixpkgs> {} }:
let
  rustToolchain = pkgs.rust-bin.stable.latest.default.override {
    extensions = [ "rust-src" "rust-analyzer" "rustfmt" "clippy" ];
  };
in
pkgs.mkShell {
  buildInputs = with pkgs; [
    rustToolchain
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

  shellHook = ''
    echo "HOOP development environment loaded"
    echo "Rust version: $(rustc --version)"
    echo "Node version: $(node --version)"
    echo "pnpm version: $(pnpm --version)"
  '';
}
