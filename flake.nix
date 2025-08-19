{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    flake-parts.url = "github:hercules-ci/flake-parts";

    flake-compat.url = "https://flakehub.com/f/edolstra/flake-compat/1.tar.gz";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    naersk = {
      url = "github:nix-community/naersk";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        fenix.follows = "fenix";
      };
    };
  };

  outputs =
    inputs@{
      flake-parts,
      naersk,
      nixpkgs,
      ...
    }:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = nixpkgs.lib.systems.flakeExposed;

      imports = [ flake-parts.flakeModules.partitions ];

      partitionedAttrs = {
        checks = "dev";
        devShells = "dev";
        formatter = "dev";
      };

      partitions.dev = {
        extraInputsFlake = ./nix/dev;
        module.imports = [ ./nix/dev ];
      };

      perSystem =
        {
          inputs',
          lib,
          pkgs,
          ...
        }:
        let
          toolchain = inputs'.fenix.packages.fromToolchainFile {
            file = ./rust-toolchain.toml;
            sha256 = "sha256-+9FmLhAOezBZCOziO0Qct1NOrfpjNsXxc/8I0c7BdKE=";
          };

          naersk' = pkgs.callPackage naersk {
            cargo = toolchain;
            rustc = toolchain;
          };

          manifest = (lib.importTOML ./pumpkin/Cargo.toml).package;
          workspace-manifest = (lib.importTOML ./Cargo.toml).workspace.package;
        in
        {
          packages.default = naersk'.buildPackage {
            pname = manifest.name;
            inherit (workspace-manifest) version;
            src = lib.fileset.toSource {
              root = ./.;
              fileset = lib.fileset.unions [
                ./Cargo.lock
                ./Cargo.toml

                ./assets
                ./pumpkin
                ./pumpkin-api-macros
                ./pumpkin-config
                ./pumpkin-data
                ./pumpkin-inventory
                ./pumpkin-inventory
                ./pumpkin-macros
                ./pumpkin-nbt
                ./pumpkin-protocol
                ./pumpkin-registry
                ./pumpkin-util
                ./pumpkin-world
              ];
            };
          };
        };
    };
}
