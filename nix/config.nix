{ lib, ... }: with lib; {
  globals = mkOption {
    type = types.submodule {
      options = {
        animation_duration = mkOption {
          type = types.nullOr types.float;
          default = 1.0;
          description = "The animation duration to show the widget (in milliseconds)";
        };
        show_duration = mkOption {
          type = types.nullOr types.float;
          default = 5.0;
          description = "The animation duration to show the widget (in seconds)";
        };
        background = mkOption {
          type = types.nullOr types.string;
          default = "#000";
          description = "Background Color of widget, support: '#RRGGBBAA', '#RGBA' and '#RGB'";
        };
        foreground_color = mkOption {
          type = types.nullOr types.string;
          default = "#fff";
          description = "Foreground Color of widget, support: '#RRGGBBAA', '#RGBA' and '#RGB'";
        };
      };
    };
  };
  output = mkOption {
    type = types.nullOr types.string;
    default = nill;
    description = "Output Screen where notification has been showed";
  };
  actions = mkOption {
    type = types.nullOr types.attrs;
    default = {
      LeftClick = {
        action = "OpenNotification";
      };
      RightClick = {
        action = "Close";
      };
      ScrollUp = {
        action = "Close";
      };
    };
  };
  window = mkOption {
    type = types.nullOr types.submodule {
      options = {
        position = mkOption {
          type = types.enum [ "Top" "Left" "Right" "Bottom" ];
          description = "The Position into Screen";
        };
        radius = mkOption {
          type = types.nullOr types.number;
          default = nill;
          description = "The radius of the widget [default: 100]";
        };
        width = mkOption {
          type = types.nullOr types.number;
          default = nill;
          description = "The width of the widget [default: 600]";
        };
        height = mkOption {
          type = types.nullOr types.number;
          default = nill;
          description = "The height of the widget [default: 80]";
        };
      };
    };
    default = {
      height = 80;
      position = "Top";
      radius = 100;
      width = 600;
    };
  };
  battery = mkOption {
    type = types.submodule {
      options = {
        enabled = mkOption {
          type = types.bool;
        };
        refresh_time = mkOption {
          type = types.float;
        };
        level = mkOption {
          type = types.nullOr types.attrs;
          default = nill;
        };
      };
    };
  };
  urgency = mkOption {
    type = types.submodule {
      options = {
        low = mkOption {
          type = types.submodule {
            options = {
              show_duration = mkOption {
                type = types.nullOr types.float;
                default = 5.0;
                description = "The animation duration to show the widget (in seconds)";
              };
              background = mkOption {
                type = types.nullOr types.string;
                default = "#000";
                description = "Background Color of widget, support: '#RRGGBBAA', '#RGBA' and '#RGB'";
              };
              foreground_color = mkOption {
                type = types.nullOr types.string;
                default = "#fff";
                description = "Foreground Color of widget, support: '#RRGGBBAA', '#RGBA' and '#RGB'";
              };
            };
          };
        };
        normal = mkOption {
          type = types.submodule {
            options = {
              show_duration = mkOption {
                type = types.nullOr types.float;
                default = 5.0;
                description = "The animation duration to show the widget (in seconds)";
              };
              background = mkOption {
                type = types.nullOr types.string;
                default = "#000";
                description = "Background Color of widget, support: '#RRGGBBAA', '#RGBA' and '#RGB'";
              };
              foreground_color = mkOption {
                type = types.nullOr types.string;
                default = "#fff";
                description = "Foreground Color of widget, support: '#RRGGBBAA', '#RGBA' and '#RGB'";
              };
            };
          };
        };
        critical = mkOption {
          type = types.submodule {
            options = {
              show_duration = mkOption {
                type = types.nullOr types.float;
                default = 5.0;
                description = "The animation duration to show the widget (in seconds)";
              };
              background = mkOption {
                type = types.nullOr types.string;
                default = "#000";
                description = "Background Color of widget, support: '#RRGGBBAA', '#RGBA' and '#RGB'";
              };
              foreground_color = mkOption {
                type = types.nullOr types.string;
                default = "#fff";
                description = "Foreground Color of widget, support: '#RRGGBBAA', '#RGBA' and '#RGB'";
              };
            };
          };
        };
      };
    };
  };
}
