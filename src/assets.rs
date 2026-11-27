//! Models, textures and sounds, loaded once after the window opens.
//!
//! The models come from `assets/models/props.mesh` (KayKit, Poly Pizza; see
//! `assets/licenses/CREDITS.md`), the tiling materials from ambientCG, the
//! heroes from `assets/heroes/`. A few textures are made in code: the belt's
//! slats, water, shoji paper and the kitchen curtain. The sounds are Tidewheel
//! Sushi's. Everything is found under `assets/` in the app's folder:
//! `GIZLET_APP_DIR` on Gizlet OS, else next to the program, else the working
//! directory (`cargo run`).

use crate::m3::V3;
use crate::mesh::{bounds, Pack, Part};
use crate::render::make_texture;
use raylib_sys::{self as ffi, Mesh, Music, Sound, Texture2D};
use std::collections::HashMap;
use std::ffi::CString;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// A model on the GPU, with its parts kept on the CPU for building rooms.
pub struct Prop {
    pub meshes: Vec<(Mesh, Option<usize>)>,
    pub parts: Vec<Part>,
    pub lo: V3,
    pub hi: V3,
}

impl Prop {
    pub fn size(&self) -> V3 {
        self.hi - self.lo
    }
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

/// Textures made in code, by index into `Assets::textures`.
pub struct Made {
    pub slats: usize,
    pub water: usize,
    pub paper: usize,
    pub steel: usize,
}

pub struct Assets {
    pub dir: PathBuf,
    pub props: HashMap<String, Prop>,
    pub textures: Vec<Texture2D>,
    pub by_name: HashMap<String, usize>,
    pub made: Made,
    pub sounds: Option<Sounds>,
    pub music: Option<Music>,
}

// Texture, mesh and sound handles are plain numbers owned by the one thread that draws.
unsafe impl Sync for Assets {}
unsafe impl Send for Assets {}

static ASSETS: OnceLock<Assets> = OnceLock::new();

pub fn get() -> &'static Assets {
    ASSETS.get().expect("assets are loaded after the window opens")
}

impl Assets {
    pub fn prop(&self, name: &str) -> &Prop {
        self.props.get(name).unwrap_or_else(|| panic!("no model named {name}"))
    }

    pub fn tex(&self, name: &str) -> usize {
        *self.by_name.get(name).unwrap_or_else(|| panic!("no texture named {name}"))
    }

    pub fn texture(&self, i: usize) -> Texture2D {
        self.textures[i]
    }
}

pub fn root() -> PathBuf {
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
    tries.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")));
    tries.push(PathBuf::from("."));
    tries.into_iter().find(|d| d.join("assets/models/props.mesh").is_file()).unwrap_or_else(|| PathBuf::from("."))
}

fn cstr(p: &Path) -> CString {
    CString::new(p.to_string_lossy().as_bytes()).unwrap_or_default()
}

fn texture(p: &Path, repeat: bool) -> Result<Texture2D, String> {
    let mut t = unsafe { ffi::LoadTexture(cstr(p).as_ptr()) };
    if t.id == 0 {
        return Err(format!("cannot load {}", p.display()));
    }
    unsafe {
        ffi::GenTextureMipmaps(&mut t);
        ffi::SetTextureFilter(t, ffi::TextureFilter::TEXTURE_FILTER_TRILINEAR as i32);
        if repeat {
            ffi::SetTextureWrap(t, ffi::TextureWrap::TEXTURE_WRAP_REPEAT as i32);
        }
    }
    Ok(t)
}

fn hash(x: f32, y: f32) -> f32 {
    let h = (x * 127.1 + y * 311.7).sin() * 43758.547;
    h - h.floor()
}

/// Loads everything; call once after `InitWindow` (and `InitAudioDevice` for sounds).
pub fn load(audio: bool) -> Result<(), String> {
    let dir = root().join("assets");
    let raw = std::fs::read(dir.join("models/props.mesh")).map_err(|e| format!("cannot read the models: {e}"))?;
    let pack = Pack::parse(&raw)?;
    let mut textures = Vec::new();
    let mut by_name = HashMap::new();
    for name in &pack.textures {
        by_name.insert(name.clone(), textures.len());
        textures.push(texture(&dir.join("textures").join(name), false)?);
    }
    for name in ["floor_wood", "tatami", "deck", "plaster", "counter"] {
        by_name.insert(name.to_string(), textures.len());
        textures.push(texture(&dir.join("textures").join(format!("{name}.png")), true)?);
    }
    let mut add = |name: &str, t: Texture2D| {
        by_name.insert(name.to_string(), textures.len());
        textures.push(t);
        textures.len() - 1
    };
    // The belt: steel slats with dark joints, one slat per 0.2 of `u`.
    let slats = add(
        "slats",
        make_texture(64, |u, v| {
            let joint = ((u * 4.0).fract() < 0.1) as i32 as f32;
            let edge = ((v - 0.5).abs() > 0.42) as i32 as f32;
            let shine = 0.08 * (1.0 - (v - 0.35).abs() * 3.0).max(0.0);
            let k = 0.78 + shine - joint * 0.32 - edge * 0.25 + hash(u * 9.0, v * 7.0) * 0.03;
            let c = |b: f32| (b * k * 255.0).clamp(0.0, 255.0) as u8;
            (c(0.86), c(0.88), c(0.9), 255)
        }),
    );
    let water = add(
        "water",
        make_texture(128, |u, v| {
            let w = ((u * 6.283 * 3.0).sin() + (v * 6.283 * 2.0 + u * 4.0).sin()) * 0.5;
            let k = 0.85 + 0.15 * w + hash(u * 31.0, v * 17.0) * 0.04;
            ((40.0 * k) as u8, (120.0 * k) as u8, (150.0 * k) as u8, 255)
        }),
    );
    let paper = add(
        "paper",
        make_texture(64, |u, v| {
            let grid = ((u * 3.0).fract() < 0.07 || (v * 4.0).fract() < 0.06) as i32 as f32;
            let fibre = hash(u * 53.0, v * 47.0) * 0.05;
            let k = 1.0 - fibre;
            if grid > 0.0 {
                (92, 62, 40, 255)
            } else {
                ((246.0 * k) as u8, (238.0 * k) as u8, (222.0 * k) as u8, 255)
            }
        }),
    );
    let steel = add(
        "steel",
        make_texture(32, |u, _| {
            let k = 0.82 + 0.1 * (u * 6.283).sin();
            ((200.0 * k) as u8, (206.0 * k) as u8, (212.0 * k) as u8, 255)
        }),
    );
    let mut props = HashMap::new();
    for (name, parts) in pack.models {
        let meshes = parts
            .iter()
            .map(|p| {
                let b = crate::mesh::Builder {
                    pos: p.pos.clone(),
                    nrm: p.nrm.clone(),
                    uv: p.uv.clone(),
                    col: p.col.clone(),
                    idx: p.idx.clone(),
                };
                (b.upload(false), p.tex.map(|t| by_name[&pack.textures[t]]))
            })
            .collect();
        let (lo, hi) = bounds(&parts);
        // Parts keep the pack's texture numbers; map them to ours.
        let parts = parts.into_iter().map(|mut p| {
            p.tex = p.tex.map(|t| by_name[&pack.textures[t]]);
            p
        });
        props.insert(name, Prop { meshes, parts: parts.collect(), lo, hi });
    }
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
    let music = audio
        .then(|| unsafe {
            let m = ffi::LoadMusicStream(cstr(&dir.join("audio/lanterns-on-the-belt-compact.wav")).as_ptr());
            (m.frameCount > 0).then(|| {
                ffi::SetMusicVolume(m, 0.35);
                m
            })
        })
        .flatten();
    let made = Made { slats, water, paper, steel };
    ASSETS
        .set(Assets { dir, props, textures, by_name, made, sounds, music })
        .map_err(|_| "assets loaded twice".to_string())
}

/// Plays a sound when audio is available.
pub fn play(pick: impl FnOnce(&Sounds) -> Sound) {
    if let Some(s) = ASSETS.get().and_then(|a| a.sounds.as_ref()) {
        unsafe { ffi::PlaySound(pick(s)) };
    }
}
