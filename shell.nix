{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    cargo
    rustc
    rustfmt
    clippy
    pkg-config
  ];

  buildInputs = with pkgs; [
    alsa-lib
    libGL
    libx11
    libxcursor
    libxi
    libxrandr
    libxkbcommon
    udev
    vulkan-loader
    wayland
    wayland-protocols
    lld
  ];

  shellHook = ''
    export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${pkgs.lib.makeLibraryPath [
      pkgs.alsa-lib
      pkgs.libGL
      pkgs.libx11
      pkgs.libxcursor
      pkgs.libxi
      pkgs.libxrandr
      pkgs.libxkbcommon
      pkgs.udev
      pkgs.vulkan-loader
      pkgs.wayland
      pkgs.wayland-protocols
      pkgs.lld
    ]}"
  '';
}
