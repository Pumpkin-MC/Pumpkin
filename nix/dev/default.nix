{ inputs, ... }:
{
  imports = [ inputs.treefmt-nix.flakeModule ];

  perSystem =
    { self', pkgs, ... }:
    {
      treefmt.programs = {
        # Docker
        dockerfmt.enable = true;

        # Nix
        deadnix.enable = true;
        statix.enable = true;
        nixfmt = {
          enable = true;
          strict = true;
        };

        # Rust
        rustfmt.enable = true;

        # TOML
        taplo.enable = true;
      };

      checks = self'.packages;

      devShells.default = pkgs.mkShell {
        name = "pumpkin";

        inputsFrom = [ self'.packages.default ];

        packages = [ pkgs.nil ];
      };
    };
}
