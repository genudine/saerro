{
  description = "PlanetSide 2 Metagame Harvesting Service";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  nixConfig = {
    extra-substituters = [
      "https://nix-community.cachix.org"
    ];

    extra-trusted-public-keys = [
      "nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs="
    ];
  };

  outputs = inputs: inputs.flake-parts.lib.mkFlake { inherit inputs; } {
    systems = [ "x86_64-linux" "aarch64-linux" ];
    perSystem = { config, self', pkgs, lib, system, ... }: let
      fenix = inputs.fenix.packages.${system}.minimal;
      cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
      
      buildDeps = [
        pkgs.openssl
      ];

      devDeps = [
        fenix.toolchain
        pkgs.docker-compose
        pkgs.cargo-watch
      ];
      PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
    in {
      packages.default = (pkgs.makeRustPlatform {
        cargo = fenix.toolchain;
        rustc = fenix.toolchain;
      }).buildRustPackage {
        inherit (cargoToml.package) name version;
        cargoLock.lockFile = ./Cargo.lock;
        src = ./.;
        nativeBuildInputs = [ pkgs.pkg-config ];
        buildInputs = buildDeps ++ devDeps;
      };

      devShells.default = pkgs.mkShell {
        nativeBuildInputs = buildDeps ++ devDeps;
      };
    };
  };
}