{
  config,
  lib,
  pkgs,
  ...
}:
with lib; let
  app = import ./. {
    inherit pkgs lib;
    system = pkgs.system;
  };
  cfg = config.programs.sosd;
in {
  options.programs.sosd = {
    enable = mkEnableOption "Enable Soft OSD";
    package = mkOption {
      type = types.package;
      default = app.packages.default;
      description = "Package for Soft OSD";
    };
  };

  config = mkIf cfg.enable {
    environment.systemPackages = [cfg.package];
    boot.initrd.systemd.dbus.enable = true;
  };
}
