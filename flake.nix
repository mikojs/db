{
  description = "Parse database information from environment variables.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
    flake-parts.url = "github:hercules-ci/flake-parts";
  };

  outputs =
    inputs@{
      self,
      nixpkgs,
      flake-parts,
      ...
    }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      perSystem =
        { pkgs, self', ... }:
        {
          packages = {
            miko-db = pkgs.rustPlatform.buildRustPackage {
              pname = "db";
              version = "0.1.0";
              src = ./.;
              cargoLock.lockFile = ./Cargo.lock;
            };

            default = self'.packages.miko-db;
          };
        };

      flake.overlays.default = final: prev: {
        miko-db = self.packages.${final.system}.miko-db;
      };
    };
}
