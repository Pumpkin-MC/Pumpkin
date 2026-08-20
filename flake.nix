{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";

    flake-parts.url = "github:hercules-ci/flake-parts";

    flake-compat = {
      url = "github:NixOS/flake-compat";
      flake = false;
    };

    self.submodules = true;
  };

  outputs =
    inputs@{
      flake-parts,
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
          manifest = (lib.importTOML ./crates/pumpkin/Cargo.toml).package;
          workspace-manifest = (lib.importTOML ./Cargo.toml).workspace.package;
        in
        {
          packages.default = pkgs.rustPlatform.buildRustPackage {
            pname = manifest.name;
            inherit (workspace-manifest) version;

            src = lib.cleanSource ./.;

            cargoLock = {
              lockFile = ./Cargo.lock;
            };

            nativeBuildInputs = [
              pkgs.rustfmt
              pkgs.pkg-config
            ];

            cargoBuildFlags = [
              "--package"
              "pumpkin"
            ];

            CARGO_PROFILE_RELEASE_LTO = "thin";
            CARGO_PROFILE_RELEASE_CODEGEN_UNITS = "16";

            doCheck = false;
          };

          devShells.default = pkgs.mkShell {
            packages = with pkgs; [
              cargo
              clippy
              rust-analyzer
              rustc
              rustfmt
              pkg-config
            ];
          };

          formatter = pkgs.nixfmt-tree;
        };
    };
}
