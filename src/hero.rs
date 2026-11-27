//! The playable visitors: rigged, animated KayKit characters (Kay Lousberg,
//! CC0) loaded from glTF, after gizlet/little-prince's `hero.rs`.
//!
//! raylib only reads the file. The body parts are merged into one mesh, the
//! bone matrices are worked out here from two blended animations, and the
//! vertices are skinned on the CPU into one of two buffers in turn, so a
//! frame never rewrites the buffer the GPU may still be reading. A full
//! stomach pushes the belly vertices outward before skinning.

use crate::m3::{v3, M4, V3};
use crate::render::{Gl, Look};
use raylib_sys::{self as ffi, Mesh, Model, ModelAnimation, Texture2D, Transform};
use std::ffi::CString;
use std::path::Path;

type M = [f32; 16];

/// raymath's MatrixMultiply: `l` applied first, then `r`.
fn mul(l: &M, r: &M) -> M {
    let mut o = [0.0; 16];
    for i in 0..4 {
        for j in 0..4 {
            o[4 * i + j] = (0..4).map(|k| l[4 * i + k] * r[4 * k + j]).sum();
        }
    }
    o
}

fn scale(x: f32, y: f32, z: f32) -> M {
    [x, 0.0, 0.0, 0.0, 0.0, y, 0.0, 0.0, 0.0, 0.0, z, 0.0, 0.0, 0.0, 0.0, 1.0]
}

fn translate(x: f32, y: f32, z: f32) -> M {
    [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, x, y, z, 1.0]
}

/// raymath's QuaternionToMatrix.
fn rot(x: f32, y: f32, z: f32, w: f32) -> M {
    let (a2, b2, c2) = (x * x, y * y, z * z);
    let (ac, ab, bc) = (x * z, x * y, y * z);
    let (ad, bd, cd) = (w * x, w * y, w * z);
    [
        1.0 - 2.0 * (b2 + c2),
        2.0 * (ab + cd),
        2.0 * (ac - bd),
        0.0,
        2.0 * (ab - cd),
        1.0 - 2.0 * (a2 + c2),
        2.0 * (bc + ad),
        0.0,
        2.0 * (ac + bd),
        2.0 * (bc - ad),
        1.0 - 2.0 * (a2 + b2),
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
    ]
}

fn srt(t: &Transform) -> M {
    let (s, q, p) = (t.scale, t.rotation, t.translation);
    mul(&mul(&scale(s.x, s.y, s.z), &rot(q.x, q.y, q.z, q.w)), &translate(p.x, p.y, p.z))
}

fn srt_inv(t: &Transform) -> M {
    let (s, q, p) = (t.scale, t.rotation, t.translation);
    mul(&mul(&translate(-p.x, -p.y, -p.z), &rot(-q.x, -q.y, -q.z, q.w)), &scale(1.0 / s.x, 1.0 / s.y, 1.0 / s.z))
}

fn blend(a: &Transform, b: &Transform, k: f32) -> Transform {
    let l = |x: f32, y: f32| x + (y - x) * k;
    let (qa, mut qb) = (a.rotation, b.rotation);
    if qa.x * qb.x + qa.y * qb.y + qa.z * qb.z + qa.w * qb.w < 0.0 {
        qb = ffi::Quaternion { x: -qb.x, y: -qb.y, z: -qb.z, w: -qb.w };
    }
    let q = ffi::Quaternion { x: l(qa.x, qb.x), y: l(qa.y, qb.y), z: l(qa.z, qb.z), w: l(qa.w, qb.w) };
    let n = (q.x * q.x + q.y * q.y + q.z * q.z + q.w * q.w).sqrt().max(1e-6);
    Transform {
        translation: ffi::Vector3 {
            x: l(a.translation.x, b.translation.x),
            y: l(a.translation.y, b.translation.y),
            z: l(a.translation.z, b.translation.z),
        },
        rotation: ffi::Quaternion { x: q.x / n, y: q.y / n, z: q.z / n, w: q.w / n },
        scale: ffi::Vector3 { x: l(a.scale.x, b.scale.x), y: l(a.scale.y, b.scale.y), z: l(a.scale.z, b.scale.z) },
    }
}

/// What the visitor is doing this frame.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Act {
    Idle,
    /// Walking at this many metres a second.
    Walk(f32),
    /// Taking a plate and eating, 0..1 through it.
    Eat(f32),
    Cheer,
    /// Knocked over: the hospital, or a skeleton falling apart.
    Down,
    /// A skeleton putting itself back together.
    Rise,
    Hurt,
}

#[derive(Clone, Copy)]
struct Track {
    anim: usize,
    t: f32,
    rate: f32,
    looped: bool,
}

struct Ids {
    idle: usize,
    walk: usize,
    run: usize,
    eat: usize,
    pick: usize,
    cheer: usize,
    down: usize,
    rise: usize,
    hurt: usize,
}

pub struct Hero {
    _model: Model,
    anims: *mut ModelAnimation,
    count: usize,
    meshes: [Mesh; 2],
    buf: usize,
    tex: Texture2D,
    bind_inv: Vec<M>,
    bind_pos: Vec<f32>,
    bind_nrm: Vec<f32>,
    /// How far each vertex moves out at a full belly.
    belly: Vec<f32>,
    bone_ids: Vec<u8>,
    wts: Vec<f32>,
    out_pos: Vec<f32>,
    out_nrm: Vec<f32>,
    bones: usize,
    hand: usize,
    ids: Ids,
    cur: Track,
    prev: Option<(Track, f32)>,
    palette: Vec<[f32; 4]>,
    /// Model units from the feet to the top of the head.
    pub height: f32,
    foot: f32,
    /// The hand's position in model units, this frame.
    pub hand_pos: V3,
}

fn cname(raw: &[std::ffi::c_char]) -> String {
    let bytes: Vec<u8> = raw.iter().take_while(|&&c| c != 0).map(|&c| c as u8).collect();
    String::from_utf8_lossy(&bytes).into_owned()
}

impl Hero {
    pub fn load(path: &Path) -> Result<Hero, String> {
        let name = path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        let c = CString::new(path.to_string_lossy().as_bytes()).map_err(|e| e.to_string())?;
        if !path.is_file() {
            return Err(format!("no model at {}", path.display()));
        }
        unsafe {
            let model = ffi::LoadModel(c.as_ptr());
            let mut count = 0;
            let anims = ffi::LoadModelAnimations(c.as_ptr(), &mut count);
            let bones = model.skeleton.boneCount as usize;
            if bones == 0 || anims.is_null() || count == 0 || bones > 64 {
                return Err(format!("{name}.glb has no usable skeleton or animations"));
            }
            let bone_names: Vec<String> = (0..bones).map(|i| cname(&(*model.skeleton.bones.add(i)).name)).collect();
            let (mut pos, mut nrm, mut uv, mut ids, mut w, mut idx) = (vec![], vec![], vec![], vec![], vec![], vec![]);
            let mut tex = None;
            for m in 0..model.meshCount as usize {
                let mesh = *model.meshes.add(m);
                let n = mesh.vertexCount as usize;
                if mesh.boneIndices.is_null()
                    || mesh.boneWeights.is_null()
                    || mesh.texcoords.is_null()
                    || mesh.normals.is_null()
                {
                    continue;
                }
                let bi = std::slice::from_raw_parts(mesh.boneIndices, n * 4);
                let bw = std::slice::from_raw_parts(mesh.boneWeights, n * 4);
                // Leave out what hangs on the hand slots: weapons and shields.
                if bw[0] == 1.0 && bone_names.get(bi[0] as usize).is_some_and(|b| b.starts_with("handslot")) {
                    continue;
                }
                let base = (pos.len() / 3) as u16;
                pos.extend_from_slice(std::slice::from_raw_parts(mesh.vertices, n * 3));
                nrm.extend_from_slice(std::slice::from_raw_parts(mesh.normals, n * 3));
                uv.extend_from_slice(std::slice::from_raw_parts(mesh.texcoords, n * 2));
                ids.extend_from_slice(bi);
                w.extend_from_slice(bw);
                if mesh.indices.is_null() {
                    idx.extend((0..n as u16).map(|i| base + i));
                } else {
                    let tris = std::slice::from_raw_parts(mesh.indices, mesh.triangleCount as usize * 3);
                    idx.extend(tris.iter().map(|&i| base + i));
                }
                if tex.is_none() {
                    let mat = *model.materials.add(*model.meshMaterial.add(m) as usize);
                    tex = Some((*mat.maps).texture);
                }
            }
            let tex = tex.ok_or_else(|| format!("{name}.glb has no textured body"))?;
            ffi::SetTextureFilter(tex, ffi::TextureFilter::TEXTURE_FILTER_BILINEAR as i32);
            let ys = pos.chunks(3).map(|p| p[1]);
            let (foot, top) = ys.fold((f32::MAX, f32::MIN), |(a, b), y| (a.min(y), b.max(y)));
            let bone = |n: &str| bone_names.iter().position(|b| b == n);
            // The belly: vertices bound to the hips, spine and chest, in the
            // middle of the torso, move out, most at the front.
            let torso: Vec<usize> = ["hips", "spine", "chest"].iter().filter_map(|n| bone(n)).collect();
            let (y0, y1) = (foot + (top - foot) * 0.22, foot + (top - foot) * 0.55);
            let belly: Vec<f32> = (0..pos.len() / 3)
                .flat_map(|v| {
                    let k: f32 =
                        (0..4).filter(|&j| torso.contains(&(ids[v * 4 + j] as usize))).map(|j| w[v * 4 + j]).sum();
                    let (x, y, z) = (pos[v * 3], pos[v * 3 + 1], pos[v * 3 + 2]);
                    let mid = (y0 + y1) / 2.0;
                    let band = (1.0 - ((y - mid) / ((y1 - y0) / 2.0)).powi(2)).max(0.0);
                    let d = (x * x + z * z).sqrt().max(1e-3);
                    let front = 0.6 + 0.4 * (z / d).max(0.0);
                    let s = k * band * front * (top - foot) * 0.11;
                    [x / d * s, 0.0, z / d * s * 1.3]
                })
                .collect();
            let meshes = [0, 1].map(|_| upload(&pos, &nrm, &uv, &idx));
            let bind_inv = (0..bones).map(|i| srt_inv(&*model.skeleton.bindPose.add(i))).collect();
            let names: Vec<String> = (0..count as usize).map(|i| cname(&(*anims.add(i)).name)).collect();
            let find = |want: &[&str]| want.iter().find_map(|n| names.iter().position(|x| x == n)).unwrap_or(0);
            let ids_ = Ids {
                idle: find(&["Idle", "Unarmed_Idle"]),
                walk: find(&["Walking_A"]),
                run: find(&["Running_A", "Walking_A"]),
                eat: find(&["Use_Item", "Interact"]),
                pick: find(&["PickUp", "Interact"]),
                cheer: find(&["Cheer"]),
                down: find(&["Death_C_Skeletons", "Lie_Down"]),
                rise: find(&["Skeletons_Awaken_Standing", "Lie_StandUp", "Idle"]),
                hurt: find(&["Hit_A"]),
            };
            let hand = bone("handslot.r").or_else(|| bone("hand.r")).unwrap_or(0);
            let mut h = Hero {
                _model: model,
                anims,
                count: count as usize,
                meshes,
                buf: 0,
                tex,
                bind_inv,
                out_pos: pos.clone(),
                out_nrm: nrm.clone(),
                bind_pos: pos,
                bind_nrm: nrm,
                belly,
                bone_ids: ids,
                wts: w,
                bones,
                hand,
                cur: Track { anim: ids_.idle, t: 0.0, rate: 1.0, looped: true },
                ids: ids_,
                prev: None,
                palette: vec![[0.0; 4]; bones * 4],
                height: top - foot,
                foot,
                hand_pos: V3::default(),
            };
            h.pose(0.0);
            // Measure the standing figure, not the bind pose: the rig carries
            // its own scale.
            let ys = h.out_pos.chunks(3).map(|p| p[1]);
            let (foot, top) = ys.fold((f32::MAX, f32::MIN), |(a, b), y| (a.min(y), b.max(y)));
            h.foot = foot;
            h.height = top - foot;
            Ok(h)
        }
    }

    fn anim(&self, i: usize) -> &ModelAnimation {
        unsafe { &*self.anims.add(i.min(self.count - 1)) }
    }

    fn duration(&self, i: usize) -> f32 {
        self.anim(i).keyframeCount.max(1) as f32 / 60.0
    }

    fn sample(&self, tr: &Track) -> Vec<Transform> {
        let a = self.anim(tr.anim);
        let n = a.keyframeCount.max(1) as usize;
        let f = (tr.t * 60.0).max(0.0);
        let (i0, k) = (f.floor() as usize, f.fract());
        let (i0, i1) = if tr.looped { (i0 % n, (i0 + 1) % n) } else { (i0.min(n - 1), (i0 + 1).min(n - 1)) };
        unsafe {
            let p0 = *a.keyframePoses.add(i0);
            let p1 = *a.keyframePoses.add(i1);
            (0..self.bones).map(|b| blend(&*p0.add(b), &*p1.add(b), k)).collect()
        }
    }

    fn play(&mut self, anim: usize, rate: f32, looped: bool, fade: f32) {
        if anim == self.cur.anim {
            self.cur.rate = rate;
            return;
        }
        let t = if looped && self.cur.looped {
            self.cur.t / self.duration(self.cur.anim) * self.duration(anim)
        } else {
            0.0
        };
        self.prev = Some((self.cur, 1.0 - fade.clamp(0.01, 1.0)));
        self.cur = Track { anim, t, rate, looped };
    }

    /// Advances the animation for `act`, with the belly `0..1` round.
    pub fn update(&mut self, dt: f32, act: Act, belly: f32) {
        let ids = (
            self.ids.idle,
            self.ids.walk,
            self.ids.run,
            self.ids.eat,
            self.ids.pick,
            self.ids.cheer,
            self.ids.down,
            self.ids.rise,
            self.ids.hurt,
        );
        match act {
            Act::Idle => self.play(ids.0, 1.0, true, 1.0),
            Act::Walk(v) if v > 2.6 => self.play(ids.2, (v / 4.0).clamp(0.8, 1.3), true, 1.0),
            Act::Walk(v) => self.play(ids.1, (v / 1.7).clamp(0.6, 1.8), true, 1.0),
            Act::Eat(k) => {
                // Reach for the plate, then bring it to the mouth.
                let (anim, rate) =
                    if k < 0.3 { (ids.4, self.duration(ids.4) / 0.7) } else { (ids.3, self.duration(ids.3) / 0.9) };
                self.play(anim, rate, false, 1.0);
            }
            Act::Cheer => self.play(ids.5, 1.0, true, 1.0),
            Act::Down => self.play(ids.6, 1.0, false, 1.0),
            Act::Rise => self.play(ids.7, 1.4, false, 1.0),
            Act::Hurt => self.play(ids.8, 1.0, false, 1.0),
        }
        self.cur.t += dt * self.cur.rate;
        if let Some((p, k)) = &mut self.prev {
            p.t += dt * p.rate;
            *k += dt / 0.16;
            if *k >= 1.0 {
                self.prev = None;
            }
        }
        self.pose(belly);
    }

    fn pose(&mut self, belly: f32) {
        let mut pose = self.sample(&self.cur);
        if let Some((p, k)) = self.prev {
            let old = self.sample(&p);
            let k = k * k * (3.0 - 2.0 * k);
            pose = old.iter().zip(&pose).map(|(a, b)| blend(a, b, k)).collect();
        }
        if let Some(h) = pose.get(self.hand) {
            self.hand_pos = v3(h.translation.x, h.translation.y, h.translation.z);
        }
        for (b, t) in pose.iter().enumerate() {
            let m = mul(&self.bind_inv[b], &srt(t));
            for c in 0..4 {
                self.palette[b * 4 + c] = [m[c * 4], m[c * 4 + 1], m[c * 4 + 2], m[c * 4 + 3]];
            }
        }
        self.skin(belly.clamp(0.0, 1.2));
    }

    fn skin(&mut self, belly: f32) {
        let p = &self.palette;
        for v in 0..self.bind_pos.len() / 3 {
            let x = self.bind_pos[v * 3] + self.belly[v * 3] * belly;
            let y = self.bind_pos[v * 3 + 1];
            let z = self.bind_pos[v * 3 + 2] + self.belly[v * 3 + 2] * belly;
            let (nx, ny, nz) = (self.bind_nrm[v * 3], self.bind_nrm[v * 3 + 1], self.bind_nrm[v * 3 + 2]);
            let (mut o, mut n) = ([0.0f32; 3], [0.0f32; 3]);
            for k in 0..4 {
                let w = self.wts[v * 4 + k];
                if w == 0.0 {
                    continue;
                }
                let b = self.bone_ids[v * 4 + k] as usize * 4;
                let (c0, c1, c2, c3) = (p[b], p[b + 1], p[b + 2], p[b + 3]);
                for r in 0..3 {
                    o[r] += w * (c0[r] * x + c1[r] * y + c2[r] * z + c3[r]);
                    n[r] += w * (c0[r] * nx + c1[r] * ny + c2[r] * nz);
                }
            }
            let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt().max(1e-6);
            self.out_pos[v * 3..v * 3 + 3].copy_from_slice(&o);
            self.out_nrm[v * 3..v * 3 + 3].copy_from_slice(&[n[0] / l, n[1] / l, n[2] / l]);
        }
        self.buf ^= 1;
        let mesh = self.meshes[self.buf];
        for (slot, data) in [
            (ffi::RL_DEFAULT_SHADER_ATTRIB_LOCATION_POSITION, self.out_pos.as_slice()),
            (ffi::RL_DEFAULT_SHADER_ATTRIB_LOCATION_NORMAL, self.out_nrm.as_slice()),
        ] {
            unsafe {
                ffi::UpdateMeshBuffer(mesh, slot as i32, data.as_ptr().cast(), std::mem::size_of_val(data) as i32, 0)
            };
        }
    }

    /// The model matrix that stands the hero at `feet`, turned by `yaw`, `h` metres tall.
    pub fn place(&self, feet: V3, yaw: f32, h: f32) -> M4 {
        let k = h / self.height.max(1e-3);
        M4::translate(v3(0.0, -self.foot, 0.0))
            .then(&M4::scale(k, k, k))
            .then(&M4::yaw(yaw))
            .then(&M4::translate(feet))
    }

    pub fn draw(&self, gl: &Gl, m: &M4, look: &Look, ink: Option<f32>) {
        let mesh = self.meshes[self.buf];
        if let Some(w) = ink {
            gl.outline(mesh, m, w, crate::render::rgb(0x2A1A22));
        }
        gl.draw(mesh, Some(self.tex), m, look);
    }
}

fn upload(pos: &[f32], nrm: &[f32], uv: &[f32], idx: &[u16]) -> Mesh {
    let b = crate::mesh::Builder {
        pos: pos.to_vec(),
        nrm: nrm.to_vec(),
        uv: uv.to_vec(),
        col: vec![255; pos.len() / 3 * 4],
        idx: idx.to_vec(),
    };
    b.upload(true)
}
