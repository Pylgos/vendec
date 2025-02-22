{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
  };

  outputs =
    {
      self,
      flake-utils,
      nixpkgs,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        mkShell = pkgs.mkShell.override { stdenv = pkgs.clangStdenv; };
      in
      {
        devShell = mkShell {
          nativeBuildInputs = [
            pkgs.cargo
            pkgs.rustfmt
            pkgs.clippy
            pkgs.rust-analyzer
            pkgs.rustc
            pkgs.rust-bindgen
            pkgs.pkg-config
          ];
          buildInputs = [
            pkgs.libva
          ];
          LIBVA_PATH = "${pkgs.lib.getDev pkgs.libva}";
          LIBCLANG_PATH = "${pkgs.lib.getLib pkgs.libclang}/lib";
          LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath [ pkgs.libva ]}";
        };
      }
    );
}
