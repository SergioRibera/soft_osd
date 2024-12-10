{
  crane,
  fenix,
}: {
  config,
  lib,
  pkgs,
  ...
}:
with lib; let
  sosd = import ./. {
    inherit crane fenix pkgs lib;
    system = pkgs.system;
  };
  cfg = config.programs.sosd;
in {
  options.programs.sosd = {
    enable = mkEnableOption "Enable Soft OSD";
  };

  config = mkIf cfg.enable {
    environment.systemPackages = [sosd.packages.default];
    boot.initrd.systemd.dbus.enable = true;
  };
}
