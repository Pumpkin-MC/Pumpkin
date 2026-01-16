{
  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";

    flake-parts.url = "github:hercules-ci/flake-parts";

    flake-compat.url = "https://flakehub.com/f/edolstra/flake-compat/1.tar.gz";

    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs@{
      flake-parts,
      naersk,
      nixpkgs,
      ...
    }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = nixpkgs.lib.systems.flakeExposed;

      perSystem =
        {
          lib,
          pkgs,
          ...
        }:
        let
          naersk' = pkgs.callPackage naersk { };

          manifest = (lib.importTOML ./pumpkin/Cargo.toml).package;
          workspace-manifest = (lib.importTOML ./Cargo.toml).workspace.package;
        in
        {
          packages.default = naersk'.buildPackage {
            pname = manifest.name;
            inherit (workspace-manifest) version;

            nativeBuildInputs = [ pkgs.rustfmt ];

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
                ./pumpkin-macros
                ./pumpkin-nbt
                ./pumpkin-protocol
                ./pumpkin-util
                ./pumpkin-world
              ];
            };
          };

          devShells.default = pkgs.mkShell {
            packages = with pkgs; [
              cargo
              clippy
              rust-analyzer
              rustc
              rustfmt
            ];
          };

          formatter = pkgs.nixfmt-tree;
        };
    };
}
