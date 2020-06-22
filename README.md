![CI](https://github.com/Cokemonkey11/thw-subscriber/workflows/CI/badge.svg?branch=master)

[![asciicast](https://asciinema.org/a/tVQMk2d11rrVie1KilyFKv4He.svg)](https://asciinema.org/a/tVQMk2d11rrVie1KilyFKv4He)

# Installing

## Ubuntu

```bash
sudo apt-get install xorg-dev libxcb-composite0-dev
cargo run
```

## Windows

```bash
# with Visual Studio installed
cargo run
```

## Windows subsystem for linux

_Optional steps below are for enabling clipboard support_

1. `$ sudo apt-get install xorg-dev libxcb-composite0-dev`
1. Optional: Install [vcxsrv](https://sourceforge.net/projects/vcxsrv/)
1. Optional: run vcxsrv with the `-ac` flag
1. Optional: `$ echo "export DISPLAY=$(cat /etc/resolv.conf | tail -n 1| cut -d' ' -f2):0.0" >> ~/.bashrc`
1. Optional: `$ . ~/.bashrc`
1. `$ cargo run`
