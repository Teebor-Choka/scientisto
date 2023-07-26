{ pkgs ? import <nixpkgs> { }, ... }:
let
  linuxPkgs = with pkgs; lib.optional stdenv.isLinux (
    inotifyTools
  );
  macosPkgs = with pkgs; lib.optional stdenv.isDarwin (
    with darwin.apple_sdk.frameworks; [
      # macOS file watcher support
      CoreFoundation
      CoreServices
    ]
  );
  devPkgs = with pkgs; [
    ## rust
    (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
  ];
in
with pkgs;
mkShell {
  buildInputs = devPkgs;
}
