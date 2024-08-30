{ pkgs ? import <nixpkgs> { } }:
pkgs.mkShell {
  name = "Rust_Engine";
  # Additional tooling
  buildInputs = with pkgs; [
    rustfmt       # Formatter
    cargo
    rustc
    vulkan-tools
    vulkan-loader
    vulkan-validation-layers
    cmake
    ninja
    python3
  ];

  LD_LIBRARY_PATH="${pkgs.vulkan-loader}/lib:${pkgs.vulkan-validation-layers}/lib";
  VULKAN_SDK = "${pkgs.vulkan-headers}";
  VK_LAYER_PATH = "${pkgs.vulkan-validation-layers}/share/vulkan/explicit_layer.d";
}
