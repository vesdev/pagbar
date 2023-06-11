{
  description = "pagbar";

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

        craneLib = crane.lib.${system};

      in with pkgs;{

        packages.default = craneLib.buildPackage rec {
          src = craneLib.cleanCargoSource (craneLib.path ./.);
          doCheck = true;
          buildInputs = [
            libGL
            libxkbcommon
            wayland
            xorg.libX11
            xorg.libXcursor
            xorg.libXi
            xorg.libXrandr
          ];

          nativeBuildInputs = [
            pkg-config
            pkgs.makeWrapper 
          ];

          postInstall = ''
            wrapProgram "$out/bin/pagbar" --prefix LD_LIBRARY_PATH : "${lib.makeLibraryPath buildInputs}"
          '';
          LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;
        };

        devShells.default = mkShell rec {
          nativeBuildInputs = [
            pkg-config
          ];
          buildInputs = [
            (rust-bin.stable.latest.default.override { extensions = [ "rust-src" "rust-analyzer" ]; })
            bashInteractive
            rust-bin.stable.latest.default
            rust-analyzer
            

            libxkbcommon
            libGL

            # WINIT_UNIX_BACKEND=wayland
            # wayland

            # WINIT_UNIX_BACKEND=x11
            xorg.libXcursor
            xorg.libXrandr
            xorg.libXi
            xorg.libX11
          ];

          LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;
        };

        overlay = final: prev: {
          pagbar = self.packages.${final.system}.pagbar;
        };
      });
}