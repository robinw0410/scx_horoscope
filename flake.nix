{
  description = "Astrological CPU Scheduler";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, ... }:
    (flake-utils.lib.eachDefaultSystem (system:
      let pkgs = nixpkgs.legacyPackages.${system};
      in {
        packages.scx_horoscope = pkgs.callPackage ./nix/package.nix { };
        packages.default = self.packages.${system}.scx_horoscope;
      })) // {
        nixosModules.default = { config, lib, pkgs, ... }: {
          imports = [ ./nix/module.nix ];

          services.scx_horoscope.package = lib.mkDefault
            self.packages.${pkgs.stdenv.hostPlatform.system}.scx_horoscope;
        };
      };
}

