{
  description = "pagbar";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    naersk.url = "github:nix-community/naersk";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
    naersk,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system:
    let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        naersk-lib = pkgs.callPackage naersk { };

        libPath = with pkgs; lib.makeLibraryPath [
          libGL
          libxkbcommon
          wayland
          xorg.libX11
          xorg.libXcursor
          xorg.libXi
          xorg.libXrandr
        ];
        # cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        pagbar = with pkgs; naersk-lib.buildPackage{
          src = ./.;
          doCheck = true;
          pname = "pagbar";

          nativeBuildInputs = [ 
            pkg-config 
            pkgs.makeWrapper 
          ];
          
          buildInputs = with pkgs; [
            xorg.libxcb
          ];

          postInstall = ''
            wrapProgram "$out/bin/pagbar" --prefix LD_LIBRARY_PATH : "${libPath}"
          '';
          
        };

      LD_LIBRARY_PATH = libPath;

      in {
 
        packages.default = pagbar;

        devShells.default = with pkgs; mkShell rec {
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

          LD_LIBRARY_PATH = lib.makeLibraryPath commonArgs.buildInputs;
        };
      });
}