# Flair for Horizon

A native mod for [Horizon](https://github.com/horizon-rust/horizon). Flair adds a cosmetics catalog to the title screen and the pause menu. Pick an animated cape, or wear your own. Cosmetics show on your own player only; other players do not see them.

## Using the catalog

| Where | How |
|---|---|
| Title screen | Click **Flair** (in the row under Multiplayer) |
| In game | Press Esc, then click **Flair** (left of the small icon buttons) |

Click a cape to wear it, and click it again to take it off. Your choice is remembered. Press Esc or click outside the window to close it. The mouse wheel scrolls when there are more capes than fit.

## Capes

Six animated capes come with the mod: Royal, Nebula, Ember, Prism, Frost and Tide.

### Your own capes

Put PNG files in this folder and open the catalog again:

```
%APPDATA%\horizon\horizon-mods\.horizon-mod-data\flair\capes\
```

On Linux and macOS it is the `horizon-mods/.horizon-mod-data/flair/capes/` folder next to your Horizon `options.txt`.

| Kind | Image size | Notes |
|---|---|---|
| Plain cape | 64x32 | The standard Minecraft cape layout |
| HD cape | 128x64, 256x128, 512x256 | Shown at 64x32 |
| Animated cape | Frames stacked top to bottom, each half as tall as the image is wide | Up to 64 frames |

An animated cape plays one frame per game tick (20 per second). To slow it down, add a file named like the image plus `.mcmeta`, for example `wave.png.mcmeta`, the same way resource packs do:

```json
{ "animation": { "frametime": 3 } }
```

The file name, without `.png`, is the name shown in the catalog.

## Coming next

Hats, back items, shoulder pets and auras have their tabs already and arrive in later versions.

## Install

In the Horizon launcher, use Add from GitHub with this repository's link. Then build your native mods in the launcher as usual.

## Build from source

You need Rust 1.95 or newer, Python 3.12 or newer, a Horizon checkout at the commit in `compatibility.json`, and the horizon-mixins tool at the revision listed there.

| Command | What it does |
|---|---|
| `cargo test` | Runs the tests of the cape art, the cape file reader and the catalog layout |
| `python tools/bundle.py apply --horizon <horizon> --mixins-tool <horizon-mixins> --out <stage> --check-target <target>` | Applies the mod to Horizon and checks that the game compiles |
| `python tools/bundle.py package` | Writes `build/flair-<version>.hnmod` |

Made for Horizon Nightly 2026.10.30 (commit `1da354b`) with horizon-mixins `3f99a5c`. Tested on Windows 11.

## License

MIT. See [LICENSE](LICENSE).
