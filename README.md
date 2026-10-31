# Flair for Horizon

A native mod for [Horizon](https://github.com/horizon-rust/horizon). Flair adds a cosmetics catalog to the title screen and the pause menu: animated capes, hats, wings, shoulder pets and auras, plus any you make yourself. Cosmetics show on your own player only; other players do not see them.

## Using the catalog

| Where | How |
|---|---|
| Title screen | Click **Flair** (in the row under Multiplayer) |
| In game | Press Esc, then click **Flair** (left of the small icon buttons) |

Pick a tab on the left, then click an item to wear it and click it again to take it off. You can wear one item from each tab at the same time. Your choices are remembered. In a world, the catalog shows your player on the right, wearing everything, turning toward the mouse. Press Esc or click outside the window to close it. The mouse wheel scrolls when a tab has more items than fit.

Hats grow a little when you wear a helmet, so they always sit on top of it. Wings and pets move out a little over a chestplate.

## What is included

| Tab | Items |
|---|---|
| Capes | Royal, Nebula, Ember, Prism, Frost, Tide, Aurora, Void Rift, Sakura, Storm, Code Rain, Synthwave |
| Hats | Bandana, Neon Bandana, Top Hat, Halo, Crown, Devil Horns |
| Back | Angel Wings, Dragon Wings, Fairy Wings |
| Pets | Cat, Black Cat, Mini Dragon, Void Dragon |
| Auras | Orbit, Helix, Sparkles, Embers, Snowfall |

Everything is animated: capes shift and shimmer, bandana tails sway, wings flap, pets look around and wag their tails, auras circle and drift.

## Your own cosmetics

Flair reads your own items from its folder:

```
%APPDATA%\horizon\horizon-mods\.horizon-mod-data\flair\
```

If you play through a launcher instance, it is the same path inside that instance's folder (for example `%APPDATA%\horizon\instances\<instance>\horizon-mods\.horizon-mod-data\flair\`). On Linux and macOS it is the `horizon-mods/.horizon-mod-data/flair/` folder next to your Horizon `options.txt`. Open the catalog again after adding files; they show up at the end of their tab.

### A cape: just a picture

Put a PNG in the `capes` folder.

| Kind | Image size |
|---|---|
| Plain cape | 64x32, the standard Minecraft cape layout |
| HD cape | 128x64, 256x128, 512x256 (shown at 64x32) |
| Animated cape | Frames stacked top to bottom, each half as tall as the image is wide, up to 64 frames |

### Anything else: a Blockbench model

Make the item in [Blockbench](https://www.blockbench.net) as a **Bedrock Entity** model, on top of the standard player model so it sits where you want it, and paint its texture there. Then make a folder for it inside `cosmetics` with these files:

| File | Needed | What it is |
|---|---|---|
| `cosmetic.json` | yes | The name and the tab, see below |
| `<anything>.geo.json` | yes | The model, from File, Export, Export Bedrock Geometry |
| `<anything>.png` | yes | The texture, at most 64x64 |
| `<anything>.animation.json` | no | An animation, from Animation, Export Animations. The first one plays in a loop |
| `<texture>.png.mcmeta` | no | Makes an animated texture play slower, see below |

`cosmetic.json`:

```json
{ "name": "Party Hat", "slot": "hat" }
```

| Key | Values | Default |
|---|---|---|
| `name` | The name shown in the catalog | The folder name |
| `slot` | `hat`, `back`, `pet`, `aura` or `cape` | Required |
| `attach` | `head` (follows the head), `body` (follows the body) or `root` (follows the whole player) | `head` for hats, `body` for back items and pets, `root` for auras |
| `glow` | `true` to keep it bright in the dark | `false` |

Model the item in the player's own coordinates: a hat goes on the player model's head, a pet on its shoulder. The bones keep their pivots and rotations, cubes can use box UV or per face UV, and animations can move, turn and scale any bone with keyframes.

An animated texture is a PNG with its frames stacked top to bottom, each frame as tall as the texture height set in Blockbench. It plays one frame per game tick (20 per second). To slow it down, add a file named like the image plus `.mcmeta`, the same way resource packs do:

```json
{ "animation": { "frametime": 4 } }
```

The repository has a finished example in `examples/cosmetics/party_hat`: copy that folder into `cosmetics` to try it.

If something does not show up, the game's log names the file and the reason.

### Limits

| Limit | Why |
|---|---|
| Textures up to 64x64 | Each cosmetic uses one 64x64 texture slot |
| Animations use keyframes only | Molang expressions are read as 0 |
| One animation per item | The first one in the file plays, in a loop |

## Install

In the Horizon launcher, use Add from GitHub with this repository's link. Then build your native mods in the launcher as usual.

## Build from source

You need Rust 1.95 or newer, Python 3.12 or newer, a Horizon checkout at the commit in `compatibility.json`, and the horizon-mixins tool at the revision listed there.

| Command | What it does |
|---|---|
| `cargo test` | Runs the tests of the models, textures, Blockbench reader, items and catalog layout |
| `python tools/bundle.py apply --horizon <horizon> --mixins-tool <horizon-mixins> --out <stage> --check-target <target>` | Applies the mod to Horizon and checks that the game compiles |
| `python tools/bundle.py package` | Writes `build/flair-<version>.hnmod` |
| `python tools/make_example.py` | Writes the example cosmetic in `examples/cosmetics/party_hat` |

Made for Horizon Nightly 2026.10.30 (commit `1da354b`) with horizon-mixins `3f99a5c`. Tested on Windows 11.

## License

MIT. See [LICENSE](LICENSE).
