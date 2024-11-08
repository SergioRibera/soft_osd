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
      sha256 = "sha256-yMuSb5eQPO/bHv+Bcf/US8LVMbf/G/0MSfiPwBhiPpk=";
    };
    # crane: cargo and artifacts manager
    craneLib = crane.overrideToolchain toolchain;

    # buildInputs for Simplemoji
    buildInputs = with pkgs; [
      fontconfig.dev
      libxkbcommon.dev
      wayland
    ];
    src = lib.cleanSourceWith {
      src = craneLib.path ./..;
      filter = path: type:
        (lib.hasInfix "/resources" path)
        || (craneLib.filterCargoSources path type);
    };

    dbusPkg = stdenv.mkDerivation {
      inherit src;
      name = "dbus-needed-files";
      outputs = [ "out" ];
      installPhase = ''
        mkdir -p $out/share/dbus-1/system.d

        cp $src/resources/rs.sergioribera.sosd.conf \
          $out/share/dbus-1/system.d/rs.sergioribera.sosd.conf
      '';
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

    # Build packages and `nix run` apps
    appPkg = rec {
      pkg = pkgs.buildEnv {
        name = "sosd";
        pathsToLink = [ "/share" "/bin" ];
        paths = [
          dbusPkg
          (craneLib.buildPackage (commonArgs // {
              cargoArtifacts = appDeps;
          }))
        ];
      };
      app = {
        type = "app";
        program = "${pkg}${pkg.passthru.exePath or "/bin/${pkg.pname or pkg.name}"}";
      };
    };
  in {
    # `nix run`
    apps.default = appPkg.app;
    # `nix build`
    packages.default = appPkg.pkg;
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
