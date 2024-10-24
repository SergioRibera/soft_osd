{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    crane.url = "github:ipetkov/crane";
    fenix.url = "github:nix-community/fenix";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    nixpkgs,
    flake-utils,
    ...
  } @ inputs:
  # Iterate over Arm, x86 for MacOs 🍎 and Linux 🐧
    flake-utils.lib.eachSystem (flake-utils.lib.defaultSystems) (
      system: let
        pkgs = nixpkgs.legacyPackages.${system};
        crane = inputs.crane.mkLib pkgs;
        fenix = inputs.fenix.packages;
        appBundle = import ./nix {
          inherit system pkgs crane fenix;
        };
      in {
        inherit (appBundle) apps packages devShells;
        # Overlays
        overlays.default = import ./nix/overlay.nix {
          inherit pkgs crane fenix;
        };
        # nixosModules
        nixosModules = {
          default = import ./nix/nixos-module.nix {
            inherit pkgs crane fenix;
          };
          home-manager = import ./nix/hm-module.nix {
            inherit pkgs crane fenix;
          };
        };
      }
    );
}
