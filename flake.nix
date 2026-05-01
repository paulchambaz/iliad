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
        clippy
        rustc
        cargo-edit
        cargo-outdated
        cargo-tarpaulin
        sqlx-cli
      ];
      iliad = pkgs.rustPlatform.buildRustPackage {
        pname = "iliad";
        version = "1.0.0";
        src = ./.;
        cargoLock.lockFile = ./Cargo.lock;
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
          name = "iliad";
          tag = "latest";
          contents = [
            pkgs.dockerTools.fakeNss
            pkgs.cacert
            iliad
          ];
          config = {
            Entrypoint = ["${iliad}/bin/iliad"];
            Env = [
              "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
            ];
          };
        };
      };
      devShell = pkgs.mkShell {
        nativeBuildInputs = buildPkgs;
        buildInputs = libPkgs ++ devPkgs;
      };
    });
}
