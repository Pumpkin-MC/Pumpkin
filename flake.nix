{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    flake-compat.url = "https://flakehub.com/f/edolstra/flake-compat/1.tar.gz";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs @ {
    nixpkgs,
    flake-parts,
    fenix,
    ...
  }:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = nixpkgs.lib.systems.flakeExposed;
      perSystem = {
        pkgs,
        system,
        ...
      }: let
        toolchain = fenix.packages.${system}.stable.toolchain;
        manifest = (pkgs.lib.importTOML ./pumpkin/Cargo.toml).package;
        workspace-manifest =
          (pkgs.lib.importTOML ./Cargo.toml).workspace.package;
      in {
        formatter = pkgs.nixfmt-rfc-style;
        _module.args.pkgs = import nixpkgs {
          inherit system;
        };

        devShells.default =
          pkgs.mkShell
          {
            nativeBuildInputs = with pkgs; [
              toolchain
              pkg-config
            ];
          };

        packages.default =
          (pkgs.makeRustPlatform {
            rustc = toolchain;
            cargo = toolchain;
          }).buildRustPackage {
            pname = manifest.name;
            version = workspace-manifest.version;

            src = ./.;

            useFetchCargoVendor = true;
            cargoLock = {
              lockFile = ./Cargo.lock;
            };
          };
      };
    };
}
