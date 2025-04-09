{
  stdenv ? pkgs.stdenv,
  lib ? pkgs.lib,
  pkgs ? import <nixpkgs> { },
  ...
}: let
  toolchain = (pkgs.rust-bin.fromRustupToolchainFile ./../rust-toolchain.toml);
  cargoManifest = lib.importTOML ./../Cargo.toml;

  # buildInputs
  buildInputs = with pkgs; [
    fontconfig.dev
    libxkbcommon.dev
    xorg.libX11
    xorg.libXcursor
    xorg.libXi
    xorg.libXrandr
    wayland
    libgcc
  ];

  appBase = pkgs.rustPlatform.buildRustPackage {
    pname = cargoManifest.package.name;
    version = cargoManifest.package.version;
    src = pkgs.lib.cleanSource ./..;
    cargoLock = {
      lockFile = ./../Cargo.lock;
      outputHashes = {
        "dpi-0.1.1" = "sha256-SJvOy8Tqyt9BmH7yQ9G12B7ZsqKU4G6Tp7/3SYnXmKI=";
      };
    };
    doCheck = false;
    nativeBuildInputs = with pkgs;
      lib.optionals stdenv.isLinux [
        pkg-config
        autoPatchelfHook
      ]
      ++ lib.optionals stdenv.buildPlatform.isDarwin [
        libiconv
      ];
    runtimeDependencies = with pkgs;
      lib.optionals stdenv.isLinux [
        wayland
        libxkbcommon
      ];

    postFixup = lib.optionalString stdenv.isLinux ''
      patchelf --set-rpath "${lib.makeLibraryPath buildInputs}" $out/bin/${cargoManifest.package.name}
    '';

    dontWrapQtApps = true;
    makeWrapperArgs = [
      "--prefix LD_LIBRARY_PATH : ${lib.makeLibraryPath buildInputs}"
      "--prefix PATH : ${lib.makeBinPath [ pkgs.wayland ]}"
    ];
    inherit buildInputs;
  };

in {
  # `nix run`
  apps.default = appBase;
  # `nix build`
  packages.default = appBase;
  # `nix develop`
  devShells.default = pkgs.mkShell {
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
