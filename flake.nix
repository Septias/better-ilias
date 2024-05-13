{
  description = "Application to set wallpapers from reddit as desktop-background";
  inputs = {
    os_flake.url = "github:septias/nixos-config";
    nixpkgs.follows = "os_flake/nixpkgs";
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
          ];
          rust-toolchain = pkgs.rust-bin.stable.latest.default.override {
            extensions = ["rust-src" "rustfmt" "rust-docs" "clippy" "rust-analyzer"];
          };
          rustPlatform = pkgs.makeRustPlatform {
            cargo = rust-toolchain;
            rustc = rust-toolchain;
          };
          name = "reddit-wallpapers";
          dist = ./dist;
          /* frontend = pkgs_unstable.stdenv.mkDerivation (finalAttrs: {
            inherit version;
            pname = name;
            src = ./.;
            pnpmDeps = pkgs_unstable.fetchPnpmDeps {
              inherit finalAttrs src pname;
              hash = pkgs.fakeHash;
            };
            #nativeBuildInputs = [pkgs.pnpmConfigHook];
          }); */
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
              inherit buildInputs name desktopItem;
              nativeBuildInputs = buildInputs;
              src = ./src-tauri;
              cargoLock = {
                lockFile = ./src-tauri/Cargo.lock;
              };

              postPatch = ''
                substituteInPlace tauri.conf.json --replace '"distDir": "../dist",' '"distDir": "${dist}",'
              '';
      
              postInstall = ''
                mkdir -p $out/share/icons/hicolor/128x128/apps
                cp ${icon} $out/share/icons/hicolor/128x128/apps/better-ilias.png
                mkdir -p "$out/share/applications"
                cp $desktopItem/share/applications/* $out/share/applications
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
            '';
          };
        }
      );
}
