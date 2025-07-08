{
  description = "Better Ilias handler";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.05";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
  };
  outputs = inputs:
    with inputs;
      flake-utils.lib.eachDefaultSystem (
        system: let
          pname = "better-ilias";
          version = "1.0.1";

          pkgs = import nixpkgs {
            overlays = [(import rust-overlay)];
            inherit system;
          };

          nativeBuildInputs = with pkgs; [
            pkg-config
            gobject-introspection
            cargo-tauri.hook
            pnpm_9.configHook
            nodejs
          ];

          buildInputs = with pkgs; [
            at-spi2-atk
            atkmm
            cairo
            gdk-pixbuf
            glib
            gtk3
            harfbuzz
            librsvg
            libsoup_3
            pango
            webkitgtk_4_1
            openssl
            makeWrapper
          ];

          rust-toolchain = pkgs.rust-bin.stable.latest.default.override {
            extensions = ["rust-src" "rustfmt" "rust-docs" "clippy" "rust-analyzer"];
          };

          rustPlatform = pkgs.makeRustPlatform {
            cargo = rust-toolchain;
            rustc = rust-toolchain;
          };

          desktopItem = pkgs.makeDesktopItem {
            name = "Better Ilias";
            desktopName = "Better Ilias";
            icon = "better-ilias";
            exec = "better-ilias";
            categories = ["Office"];
          };

          src = pkgs.lib.cleanSource ./.;
        in rec {
          formatter = pkgs.alejandra;
          packages = {
            ${pname} = rustPlatform.buildRustPackage (finalAttrs: {
              inherit buildInputs nativeBuildInputs pname desktopItem version src;

              pnpmDeps = pkgs.pnpm_9.fetchDeps {
                inherit (finalAttrs) pname version src;
                hash = "sha256-H4Ux4PjahhYAUGRVzXM5znmSAncXMn5wy96R7jBlHFc=";
              };

              cargoRoot = "src-tauri";
              cargoLock = {
                lockFile = "${src}/src-tauri/Cargo.lock";
              };
              buildAndTestSubdir = finalAttrs.cargoRoot;

              meta = {
                description = "Application to set r/wallpapers from reddit as desktop-background";
                homepage = "https://github.com/Septias/reddit-wallpapers";
                mainProgram = "reddit-wallpapers";
              };
            });
            default = packages.${pname};
          };
          devShells.default = pkgs.mkShell {
            inherit nativeBuildInputs;
            buildInputs = buildInputs ++ [rust-toolchain pkgs.cargo-tauri];
            RUST_BACKTRACE = 1;

            shellHook = ''
              export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath buildInputs}:$LD_LIBRARY_PATH
              export XDG_DATA_DIRS=${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}:${pkgs.gtk3}/share/gsettings-schemas/${pkgs.gtk3.name}:$XDG_DATA_DIRS
              export WEBKIT_DISABLE_COMPOSITING_MODE=1
            '';
          };
        }
      );
}
