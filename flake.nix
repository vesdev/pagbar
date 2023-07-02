{
  description = "status bar";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
    crane,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default;
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        commonArgs = with pkgs; {
          buildInputs = [          
            libxkbcommon
            libGL
            vulkan-loader

            # WINIT_UNIX_BACKEND=wayland
            # wayland

            # WINIT_UNIX_BACKEND=x11
            xorg.libXcursor
            xorg.libXrandr
            xorg.libXi
            xorg.libX11
          ];

          nativeBuildInputs = [
            pkg-config
          ];
        };
                  
        pagbar = craneLib.buildPackage rec {
          src = craneLib.cleanCargoSource (craneLib.path ./.);

          buildInputs = commonArgs.buildInputs; 
          nativeBuildInputs = [
            pkgs.makeWrapper
          ] 
          ++ commonArgs.nativeBuildInputs;

          postInstall = ''
            wrapProgram "$out/bin/pagbar" --prefix LD_LIBRARY_PATH : "${pkgs.lib.makeLibraryPath buildInputs}"
          '';
        };

      in with pkgs; {        
        packages.default = pagbar;

        apps.default = flake-utils.lib.mkApp {
          drv = pagbar;
        };

        devShells.default = mkShell rec {
          buildInputs = [
            (rustToolchain.override { extensions = [ "rust-src" "rust-analyzer" ]; })
            bashInteractive
            rust-analyzer
          ]
          ++ commonArgs.buildInputs;

          nativeBuildInputs = commonArgs.nativeBuildInputs;          

          LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;
        };
      }
    );
}