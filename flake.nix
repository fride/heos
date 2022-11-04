{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    crane.inputs.nixpkgs.follows = "nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
    flake-utils.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, crane, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };

        # Common derivation arguments used for all builds
        commonArgs = {
          src = craneLib.cleanCargoSource ./.;

          buildInputs = with pkgs; [
            # Add extra build inputs here, etc.
            pkgs.libiconv
          ];

          nativeBuildInputs = with pkgs; [
            # Add extra native build inputs here, etc.
            # pkg-config
          ];
        };

        rustToolchain = pkgs.pkgsBuildHost.rust-bin.stable.nightly."2022-08-08".override {
          targets = [ "x86_64-apple-darwin" ];
        };
        craneLib = crane.lib.${system};
        src = ./.;

        # Build *just* the cargo dependencies, so we can reuse
        # all of that work (e.g. via cachix) when running in CI
        cargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
          inherit src;
        });

        # Run clippy (and deny all warnings) on the crate source,
        # resuing the dependency artifacts (e.g. from build scripts or
        # proc-macros) from above.
        #
        # Note that this is done as a separate derivation so it
        # does not impact building just the crate by itself.
        heosd-clippy = craneLib.cargoClippy {
          inherit cargoArtifacts src;
          cargoClippyExtraArgs = "-- --deny warnings";
        };

        # Build the actual crate itself, reusing the dependency
        # artifacts from above.
        heosd = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts src;
        });

        # Also run the crate tests under cargo-tarpaulin so that we can keep
        # track of code coverage
        heosd-coverage = craneLib.cargoTarpaulin( commonArgs // {
          inherit cargoArtifacts src;
        });
        configuration = ./configuration;
      in
      {
        defaultPackage = heosd;
        # Add the config directory
        configuration = configuration;
        checks = {
         inherit
           # Build the crate as part of `nix flake check` for convenience
           heosd
           heosd-clippy
           heosd-coverage;
        };
      });
}
