{
  description = "A very basic flake";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };
  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }:
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

          myRustBuild = rustPlatform.buildRustPackage {
            pname = "heosd"; # make this what ever your cargo.toml package.name is
            version = "0.1.0";
            src = ./.; # the folder with the cargo.toml
            cargoLock = {
                lockFile = ./Cargo.lock;
            };
            nativeBuildInputs = [ pkgs.pkg-config ];
          };

        in {
          defaultPackage = myRustBuild;
          #
          # TODO this is a bash. I don't want a bash!
          #
          devShell = pkgs.mkShell {
            buildInputs =
              [ pkgs.fish (rustVersion.override { extensions = [ "rust-src" ]; }) ];
          };
        });
}
