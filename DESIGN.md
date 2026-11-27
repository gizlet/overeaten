# Overeaten · 自助大胃王: design

An all-you-can-eat game in 3D. Nobody cooks: you are the customer. You pick a hero, pay admission at the door and, before the room closes, finish its task. Everything you eat scores points, so the more you eat the higher you score; but a stomach gauge keeps greed honest, and past its red line a short countdown runs to the hospital.

Everything here is original: the restaurants, the rooms, the tasks and the cards. The models and textures are open CC0 work by KayKit (Kay Lousberg), Quaternius, Kenney and ambientCG, listed in `assets/licenses/CREDITS.md`. The rooms borrow the idea of kitchen levels whose layout changes how you move, from co-operative cooking games, but none of their levels, characters or look.

## The loop

1. **Title.** A meal plays by itself behind the logo.
2. **Choose a hero.** Four heroes on a stage; locked ones stand in silhouette with what unlocks them.
3. **Choose a room.** Its ticket shows the room's machinery, the main task, two side goals drawn for this visit, the admission, the closing time, today's menu, the star lines and the best score. The room plays itself behind the ticket.
4. **Pay and eat.** Walk to the belts and take plates as they pass. Every dish has **points** and a **fullness**; the stomach digests steadily.
   - Different dishes within 5 s chain a **combo**: each link adds ×0.25 to the points, up to ×2.
   - Up to 100% you are fine; past 100% you walk slower; past the **red line (120%)** a countdown runs, and if it ends you go to hospital.
   - Finishing the main task does not end the meal: keep eating for points until closing, or settle the bill from the pause menu.
5. **The bill.** Admission, the food eaten, the combo bonus, the fullest moment, each side goal, the score, the stamp, the stars and the new cards turning over one by one.

| Outcome | When |
| --- | --- |
| **Well fed** | The main task is done by closing (or when you settle up). Score = food × combo + 40 per side goal. |
| **Unfinished** | Closing comes first. However high the score, the room is failed. |
| **Rule broken** | The task's rule is broken (too many plates for an efficiency task, the same dish twice in a row in the dark). |
| **Hospital** | The countdown runs out. The score is void, but a hidden card is waiting. |

Stars: one for clearing the room, a second and a third for the room's score lines. Stars unlock heroes; a room opens once the one before it is cleared.

## Numbers

| Rule | Value |
| --- | --- |
| Belt speed | 70 cm/s; a plate every 1.4 s while 100 cm of belt is free; plates go back after 1.6 laps |
| Reach | 112 cm from the feet, and the belt must be at counter height for the floor you stand on |
| A plate | 0.85 s from taking it to swallowing; it counts 0.32 s in |
| Red line, countdown | 120%; 5 s (7 s for the Knight) |
| Digestion | 1% a second (hero-dependent) |
| Side goal | +40 points each |

| Dish | Colour | Points | Fullness |
| --- | --- | --- | --- |
| Salmon Nigiri · 三文鱼握 | red | 18 | 22% |
| Tuna Nigiri · 金枪鱼握 | red | 17 | 20% |
| Octopus Nigiri · 章鱼握 | white | 12 | 15% |
| Shrimp Nigiri · 鲜虾握 | white | 13 | 17% |
| Tamago Nigiri · 玉子烧握 | yellow | 9 | 18% |
| Uni Gunkan · 海胆军舰 | yellow | 22 | 18% |
| Golden Sashimi · 金盘刺身 (on schedule) | gold | 30 | 16% |

Each dish rides a plate of its colour. All numbers live in `src/rules.rs`.

## Heroes

All four share KayKit's 41-bone rig and its animations: walking, running, reaching, eating, cheering, being knocked down and, for the skeleton, falling apart and getting up. The belly grows rounder as the stomach fills and the body flushes red in danger.

| Hero | Model | Trait | Unlock |
| --- | --- | --- | --- |
| Traveller · 旅人 | Rogue (hooded) | Even at everything | From the start |
| Iron-Gut Knight · 铁胃骑士 | Knight | A quarter more stomach, 7 s in the red, slower, steady on the leaning deck | 6 stars |
| Digestion Mage · 消化法师 | Mage | Smaller stomach, digests 1.6× as fast, sees the kitchen's next plate | 12 stars |
| Skeleton · 骷髅 | Skeleton Minion | Food falls through the ribs: half as filling and half the points; three bites within 2.5 s and it falls apart for 2 s | After a trip to hospital |

## Carousel Sushi: eight rooms

| # | Room | Floor and machinery | Main task (kind) |
| --- | --- | --- | --- |
| 1 | Opening Day · 开张 | One oval belt | Score 120 (quantity) |
| 2 | Colonnade · 柱廊 | Pillars and stools in the way, round the belt and beside it | Eat each of the six sushi (collect) |
| 3 | Twin Loops · 双环 | Two belts turning opposite ways, a pond between them and one narrow bridge | Red → white → yellow, three rounds (order) |
| 4 | Tatami · 榻榻米 | A raised tatami floor with steps; the upper belt can only be reached standing on it, and serves the richer dishes | Score 150 within 10 plates (efficiency) |
| 5 | Golden Hour · 金盘时刻 | Sliding doors shut off stretches of belt; a golden plate every 10 s | Three golden plates within 4 s of serving (timing) |
| 6 | Lantern Night · 暗夜 | Dark: only what the lanterns or your own light reach can be seen | Score 140, never the same dish twice in a row (restraint) |
| 7 | Ferry · 渡船 | A deck on the water that leans every 13 s and slides you aside | Close the meal between 90% and 100% full, 8+ plates (precision) |
| 8 | Grand Slam · 大满贯 | Twin belts, the bridge, pillars, a sliding door and dim lanterns | 6 s above the red line in all, and score 200 (risk) |

Each room draws two side goals from its pool, of kinds other than its main task's and different from each other, so a room changes from visit to visit. The autopilot test (`cargo test`) plays every room five times and must clear it at least four times without the hospital.

## Ten kinds of task

| Kind | Tests | Examples |
| --- | --- | --- |
| Quantity | Eating a lot | Score 120; eat 12 plates |
| Collect | Eating everything | Each of the six sushi; four different dishes; all three colours |
| Order | Planning | Red → white → yellow |
| Restraint | Holding back | Never pass 100%; no tamago; never the same dish twice in a row |
| Efficiency | Choosing well | 150 points within 10 plates |
| Timing | Being there | Golden plates within 4 s; the task done within 60 s; a golden plate at all |
| Precision | Ending right | 90–100% full at closing; at least 80% full at closing |
| Risk | Daring | Seconds spent above the red line |
| Space | Moving about | A plate in each quarter of the room |
| Combo | Chaining | A four-dish combo |

## Cards

35 cards in four books, each in one of four rarities:

| Book | Cards |
| --- | --- |
| Rooms · 关卡 (16) | Clearing each room, and three stars in it |
| Heroes · 主角 (8) | Clearing a room with each hero, and each hero's feat |
| Sushi · 寿司 (7) | Eating each dish once |
| Secrets · 隐藏 (4) | The hospital, ending exactly full, an eight-dish combo, eating nothing at all |

| Rarity | Frame |
| --- | --- |
| Common · 普通 | Walnut |
| Rare · 稀有 | Steel blue with gems in the corners |
| Holo · 闪卡 | Gold, with a rainbow sheen sweeping over the picture (a shader) and twinkles |
| Hidden · 隐藏 | Violet with drifting glitch lines; name and condition stay "???" until found |

Card pictures are 3D: the dish on its plate, the hero, the room's signature piece beside a plate, rendered once into textures. On the bill new cards drop in face down and turn over; rarer ones burst into gold sparks. In the album missing cards show their backs; choosing a card enlarges it with its picture turning in 3D. Filling a book adds two stars toward the heroes.

Progress is stored in `GIZLET_SAVE_DIR/progress.txt` as plain lines (`card`, `stars`, `best`, `hero`, `hospital`, `meals`, `lang`), written atomically.

## Controls

| Button | Menus | In a meal |
| --- | --- | --- |
| D-pad or left stick | Choose | Walk |
| A | Confirm | Take the plate in reach |
| B | Back | |
| X | The album | |
| Y | The stats panel | The stats panel |
| L1 / R1 | Album books | |
| Start | | Pause, settle up, give up |
| Select | 中文/English | |
| Select + Start, held 2 s | Quit (handled by Gizlet OS) | Quit |

## The screen at 640×480

The 3D scene fills the canvas from a high three-quarter camera that leans a little toward the visitor; the interface is drawn over it on the 640×480 canvas, scaled to any window, never cropped.

| Where | What |
| --- | --- |
| Top left | The room, the main task with its progress, the side goals with ticks and crosses |
| Top right | The clock to closing (red in the last 15 s); the next golden plate in Golden Hour |
| Over the plate in reach | A gold ring on the belt, and a bubble with the dish and the points it would score now ("can't see" in the dark) |
| Bottom left | The score, the combo and its draining timer, the plates eaten |
| Bottom right | The stomach from 0 to 150%: a notch at 100%, the red zone from 120%, and a ghost of the plate in reach, red when it would cross the line |
| Danger | Red edges pulse and a countdown ring floats over the visitor |

## Look

Warm, chunky low-poly under toon light: KayKit's kitchen and furniture on ambientCG wood, tatami and plaster, Quaternius's sushi on glazed plates, steel belts with moving slats, indigo noren over the kitchen passes, warm windows by day, lantern halos by night, the sea around the ferry. Every surface uses one shader: texture × vertex colour, a two-step sun, up to six point lights and a rim of light; the hero gets an ink outline and a soft shadow.

## Platform

Rust with `raylib-sys` 6.0; Gizlet runtime 1's `libraylib.so.600` on the device (DRM, OpenGL ES 3), raylib's desktop build on a Mac. Following gizlet/little-prince on the R36S: 48 render batches in rotation, CPU skinning into alternating buffers, the static room merged by texture into a few meshes. A meal draws 35,000 to 55,000 triangles in 30 to 60 draw calls.

## Later

More restaurants share the same street, ticket and card books: a hot-pot table where food cooks at its own pace, dim sum carts, refill noodles and a grill. The data is already arranged by restaurant and room (`RESTAURANTS`, `LEVELS`, card ids such as `sushi.3.clear`).
