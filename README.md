# mahjongg

[![build](https://github.com/mosmeh/mahjongg/workflows/build/badge.svg)](https://github.com/mosmeh/mahjongg/actions)

Mahjong solitaire game (aka Shanghai) compatible with GNOME Mahjongg and KMahjongg

![](https://user-images.githubusercontent.com/1721932/88615246-7e5ab100-d0cc-11ea-885e-3c63304e5515.png)

## How to play

On Ubuntu, `gnome-mahjongg`'s assets located in `/usr/share/gnome-mahjongg/` are automatically used.

```sh
sudo apt install -y gnome-mahjongg
cargo run --release
```

On Windows and other systems where `gnome-mahjongg` is not available, clone [gnome-mahjongg's repository](https://gitlab.gnome.org/GNOME/gnome-mahjongg) and provide its assets with command-line options:

```sh
git clone https://gitlab.gnome.org/GNOME/gnome-mahjongg.git
cargo run --release -- -t gnome-mahjongg/data/postmodern.svg -m gnome-mahjongg/data/mahjongg.map
```

You can also play with [KMahjongg](https://github.com/KDE/kmahjongg)'s layouts:

```sh
sudo apt install -y kmahjongg
cargo run --release -- -m /usr/share/kmahjongg/layouts

# or
git clone https://github.com/KDE/kmahjongg.git
cargo run --release -- -m kmahjongg/layouts
```

## Command-line options

```
-w, --width <width>              Width of window in pixels [default: 900]
-h, --height <height>            Height of window in pixels [default: 600]
-t, --theme <theme>              Theme file (GNOME Mahjongg format) [default: /usr/share/gnome-
                                    mahjongg/themes/postmodern.svg]
-m, --map <map>...               Map files or directories containing map files (GNOME Mahjongg or KMahjongg format)
                                    [default: /usr/share/gnome-mahjongg/maps/]
-b, --background <background>    Background color [default: #34385b]
```
