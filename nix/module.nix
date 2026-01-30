{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.services.scx_horoscope;
in
{
  options.services.scx_horoscope = {
    enable = lib.mkEnableOption "Astrological CPU scheduler";

    extraArgs = lib.mkOption {
      type = lib.types.listOf lib.types.singleLineStr;
      default = [ ];
      example = [
        "--slice-us 5000"
        "--cosmic-weather"
        "--verbose"
      ];
      description = ''
        Additional runtime scheduler parameters.

        Run `scx_horoscope --help` to see the available options.
      '';
    };
  };

  config = lib.mkIf cfg.enable {
    systemd.services.scx_horoscope = {
      description = "Astrological CPU scheduler";
      wantedBy = [ "multi-user.target" ];

      unitConfig.ConditionPathIsDirectory = "/sys/kernel/sched_ext";

      startLimitIntervalSec = 30;
      startLimitBurst = 2;

      serviceConfig = {
        Type = "simple";
        ExecStart = "${lib.getExe pkgs.scx_horoscope} ${lib.escapeShellArgs cfg.extraArgs}";
        Restart = "on-failure";
      };
    };

    assertions = [
      {
        assertion = config.boot.kernelPackages.kernel.versionAtLeast "6.12";
        message = "sched_ext schedulers require kernels 6.12 or later";
      }
    ];
  };
}

