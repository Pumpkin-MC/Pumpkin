{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";

    flake-parts.url = "github:hercules-ci/flake-parts";

    flake-compat = {
      url = "github:NixOS/flake-compat";
      flake = false;
    };

    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    self.submodules = true;
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

          manifest = (lib.importTOML ./crates/pumpkin/Cargo.toml).package;
          workspace-manifest = (lib.importTOML ./Cargo.toml).workspace.package;
        in
        {
          packages.default = naersk'.buildPackage {
            pname = manifest.name;
            inherit (workspace-manifest) version;

            nativeBuildInputs = [ pkgs.rustfmt ];

            src = lib.fileset.toSource {
              root = ./.;
              fileset = lib.fileset.gitTracked ./.;
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
