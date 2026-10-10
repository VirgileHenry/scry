{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell rec {
  nativeBuildInputs = [
    pkgs.pkg-config
  ];

  buildInputs = [
    pkgs.wayland
    pkgs.libxkbcommon
    pkgs.fontconfig
    pkgs.freetype
    pkgs.expat
    pkgs.vulkan-loader
    pkgs.libpulseaudio
    pkgs.linux-pam
  ];

  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
}
