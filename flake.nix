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
            pkgs.rust-analyzer
          ];
          buildInputs = [
            pkgs.xorg.libxcb
          ];
          VK_ADD_LAYER_PATH = "${pkgs.vulkan-validation-layers}/share/vulkan/explicit_layer.d";
          RADV_PERFTEST = "video_encode,video_decode";
          LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath [ pkgs.vulkan-loader ]}";
        };
      }
    );
}
