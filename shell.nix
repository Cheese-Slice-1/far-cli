{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShellNoCC {
  name = "farcli-shell";
  packages = with pkgs; [
    cargo
    rustc
    clippy
    rustfmt
  ];
}

