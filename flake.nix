{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    nixpkgs,
    flake-utils,
    rust-overlay,
    ...
  } @ inputs: 
    (flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        bundle = import ./nix {
          inherit pkgs system;
        };
      in {
        inherit (bundle) apps packages devShells;
    })) // (flake-utils.lib.eachDefaultSystemPassThrough (system:
      {
        overlays.default = final: prev: {
          sosd = inputs.self.packages.${prev.system}.default;
        };

        # nixosModules
        nixosModules = {
          default = ./nix/nixos-module.nix;
          home-manager = ./nix/hm-module.nix;
        };
      }
    ));
}
