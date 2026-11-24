//! The painted art and the sounds, loaded once after the window opens.
//!
//! The paintings come from Tidewheel Sushi (gizlet/tidewheel-sushi): the
//! room, the carousel island, the dishes in layers, the furniture and the
//! visitor's nine frames. They are found under `assets/` in the app's folder:
//! `GIZLET_APP_DIR` on Gizlet OS, else next to the program, else the working
//! directory (`cargo run`).

use raylib_sys::{self as ffi, Sound, Texture2D};
use std::ffi::CString;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

pub struct Dish {
    pub ceramic: Texture2D,
    pub contact: Texture2D,
    pub food: Texture2D,
    pub held: Texture2D,
}

pub struct Player {
    pub front: Texture2D,
    pub right: Texture2D,
    pub back: Texture2D,
    pub walk_front: Texture2D,
    pub walk_right: Texture2D,
    pub walk_back: Texture2D,
    pub reach: Texture2D,
    pub bite: Texture2D,
    pub savor: Texture2D,
}

pub struct Art {
    pub room: Texture2D,
    pub island: Texture2D,
    pub island_shadow: Texture2D,
    pub booth: Texture2D,
    pub condiments: Texture2D,
    pub tea: Texture2D,
    pub yuzu: Texture2D,
    pub plate_shadow: Texture2D,
    pub dishes: Vec<Dish>,
    pub player: Player,
    pub clock: Texture2D,
    pub plates: Texture2D,
    pub glow: Texture2D,
}

pub struct Sounds {
    pub admission: Sound,
    pub bite: Sound,
    pub card_flip: Sound,
    pub card_reveal: Sound,
    pub closing: Sound,
    pub recover: Sound,
    pub red: Sound,
}

pub struct Assets {
    pub art: Art,
    pub sounds: Option<Sounds>,
}

// Texture and sound handles are plain numbers owned by the one thread that draws.
unsafe impl Sync for Assets {}
unsafe impl Send for Assets {}

static ASSETS: OnceLock<Assets> = OnceLock::new();

pub fn get() -> &'static Assets {
    ASSETS.get().expect("assets are loaded after the window opens")
}

fn root() -> PathBuf {
    let mut tries = Vec::new();
    if let Some(d) = std::env::var_os("GIZLET_APP_DIR") {
        tries.push(PathBuf::from(d));
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(bin) = exe.parent() {
            tries.push(bin.to_path_buf());
            if let Some(up) = bin.parent() {
                tries.push(up.to_path_buf());
            }
        }
    }
    tries.push(PathBuf::from("."));
    tries.into_iter().find(|d| d.join("assets/art/room/room.png").is_file()).unwrap_or_else(|| PathBuf::from("."))
}

fn cstr(p: &Path) -> CString {
    CString::new(p.to_string_lossy().as_bytes()).unwrap_or_default()
}

/// A texture with mipmaps, so the large paintings shrink smoothly to the handheld's screen.
fn texture(p: &Path) -> Result<Texture2D, String> {
    let mut t = unsafe { ffi::LoadTexture(cstr(p).as_ptr()) };
    if t.id == 0 {
        return Err(format!("cannot load {}", p.display()));
    }
    unsafe {
        ffi::GenTextureMipmaps(&mut t);
        ffi::SetTextureFilter(t, ffi::TextureFilter::TEXTURE_FILTER_TRILINEAR as i32);
    }
    Ok(t)
}

/// Loads everything; call once after `InitWindow` (and `InitAudioDevice` for sounds).
pub fn load(audio: bool) -> Result<(), String> {
    let dir = root().join("assets");
    let t = |p: &str| texture(&dir.join("art").join(p));
    let dishes = crate::rules::DISHES
        .iter()
        .map(|d| {
            let n = d.art;
            Ok(Dish {
                ceramic: t(&format!("dishes/ceramics/{n}.png"))?,
                contact: t(&format!("dishes/contact/{n}.png"))?,
                food: t(&format!("dishes/ingredients/{n}.png"))?,
                held: t(&format!("dishes/held/{n}.png"))?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let art = Art {
        room: t("room/room.png")?,
        island: t("room/island-wave.png")?,
        island_shadow: t("room/island-shadow.png")?,
        booth: t("room/props/booth.png")?,
        condiments: t("room/props/condiments.png")?,
        tea: t("room/props/tea.png")?,
        yuzu: t("room/props/yuzu.png")?,
        plate_shadow: t("dishes/plate-shadow.png")?,
        dishes,
        player: Player {
            front: t("player/front.png")?,
            right: t("player/right.png")?,
            back: t("player/back.png")?,
            walk_front: t("player/walk-front.png")?,
            walk_right: t("player/walk-right.png")?,
            walk_back: t("player/walk-back.png")?,
            reach: t("player/reach.png")?,
            bite: t("player/bite.png")?,
            savor: t("player/savor.png")?,
        },
        clock: t("hud/clock.png")?,
        plates: t("hud/plates.png")?,
        glow: unsafe {
            let white = ffi::Color { r: 255, g: 255, b: 255, a: 255 };
            let clear = ffi::Color { r: 255, g: 255, b: 255, a: 0 };
            let img = ffi::GenImageGradientRadial(128, 128, 0.0, white, clear);
            let g = ffi::LoadTextureFromImage(img);
            ffi::UnloadImage(img);
            ffi::SetTextureFilter(g, ffi::TextureFilter::TEXTURE_FILTER_BILINEAR as i32);
            g
        },
    };
    let sounds = audio.then(|| {
        let s = |n: &str| unsafe { ffi::LoadSound(cstr(&dir.join("audio").join(n)).as_ptr()) };
        Sounds {
            admission: s("admission.wav"),
            bite: s("bite.wav"),
            card_flip: s("card-flip.wav"),
            card_reveal: s("card-reveal.wav"),
            closing: s("closing.wav"),
            recover: s("stomach-recover.wav"),
            red: s("stomach-red.wav"),
        }
    });
    ASSETS.set(Assets { art, sounds }).map_err(|_| "assets loaded twice".to_string())
}

/// Plays a sound when audio is available.
pub fn play(pick: impl FnOnce(&Sounds) -> Sound) {
    if let Some(s) = ASSETS.get().and_then(|a| a.sounds.as_ref()) {
        unsafe { ffi::PlaySound(pick(s)) };
    }
}
