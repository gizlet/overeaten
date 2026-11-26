# Overeaten · 吃吃吃

A reverse cooking game for [Gizlet OS](https://github.com/gizlet/os) handhelds such as the R36S: you don't cook, you eat. Pay admission at the door, then finish the restaurant's eating task before closing time without eating yourself into hospital.

![Carousel Sushi](docs/screenshots/dining.jpg)

The first restaurant, **Carousel Sushi · 转转寿司**, is playable. Walk round the counter island and take plates off the belt as they pass. Eat 160 coins' worth before closing while watching the stomach gauge: past 120% a five-second countdown to the hospital starts. Finished meals award collectible cards, which the album keeps. [DESIGN.md](DESIGN.md) describes the whole game: the core loop, the planned restaurants, the cards, the controls and the 640×480 layout.

| Title | The bill | The album |
| --- | --- | --- |
| ![Title](docs/screenshots/title.png) | ![Bill](docs/screenshots/result.png) | ![Album](docs/screenshots/album.png) |

## Getting the code

Images, sounds and the font are stored with [Git LFS](https://git-lfs.com) (`.gitattributes`). Install it before cloning, or fetch them afterwards:

```sh
git lfs install
git clone https://github.com/gizlet/overeaten.git
git lfs pull    # only if the clone was made without Git LFS
```

Without them the build fails on the font and the game cannot load its art.

## Play on a Mac

Requirements: [rustup](https://rustup.rs), CMake (raylib-sys builds raylib for the desktop) and Git LFS.

```sh
cargo run --release
```

| Keys | Button |
| --- | --- |
| Arrows or WASD | D-pad |
| Space, Enter or K | A: eat, confirm |
| Backspace or J | B: back |
| I | X: album |
| Esc or P | Start: pause |
| Tab | Select: 中文/English |

Gamepads work too. The window opens at 1280×960 and can be resized; the picture is the device's 640×480 canvas, scaled.

| Variable | Effect |
| --- | --- |
| `GIZLET_SAVE_DIR` | Where progress is saved; `./save` by default |
| `GIZLET_LANG=en` | Start in English (Simplified Chinese otherwise; Select switches and remembers) |
| `GIZLET_SWAP_AB=1` | Swap A and B |
| `OVEREATEN_WINDOW=640x480` | Window size |
| `OVEREATEN_AUTOPILOT=1` | The visitor plays by itself |
| `OVEREATEN_MUTE=1` | No sound |

```sh
cargo test    # belt, collisions, eating, combo, hospital, closing, cards, saves; the autopilot finishes the task
```

### Screenshots

`OVEREATEN_CAPTURE` renders one frame to a PNG and exits. Scenes are `title`, `street`, `dining`, `danger`, `result`, `album` and `icon`; `OVEREATEN_AT` sets the seconds into the meal, played by the autopilot.

```sh
OVEREATEN_CAPTURE=$PWD/shots/dining.png OVEREATEN_SCENE=dining OVEREATEN_AT=14 \
  OVEREATEN_WINDOW=640x480 GIZLET_SAVE_DIR=/tmp/overeaten-shot ./target/release/overeaten
```

`assets/icon.png` is the `icon` scene, cropped to its middle square and scaled to 512×512.

## Run on Gizlet OS

On the device the game links Gizlet runtime 1's `libraylib.so.600` and reads the handheld's controls from evdev. `scripts/package.sh` builds the application and writes its `.gzl` package.

Install Python 3.11 or newer, [zig](https://ziglang.org), and the Rust `aarch64-unknown-linux-gnu` target. Build runtime 1's `libraylib.so` in [gizlet/os](https://github.com/gizlet/os) with `bazel build --config=r36s //runtime:raylib`.

```sh
GIZLET_RAYLIB_LIB=/path/to/folder/with/libraylib.so scripts/package.sh
scp dist/io.gizlet.overeaten-0.1.1-aarch64-unknown-linux-gnu-lite.gzl root@DEVICE:/tmp/
ssh root@DEVICE gizlet install /tmp/io.gizlet.overeaten-0.1.1-aarch64-unknown-linux-gnu-lite.gzl
```

Hold Select + Start for two seconds to quit.

## Release

Change `version` in `gizlet.toml` and merge to `main`. CI tests and builds the
ARM64 package, creates `v<version>`, publishes a GitHub Release, and waits for
Gizlet Store validation. Rerun the original CI run to retry a failed release.

## Layout

| Path | Holds |
| --- | --- |
| `gizlet.toml` | Store metadata and package declaration |
| `src/main.rs` | The window, the frame loop and captures |
| `src/app.rs` | Screens, input handling and turning meal events into effects |
| `src/game.rs` | A meal's rules: the belt, the visitor, the stomach, the clock, cards earned and the autopilot |
| `src/rules.rs` | Data: dishes, numbers, restaurants and cards |
| `src/screens.rs` | Drawing each screen and the HUD |
| `src/art.rs` | The painted stage (room, island, dishes, furniture, the visitor's frames), plus cards and icons drawn from shapes |
| `src/assets.rs` | Loads the paintings and sounds from `assets/` |
| `src/gfx.rs` | The 640×480 canvas, shape helpers and text atlases |
| `src/fx.rs` | Particles, floating numbers, banners and screen shake |
| `src/input.rs`, `src/evdev.rs` | Keyboard, gamepads and, on Gizlet OS, the handheld's evdev devices |
| `src/save.rs` | Cards and records in `GIZLET_SAVE_DIR/progress.txt` |
| `src/text.rs` | Interface strings in Chinese and English |
| `build.rs` | Links runtime 1's raylib with the `gizlet-os` feature |
| `scripts/package.sh`, `scripts/pack.py`, `scripts/zig-cc` | The device build and `.gzl` packaging |
| `assets/` | The paintings (`art/`), sounds (`audio/`), the font (bundled into the binary) and the icon |

## License

The room, the island, the dishes, the furniture, the visitor and the sounds are the paintings and audio of [Tidewheel Sushi](https://github.com/gizlet/tidewheel-sushi), by the same author, reused here. Code under MIT, see [LICENSE](LICENSE). The font is Noto Sans SC Bold under the SIL Open Font License 1.1 ([assets/fonts/noto_ofl.txt](assets/fonts/noto_ofl.txt)).
