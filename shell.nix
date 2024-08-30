{ pkgs ? import <nixpkgs> { } }:
pkgs.mkShell {
  name = "Rust_Engine";
  # Additional tooling
  buildInputs = with pkgs; [
    rustfmt       # Formatter
    cargo
    rustc
  ];
}
