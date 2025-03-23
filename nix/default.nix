let
  inherit (builtins) currentSystem;
in
  {
    stdenv ? pkgs.stdenv,
    system ? currentSystem,
    lib ? pkgs.lib,
    pkgs,
    crane,
    fenix,
    ...
  }: let
    # fenix: rustup replacement for reproducible builds
    toolchain = fenix.${system}.fromToolchainFile {
      file = ./../rust-toolchain.toml;
      sha256 = "sha256-Hn2uaQzRLidAWpfmRwSRdImifGUCAb9HeAqTYFXWeQk=";
    };
    # crane: cargo and artifacts manager
    craneLib = crane.overrideToolchain toolchain;

    # buildInputs
    buildInputs = with pkgs; [
      fontconfig.dev
      libxkbcommon.dev
      xorg.libX11
      xorg.libXcursor
      xorg.libXi
      xorg.libXrandr
      wayland
    ];
    src = lib.cleanSourceWith {
      src = craneLib.path ./..;
      filter = path: type: (craneLib.filterCargoSources path type);
    };

    # Base args, need for build all crate artifacts and caching this for late builds
    commonArgs = {
      doCheck = false;
      nativeBuildInputs =
        [pkgs.pkg-config]
        ++ lib.optionals stdenv.buildPlatform.isDarwin [
          pkgs.libiconv
        ];
      runtimeDependencies = with pkgs;
        lib.optionals stdenv.isLinux [
          wayland
          libxkbcommon
        ];

      inherit src buildInputs;
    };

    # app artifacts
    appDeps = craneLib.buildDepsOnly commonArgs;
    appBase = (craneLib.buildPackage (commonArgs // {
      cargoArtifacts = appDeps;
    }));

  in {
    # `nix run`
    apps.default = appBase;
    # `nix build`
    packages.default = appBase;
    # `nix develop`
    devShells.default = craneLib.devShell {
      packages = with pkgs; [
          toolchain
          pkg-config
          cargo-dist
          cargo-release
        ] ++ buildInputs;
      LD_LIBRARY_PATH = "${lib.makeLibraryPath buildInputs}";
      PKG_CONFIG_PATH = "${pkgs.fontconfig.dev}/lib/pkgconfig";
    };
  }
