{
  description = "Application to set wallpapers from reddit as desktop-background";
  inputs = {
    os_flake.url = "github:septias/nixos-config";
    nixpkgs.follows = "os_flake/nixpkgs";
    nixpkgs-unstable.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.follows = "rust-overlay/flake-utils";
    naersk.url = "github:nix-community/naersk";
  };
  outputs = inputs:
    with inputs;
      flake-utils.lib.eachDefaultSystem (
        system: let
          pkgs = import nixpkgs {
            overlays = [(import rust-overlay)];
            inherit system;
          };
          unstable = import nixpkgs-unstable {
            inherit system;
          };
          libraries = with pkgs; [
            webkitgtk
            gtk3
            cairo
            gdk-pixbuf
            glib
            dbus
            openssl_3
            librsvg
          ];

          buildInputs = with pkgs; [
            curl
            wget
            pkg-config
            dbus
            openssl_3
            openssl
            glib
            gtk3
            libsoup
            webkitgtk
            librsvg
            makeWrapper
          ];
          rust-toolchain = pkgs.rust-bin.stable.latest.default.override {
            extensions = ["rust-src" "rustfmt" "rust-docs" "clippy" "rust-analyzer"];
          };
          rustPlatform = pkgs.makeRustPlatform {
            cargo = rust-toolchain;
            rustc = rust-toolchain;
          };
          name = "better-ilias";
          version = "1.0.1";
          frontend = pkgs.stdenv.mkDerivation (finalAttrs: {
            inherit version;
            pname = "better-ilias-frontend";
            src = pkgs.lib.cleanSource ./frontend;
            nativeBuildInputs = with unstable; [
              nodejs
              unstable.pnpm.configHook
            ];
            pnpmDeps = unstable.pnpm.fetchDeps {
              inherit (finalAttrs) pname version src;
              hash = "sha256-fQ+6cYSNHX8U/hdWBNK2bKz8UvurHZrgrGeYSnNWb4k=";
            };

            installPhase = ''
              pnpm build
              cp -r dist $out
            '';
          });
          desktopItem = pkgs.makeDesktopItem {
            name = "Better Ilias";
            desktopName = "Better Ilias";
            icon = "better-ilias";
            exec = "better-ilias";
            categories = [ "Office" ];
          };
          icon = ./src-tauri/icons/icon.png;
        
        in rec {
          formatter = pkgs.alejandra;
          packages = {
            ${name} = rustPlatform.buildRustPackage rec {
              inherit buildInputs name desktopItem version;
              nativeBuildInputs = buildInputs;
              src = ./src-tauri;
              cargoLock = {
                lockFile = ./src-tauri/Cargo.lock;
              };

              postPatch = ''
                substituteInPlace tauri.conf.json --replace-fail '"distDir": "../frontend/dist",' '"distDir": "${frontend}",'
              '';
      
              postInstall = ''
                mkdir -p $out/share/icons/hicolor/128x128/apps
                cp ${icon} $out/share/icons/hicolor/128x128/apps/better-ilias.png
                mkdir -p "$out/share/applications"
                cp $desktopItem/share/applications/* $out/share/applications

                wrapProgram $out/bin/${name} --prefix PATH : ${pkgs.glib}/bin --set WEBKIT_DISABLE_COMPOSITING_MODE 1
              '';

              meta = {  
                description = "Sync Ilias to your local system";
                homepage = "https://github.com/Septias/reddit-wallpapers";
                mainProgram = "reddit-wallpapers";
              };
            };
            default = packages.${name};
          };
          devShells.default = pkgs.mkShell {
            buildInputs = buildInputs ++ [rust-toolchain pkgs.cargo-tauri];
            RUST_BACKTRACE = 1;

            shellHook = ''
              export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath libraries}:$LD_LIBRARY_PATH
              export XDG_DATA_DIRS=${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}:${pkgs.gtk3}/share/gsettings-schemas/${pkgs.gtk3.name}:$XDG_DATA_DIRS
              export WEBKIT_DISABLE_COMPOSITING_MODE=1 
            '';
          };
        }
      );
}
