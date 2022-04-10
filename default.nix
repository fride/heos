let
  sources = import ./nix/sources.nix;
  pkgs = import sources.nixpkgs {};
  naersk = pkgs.callPackage sources.naersk {};
in
 naersk.buildPackage {
        src = ./.;
        name="rusty-heos";
  }
