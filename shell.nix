{ pkgs ? import <nixpkgs> { } }:
pkgs.mkShell {
  name = "Rust_Engine";
  # Additional tooling
  buildInputs = with pkgs; [
    rustfmt       # Formatter
    cargo
    rustc
    cmake
    ninja
    wayland
    libxkbcommon
    libGL
  ];

  LD_LIBRARY_PATH="${pkgs.vulkan-loader}/lib:${pkgs.wayland}/lib:${pkgs.libxkbcommon}/lib";
}
