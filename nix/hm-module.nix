{
  crane,
  fenix,
}: {
  config,
  lib,
  pkgs,
  ...
} @ inputs:
with lib; let
  app = import ./. {
    inherit crane fenix pkgs lib;
    system = pkgs.system;
  };
  cfg = config.programs.sosd;
  tomlFormat = pkgs.formats.toml {};
  filterCfg = cfg: filterAttrs (n: v: ((builtins.typeOf v) != "null") && n != "enable") cfg;
  cfgScheme = import ./config.nix inputs;
in {
  options.programs.sosd = {
    enable = mkEnableOption "Enable Soft OSD";
    package = mkOption {
      type = types.package;
      default = app.packages.default;
      description = "Package for Soft OSD";
    };
  } // cfgScheme;

  config = mkIf cfg.enable {
    home.packages = [cfg.package];
    home.file.".config/sosd/config.toml".source = tomlFormat.generate "config.toml" (filterCfg cfg);
  };
}
