{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = nixpkgs.legacyPackages.${system};

      buildPkgs = with pkgs; [
        pkg-config
        scdoc
      ];

      libPkgs = with pkgs; [
        openssl
      ];

      devPkgs = with pkgs; [
        just
        cargo-tarpaulin
        vhs
      ];
    in {
      packages.default = pkgs.rustPlatform.buildRustPackage {
        pname = "iliad";
        version = "1.0.0";
        src = ./.;
        cargoHash = "sha256-IbnMBmAkfz89tAdOAqU92fkvu+uVbK2SS4jrlignNqY=";

        nativeBuildInputs = buildPkgs;
        buildInputs = libPkgs;

        postInstall = ''
          mkdir -p $out/share/man/man1
          scdoc < iliad.1.scd > $out/share/man/man1/iliad.1
        '';
      };

      devShell = pkgs.mkShell {
        nativeBuildInputs = buildPkgs;
        buildInputs = libPkgs ++ devPkgs;
      };
    });
}
