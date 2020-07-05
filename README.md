# mahjongg

[![build](https://github.com/mosmeh/mahjongg/workflows/build/badge.svg)](https://github.com/mosmeh/mahjongg/actions)

GNOME Mahjongg-compatible Mahjong solitaire (aka Shanghai)

## Installation

Clone this repository and run:

```sh
cargo install --path .
```

## Usage

On Ubuntu, install `gnome-mahjongg` and `mahjongg` will automatically use `gnome-mahjongg`'s assets located in `/usr/share/gnome-mahjongg/`.

```sh
sudo apt install -y gnome-mahjongg
mahjongg
```

On Windows and other systems where `gnome-mahjongg` is not available, download `gnome-mahjongg`'s assets from [here](https://gitlab.gnome.org/GNOME/gnome-mahjongg/-/tree/master/data) and provide them with command-line options:

```sh
mahjongg -t postmodern.svg -m mahjongg.map
```

## Command-line options

```
-w, --width <width>              Width of window in pixels [default: 900]
-h, --height <height>            Height of window in pixels [default: 600]
-t, --theme <theme>              Theme file (GNOME Mahjongg format) [default: /usr/share/gnome-
                                    mahjongg/themes/postmodern.svg]
-m, --map <map>                  Map file (GNOME Mahjongg format) [default: /usr/share/gnome-
                                    mahjongg/maps/mahjongg.map]
-b, --background <background>    Background color [default: #34385b]
```
