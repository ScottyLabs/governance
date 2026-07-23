{
  description = "ScottyLabs Governance";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    terranix = {
      url = "github:terranix/terranix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [ inputs.terranix.flakeModule ];
      systems = [ "x86_64-linux" ];

      perSystem =
        { config, pkgs, ... }:
        let
          gov = import ./nix/data.nix {
            inherit (pkgs) lib;
            dataDir = ./data;
          };
          genDir = ./nix/generators;
          genModules = builtins.concatMap (
            f:
            let
              fn = import (genDir + "/${f}");
              args = builtins.intersectAttrs (builtins.functionArgs fn) {
                inherit (pkgs) lib;
                inherit gov;
              };
            in
            builtins.attrValues (fn args)
          ) (builtins.filter (pkgs.lib.hasSuffix ".nix") (builtins.attrNames (builtins.readDir genDir)));
        in
        {
          terranix.exportDevShells = false;

          terranix.terranixConfigurations.governance = {
            terraformWrapper.package = pkgs.opentofu;
            modules = [ (import ./nix/base.nix) ] ++ genModules;
          };

          packages.atlantis-yaml = pkgs.writeText "atlantis.yaml" (
            builtins.toJSON {
              version = 3;
              parallel_plan = true;
              parallel_apply = true;
              projects = pkgs.lib.mapAttrsToList (name: _: {
                inherit name;
                dir = ".";
                workspace = name;
                autoplan.when_modified = [
                  "flake.nix"
                  "flake.lock"
                  "nix/**"
                  "data/**"
                ];
              }) config.terranix.terranixConfigurations;
            }
          );
        };
    };
}
