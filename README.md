# thermal-printer-tg-bot

A project to control a generic thermal printer module via Telegram. Made in Rust.

## Supported Printers

-   EM5820
-   ...and probably other printers that support ESC/POS commands

## Installation

-   Compile the project
-   Create an `.env` file containing the fields as described in `.env.example`
-   Create a new Telegram bot via @BotFather
-   Paste in the token generated by the bot into the `.env` file
-   Retrieve your user ID, put it in the `.env` file
-   Connect your printer via Serial
-   Enter your serial port into the `.env` file (eg. `/dev/ttyUSB0`)
-   Start the executable
-   ...
-   Profit! (Start sending text, commands, images and stickers to the bot and see it printed out)

## Development (Flakes)

This repo uses [Flakes](https://nixos.asia/en/flakes) from the get-go.

```bash
# Dev shell
nix develop

# or run via cargo
nix develop -c cargo run

# build
nix build
```

We also provide a [`justfile`](https://just.systems/) for Makefile'esque commands to be run inside of the devShell.

## Cross Compilation (Raspberry Pi Zero 1/1W)

> Docker is not required, but it's used here to make cross compilation a bit easier.

-   Download the toolchain for the Pi Zero: [Archive Direct](https://master.dl.sourceforge.net/project/raspberry-pi-cross-compilers/Raspberry%20Pi%20GCC%20Cross-Compiler%20Toolchains/Buster/GCC%2014.2.0/Raspberry%20Pi%201%2C%20Zero/cross-gcc-14.2.0-pi_0-1.tar.gz?viasf=1) or [Archive Mirror (in case it takes ages to download)](https://drive.google.com/file/d/1EY8ZjtlQ2vxqN5SZpU2V3Hb6EanlFHmT/view) ([Source](https://sourceforge.net/projects/raspberry-pi-cross-compilers/))
-   Untar it into `./tools`
-   Run `just crosscompile` (or `docker build --output=output .` if `just` isn't installed)
-   Copy the content of `./output` to your pi
-   Make sure the `.env` on the target is also filled in
-   Run the program as usual
