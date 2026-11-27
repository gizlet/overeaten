# Overeaten · 自助大胃王

A 3D all-you-can-eat game for [Gizlet OS](https://github.com/gizlet/os) handhelds such as the R36S, written in Rust with raylib: you don't cook, you eat. Pick a hero, pay admission at the door, then finish the room's task before closing. The more you eat, the higher you score, but stay too long past the red line and you end in hospital.

![Carousel Sushi, room 2](docs/screenshots/dining.png)

Version 0.2 opens **Carousel Sushi · 转转寿司** with eight rooms, each with its own floor, obstacles and machinery and its own kind of task: a colonnade of pillars, twin belts joined by a narrow bridge, a raised tatami floor with an upper belt, sliding doors and golden plates, a dark room lit by lanterns, a tilting ferry deck and a grand slam of everything. Four heroes eat differently, and 35 cards in four books (rooms, heroes, sushi and secrets) come in four rarities, the holographic ones with a moving rainbow sheen. [DESIGN.md](DESIGN.md) describes the whole game.

| Title | Heroes | The ticket |
| --- | --- | --- |
| ![Title](docs/screenshots/title.png) | ![Heroes](docs/screenshots/heroes.png) | ![Ticket](docs/screenshots/levels.png) |
| **Lantern Night** | **The bill** | **The album** |
| ![Lantern Night](docs/screenshots/night.png) | ![Bill](docs/screenshots/result.png) | ![Album](docs/screenshots/card.png) |

## Getting the code

Models, textures, sounds and the font are stored with [Git LFS](https://git-lfs.com) (`.gitattributes`). Install it before cloning, or fetch them afterwards:

```sh
git lfs install
git clone https://github.com/gizlet/overeaten.git
git lfs pull    # only if the clone was made without Git LFS
```

Without them the build fails on the font and the game cannot load its models.

## Play on a Mac

Requirements: [rustup](https://rustup.rs), CMake (raylib-sys builds raylib for the desktop) and Git LFS.

```sh
cargo run --release
```

| Keys | Button | |
| --- | --- | --- |
| Arrows or WASD | D-pad | Walk, choose |
| Space, Enter or K | A | Take a plate, confirm |
| Backspace or J | B | Back |
| I | X | The album |
| U | Y | Show or hide the stats panel |
| Q / E | L1 / R1 | Turn the album's books |
| Esc or P | Start | Pause; settle the bill once the task is done |
| Tab | Select | 中文/English |

Gamepads work too. The window opens at the largest 4:3 size that fits the screen and can be resized; the picture is the device's 640×480 canvas, scaled, with the 3D scene drawn at the window's full resolution.

The stats panel (Y) shows the frame rate with a frame-time graph, CPU load of the machine and of the game, free and used memory, draw calls and triangles, the SoC temperature and clock on the handheld, and during a meal the numbers in play: the room and hero, stomach and digestion, the red-line countdown, the belt, the score and its multiplier, the deck's lean and the darkness.

| Variable | Effect |
| --- | --- |
| `GIZLET_SAVE_DIR` | Where progress is saved; `./save` by default |
| `GIZLET_LANG=en` | Start in English (Simplified Chinese otherwise; Select switches and remembers) |
| `GIZLET_SWAP_AB=1` | Swap A and B |
| `OVEREATEN_WINDOW=640x480` | Window size |
| `OVEREATEN_UNLOCK=1` | Every room and hero open |
| `OVEREATEN_STATS=1` | Start with the stats panel shown |
| `OVEREATEN_AUTOPILOT=1` | The visitor plays by itself |
| `OVEREATEN_MUTE=1` | No sound |

```sh
cargo test    # belts, doors, paths, tasks, eating, combos, hospital, heroes, saves; the autopilot clears all eight rooms
```

### Screenshots

`OVEREATEN_CAPTURE` renders one frame to a PNG and exits. Scenes are `title`, `heroes`, `levels`, `dining`, `danger`, `result`, `album`, `card` and `icon`; `OVEREATEN_LEVEL` (1–8) and `OVEREATEN_HERO` (1–4) choose the room and hero, `OVEREATEN_AT` the seconds into the meal (played by the autopilot), and `OVEREATEN_BOOK` and `OVEREATEN_CARD` the album page and card.

```sh
OVEREATEN_CAPTURE=$PWD/shots/night.png OVEREATEN_SCENE=dining OVEREATEN_LEVEL=6 OVEREATEN_AT=30 \
  OVEREATEN_UNLOCK=1 OVEREATEN_WINDOW=640x480 GIZLET_SAVE_DIR=/tmp/overeaten-shot ./target/release/overeaten
```

## Models and textures

Every model and texture is CC0 (public domain) and listed with its author and source in [assets/licenses/CREDITS.md](assets/licenses/CREDITS.md):

| What | From |
| --- | --- |
| The kitchen, counters, stools, pillars, plates, jars, crates | [KayKit Restaurant Bits](https://github.com/KayKit-Game-Assets/KayKit-Restaurant-Bits-1.0) by Kay Lousberg |
| Cushions, low tables, plants, lamps, pictures | [KayKit Furniture Bits](https://github.com/KayKit-Game-Assets/KayKit-Furniture-Bits-1.0) |
| Lanterns | [KayKit Halloween Bits](https://github.com/KayKit-Game-Assets/KayKit-Halloween-Bits-1.0) |
| The Traveller, Knight and Mage; the Skeleton | [KayKit Adventurers](https://github.com/KayKit-Game-Assets/KayKit-Character-Pack-Adventures-1.0) and [Skeletons](https://github.com/KayKit-Game-Assets/KayKit-Character-Pack-Skeletons-1.0) |
| The sushi | Quaternius, from [Poly Pizza](https://poly.pizza/search/sushi); the soy dish by Kenney |
| Wood floor, tatami, deck planks, plaster, counter wood | [ambientCG](https://ambientcg.com) |

`scripts/assets.py fetch` downloads the sources, pinned to KayKit commits, Poly Pizza files and ambientCG releases, and checks them against `scripts/assets.lock.json`; `scripts/assets.py bake` writes what the game loads: `assets/models/props.mesh` (every static model flattened into textured parts, see `src/mesh.rs`), `assets/heroes/*.glb` (the characters with only the 14 animations the game plays), `assets/textures/` (atlases and tiling materials at 512 px, with `sips` on macOS) and the licenses. The baked files are committed, so building does not need the script. The belt, the counters, the walls, the doors, the bridge and the deck are built in code from the floor plans.

## Run on Gizlet OS

On the device the game links Gizlet runtime 1's `libraylib.so.600` (DRM, OpenGL ES 3) and reads the handheld's controls from evdev. `scripts/package.sh` builds the application and writes its `.gzl` package.

Install Python 3.11 or newer, [zig](https://ziglang.org), and the Rust `aarch64-unknown-linux-gnu` target. Build runtime 1's `libraylib.so` in [gizlet/os](https://github.com/gizlet/os) with `bazel build --config=r36s //runtime:raylib`.

```sh
GIZLET_RAYLIB_LIB=/path/to/folder/with/libraylib.so scripts/package.sh
scp dist/io.gizlet.overeaten-0.2.0-aarch64-unknown-linux-gnu-lite.gzl root@DEVICE:/tmp/
ssh root@DEVICE gizlet install /tmp/io.gizlet.overeaten-0.2.0-aarch64-unknown-linux-gnu-lite.gzl
```

Hold Select + Start for two seconds to quit. The rendering follows [gizlet/little-prince](https://github.com/gizlet/little-prince)'s measurements on the R36S: 48 render batches in rotation, CPU skinning into two buffers in turn, and the static room merged by texture into a few meshes (about 20,000 triangles in 10 to 20 draw calls; a meal draws 35,000 to 55,000 triangles). The frame rate on the handheld has not been measured for this version yet; the stats panel shows it.

## Release

Change `version` in `gizlet.toml` and merge to `main`. CI tests and builds the
ARM64 package, creates `v<version>`, publishes a GitHub Release, and waits for
Gizlet Store validation. Rerun the original CI run to retry a failed release.

## Layout

| Path | Holds |
| --- | --- |
| `gizlet.toml` | Store metadata and package declaration |
| `src/main.rs` | The window, the frame loop and captures |
| `src/app.rs` | Screens, input handling, meal events into effects, the bill and saving |
| `src/game.rs` | A meal's rules: belts, plates, the visitor, the stomach, the clock, scoring, goals, cards earned and the autopilot |
| `src/level.rs` | The eight floor plans: belts, counters, obstacles, platforms, doors, lamps, the tilting deck; collisions and path finding |
| `src/tasks.rs` | The ten kinds of task and side goal, and how each is judged |
| `src/rules.rs` | Data: dishes, heroes, rooms, restaurants and cards |
| `src/scene.rs` | The rooms in 3D: static meshes built from the floor plans, and the plates, doors, water and visitor each frame |
| `src/render.rs` | Toon shader with point lights, ink outlines, shadows, halos, the holographic card shader and the 3D camera |
| `src/hero.rs` | Loading the KayKit characters, blending animations, CPU skinning and the growing belly |
| `src/mesh.rs` | The model pack, a mesh builder and the belt's shapes |
| `src/cards.rs` | 3D card portraits and the card faces and backs |
| `src/screens.rs`, `src/ui.rs` | Drawing each screen, the HUD and the stats panel |
| `src/assets.rs` | Loading models, textures and sounds |
| `src/stats.rs` | CPU, memory, temperature and clock readings (from gizlet/little-prince) |
| `src/gfx.rs`, `src/fx.rs` | The 640×480 canvas, shapes, text atlases; particles, floating numbers, banners and shake |
| `src/input.rs`, `src/evdev.rs` | Keyboard, gamepads and, on Gizlet OS, the handheld's evdev devices |
| `src/save.rs` | Stars, best scores, cards and the hero in `GIZLET_SAVE_DIR/progress.txt` |
| `src/text.rs` | Interface strings in Chinese and English |
| `scripts/assets.py`, `scripts/assets.lock.json` | Fetching and baking the CC0 models and textures |
| `scripts/package.sh`, `scripts/pack.py`, `scripts/zig-cc` | The device build and `.gzl` packaging |
| `assets/` | Models, heroes, textures, licenses, sounds, the font (bundled into the binary) and the icon |

## License

Code under MIT, see [LICENSE](LICENSE). Models and textures are CC0, see [assets/licenses](assets/licenses). The sounds come from [Tidewheel Sushi](https://github.com/gizlet/tidewheel-sushi) by the same author. The font is Noto Sans SC Bold under the SIL Open Font License 1.1 ([assets/fonts/noto_ofl.txt](assets/fonts/noto_ofl.txt)).
