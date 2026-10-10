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

          # The vanilla datapack is not in the repository. The pumpkin-data build script
          # downloads the server jar for it, which the build sandbox forbids, so fetch the
          # jar here and hand it to the script. Keep the url and hash in step with
          # MC_VERSION in crates/pumpkin-data/build.rs.
          minecraft-server-jar = pkgs.fetchurl {
            url = "https://piston-data.mojang.com/v1/objects/33680f5f2ac32864d6d7cf5e56a705fdb3e05f4c/server.jar";
            hash = "sha256-0FLxTXoXNzT7pVNxHltXAWLi8qMTJn7jGiG5daZ5vmQ=";
          };
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
            PUMPKIN_MINECRAFT_SERVER_JAR = minecraft-server-jar;

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
