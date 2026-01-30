{
  description = "Astrological CPU Scheduler";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        packages = {
          scx_horoscope = pkgs.callPackage ./nix/package.nix { };
          default = self.packages.${system}.scx_horoscope;
        };

        nixosModules = {
          scx_horoscope = import ./nix/module.nix;
          default = self.nixosModules.scx_horoscope;
        };
      }
    );
}

