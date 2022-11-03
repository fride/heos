{
  description = "A very basic flake";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };
  outputs = { self, nixpkgs, flake-utils, rust-overlay, naersk, ... }:
      flake-utils.lib.eachDefaultSystem (system:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs { inherit system overlays; };
          #rustVersion = pkgs.rust-bin.stable.latest.default; # this won't work?
          # See https://github.com/oxalica/rust-overlay
          rustVersion = pkgs.rust-bin.nightly."2022-08-08".default; # this won't work?

          rustPlatform = pkgs.makeRustPlatform {
            cargo = rustVersion;
            rustc = rustVersion;
          };

            # Use naersk to build the rust app.
            # see https://www.tweag.io/blog/2022-09-22-rust-nix/

            naerskLib = pkgs.callPackage naersk {};

            heosd = naerskLib.buildPackage {
                name = "heosd";
                src = ./.;
                cargoBuildOptions = x: x ++ [ "-p" "app" ];
                nativeBuildInputs = [ pkgs.pkg-config ];
            };
        in {
          defaultPackage = heosd;
          #
          # TODO this is a bash. I don't want a bash!
          #
          devShell = pkgs.mkShell {
            buildInputs =
              [ pkgs.fish (rustVersion.override { extensions = [ "rust-src" ]; }) ];
          };
        });
}
