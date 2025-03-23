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
  } @ inputs: let
      fenix = inputs.fenix.packages;
    in
    (flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = nixpkgs.legacyPackages.${system};
        crane = inputs.crane.mkLib pkgs;
        bundle = import ./nix {
          inherit pkgs system crane fenix;
        };
      in {
        inherit (bundle) apps packages devShells;
        # nixosModules
        nixosModules = {
          default = import ./nix/nixos-module.nix {
            inherit crane fenix;
          };
          home-manager = import ./nix/hm-module.nix {
            inherit crane fenix;
          };
        };
      }
    ));
}
