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
      pkgs = import nixpkgs {
        inherit system;
      };

      buildPkgs = with pkgs; [
        pkg-config
        scdoc
      ];

      libPkgs = with pkgs; [
        openssl
      ];

      devPkgs = with pkgs; [
        vhs
        just
        cargo
        cargo-edit
        cargo-tarpaulin
      ];

      iliad = pkgs.rustPlatform.buildRustPackage {
        pname = "iliad";
        version = "1.0.0";
        src = ./.;
        cargoHash = "sha256-b68FADqSuVYrtxf8kCaVHw7pX1F65pv4R6dMyXx52Y0=";

        nativeBuildInputs = buildPkgs;
        buildInputs = libPkgs;

        postInstall = ''
          mkdir -p $out/share/man/man1
          scdoc < iliad.1.scd | sed "s/1980-01-01/$(date '+%B %Y')/" > $out/share/man/man1/iliad.1
        '';
      };
    in {
      packages = {
        default = iliad;
        iliad = iliad;

        docker = pkgs.dockerTools.buildLayeredImage {
          name = "paulchambaz/iliad";
          tag = "latest";
          contents = [iliad];

          config = {
            Env = ["PATH=/bin"];
            WorkingDir = "/app";
            Entrypoint = ["${iliad}/bin/iliad"];
          };
        };
      };

      devShell = pkgs.mkShell {
        nativeBuildInputs = buildPkgs;
        buildInputs = libPkgs ++ devPkgs;
      };
    });
}
