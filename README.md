# Notification Daemon (SOSD)

https://github.com/user-attachments/assets/06e63914-0d2c-4677-a51a-dbe4c08d0e65

**Notification Daemon** is a highly customizable notification system designed for Linux environments, supporting both Wayland and X11. It can function as a standalone notification daemon or as an SOSD (Scriptable On-Screen Display) through external scripts. The system is built to be flexible, allowing users to tailor it to their specific needs.

## Features

- **Wayland**: Works seamlessly on both Wayland and X11 environments.
- **Customizable Notifications**: Highly configurable notifications with support for different urgency levels (Low, Normal, Critical).
- **Scriptable SOSD**: Can be extended with external scripts to function as an on-screen display for various purposes.
- **Rich Configuration**: Extensive configuration options for window positioning, animation duration, colors, and more.
- **Battery Alerts**: Built-in support for battery level alerts with customizable icons and colors.
- **Input Actions**: Supports various input actions like left-click, right-click, scroll, and touch gestures.
- **Dynamic Positioning**: Notifications can be positioned at the top or bottom of the screen.
- **Customizable Urgency Levels**: Each urgency level (Low, Normal, Critical) can have its own display duration, background, and foreground colors.

## Installation

To install the Notification Daemon, clone the repository and build it using Cargo:

```bash
cargo install --git https://github.com/SergioRibera/soft_osd
```

## Configuration

The Notification Daemon is highly configurable through a TOML configuration file.

See the [example configuration file](./sosd.example.toml) for more details.

### Configuration Options

- **globals**: Global settings for animation duration, show duration, and colors.
- **window**: Settings for the notification window, including position, radius, width, and height.
- **battery**: Configuration for battery level alerts, including icons and colors for different levels.
- **urgency**: Settings for different urgency levels, including show duration and colors.
- **actions**: Mapping of input actions to notification actions.

## Usage

### Running the Daemon

To run the Notification Daemon, use the following command:

```bash
sosd -- --config /path/to/config.toml
```

### Sending Notifications

You can send notifications using the `notification` subcommand:

```bash
sosd notification --title "Test Notification" --description "This is a test notification" --urgency Normal
```

### Using as SOSD

The Notification Daemon can be extended with external scripts to function as an SOSD. For example, you can create a script that monitors system metrics and sends notifications accordingly.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request for any bugs or feature requests.

## License

This project is licensed under the [MIT](./LICENSE-MIT) and [APACHE](./LICENSE-APACHE) License. See the files for details.

## Acknowledgments

- Thanks to the Rust community for providing excellent libraries and tools.
- Special thanks to the contributors who helped make this project possible.
