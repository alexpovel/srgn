{
  description = " A grep-like tool which understands source code syntax and allows for manipulation in addition to search.";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs =
    { self, nixpkgs }:
    let
      crateName = "srgn";

      cargoToml = nixpkgs.lib.importTOML ./Cargo.toml;
      version = cargoToml.package.version;

      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      # This only returns the package derivation, not the whole structure.
      buildPackageFor =
        system:
        let
          pkgs = import nixpkgs { inherit system; };
        in
        pkgs.rustPlatform.buildRustPackage {
          pname = crateName;
          inherit version;

          src = self;
          cargoLock.lockFile = ./Cargo.lock;
        };
    in
    {
      # Support `nix build .#<package>`
      packages = nixpkgs.lib.genAttrs supportedSystems (system: {
        default = buildPackageFor system;
        "${crateName}" = buildPackageFor system;
      });

      # Support `nix run .`
      apps = nixpkgs.lib.genAttrs supportedSystems (
        system:
        let
          pkg = self.packages."${system}".default;
        in
        {
          default = {
            type = "app";
            program = "${pkg}/bin/${crateName}";
          };
          "${crateName}" = {
            type = "app";
            program = "${pkg}/bin/${crateName}";
          };
        }
      );

      overlays.default = final: prev: {
        # This will add `pkgs.srgn` for consumers of the overlay. `prev.system`
        # correctly gets the system of the nixpkgs instance the overlay is being applied
        # to.
        "${crateName}" = self.packages."${prev.system}".default;
      };
    };
}
