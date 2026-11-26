//! The 3D look: one toon shader for every surface (texture × vertex colour,
//! a two-step sun, up to six point lights and a rim of light), an ink outline
//! for the hero, soft blob shadows, the camera and the draw counters the
//! stats panel shows.
//!
//! Shaders are written once in GLSL ES 1.00 style and given a prefix for the
//! context raylib made: OpenGL ES 2 or 3 on Gizlet OS, OpenGL 3.3 on the
//! desktop (the same scheme as gizlet/little-prince).

use crate::m3::{v3, M4, V3};
use raylib_sys::{self as ffi, Color, Material, Mesh, Shader, Texture2D};
use std::cell::{Cell, RefCell};
use std::ffi::CString;

const GLES: bool = cfg!(feature = "gizlet-os");
pub const LIGHTS: usize = 6;

fn source(vertex: bool, body: &str) -> CString {
    let es3 = unsafe { ffi::rlGetVersion() } == ffi::rlGlVersion::RL_OPENGL_ES_30 as i32;
    let head = match (GLES, es3, vertex) {
        (true, false, true) => "#version 100\n",
        (true, false, false) => concat!(
            "#version 100\n#ifdef GL_FRAGMENT_PRECISION_HIGH\nprecision highp float;\n#else\nprecision mediump float;\n#endif\n",
            "#define FRAG gl_FragColor\n"
        ),
        (true, true, true) => "#version 300 es\n#define attribute in\n#define varying out\n#define texture2D texture\n",
        (true, true, false) => concat!(
            "#version 300 es\nprecision highp float;\n#define varying in\n#define texture2D texture\n",
            "out vec4 fragOut;\n#define FRAG fragOut\n"
        ),
        (false, _, true) => "#version 330\n#define attribute in\n#define varying out\n#define texture2D texture\n",
        (false, _, false) => {
            "#version 330\n#define varying in\n#define texture2D texture\nout vec4 fragOut;\n#define FRAG fragOut\n"
        }
    };
    CString::new(format!("{head}{body}")).expect("shader source")
}

fn load_shader(vs: Option<&str>, fs: &str) -> Shader {
    let vs = vs.map(|s| source(true, s));
    let fs = source(false, fs);
    let sh = unsafe { ffi::LoadShaderFromMemory(vs.as_ref().map_or(std::ptr::null(), |s| s.as_ptr()), fs.as_ptr()) };
    if sh.id == unsafe { ffi::rlGetShaderIdDefault() } {
        eprintln!("overeaten: a shader failed to compile; using raylib's default");
    }
    sh
}

/// A shader and the uniform locations asked for so far.
pub struct Uniforms {
    pub shader: Shader,
    locs: RefCell<Vec<(&'static str, i32)>>,
}

impl Uniforms {
    fn new(shader: Shader) -> Uniforms {
        Uniforms { shader, locs: RefCell::new(Vec::new()) }
    }
    fn loc(&self, name: &'static str) -> i32 {
        if let Some(&(_, l)) = self.locs.borrow().iter().find(|(n, _)| *n == name) {
            return l;
        }
        let c = CString::new(name).unwrap();
        let l = unsafe { ffi::GetShaderLocation(self.shader, c.as_ptr()) };
        self.locs.borrow_mut().push((name, l));
        l
    }
    fn set(&self, name: &'static str, v: *const f32, kind: ffi::ShaderUniformDataType, n: i32) {
        let l = self.loc(name);
        if l >= 0 {
            unsafe { ffi::SetShaderValueV(self.shader, l, v as _, kind as i32, n) };
        }
    }
    pub fn f(&self, name: &'static str, v: f32) {
        self.set(name, &v, ffi::ShaderUniformDataType::SHADER_UNIFORM_FLOAT, 1);
    }
    pub fn v2(&self, name: &'static str, v: [f32; 2]) {
        self.set(name, v.as_ptr(), ffi::ShaderUniformDataType::SHADER_UNIFORM_VEC2, 1);
    }
    pub fn v3(&self, name: &'static str, v: [f32; 3]) {
        self.set(name, v.as_ptr(), ffi::ShaderUniformDataType::SHADER_UNIFORM_VEC3, 1);
    }
    pub fn v4(&self, name: &'static str, v: [f32; 4]) {
        self.set(name, v.as_ptr(), ffi::ShaderUniformDataType::SHADER_UNIFORM_VEC4, 1);
    }
    fn v3s(&self, name: &'static str, v: &[[f32; 3]]) {
        self.set(name, v.as_ptr().cast(), ffi::ShaderUniformDataType::SHADER_UNIFORM_VEC3, v.len() as i32);
    }
    fn v4s(&self, name: &'static str, v: &[[f32; 4]]) {
        self.set(name, v.as_ptr().cast(), ffi::ShaderUniformDataType::SHADER_UNIFORM_VEC4, v.len() as i32);
    }
}

const LIT_VS: &str = r#"
attribute vec3 vertexPosition;
attribute vec3 vertexNormal;
attribute vec2 vertexTexCoord;
attribute vec4 vertexColor;
uniform mat4 mvp;
uniform mat4 matModel;
uniform vec2 uvShift;
varying vec3 vN;
varying vec3 vW;
varying vec2 vUV;
varying vec4 vC;
void main() {
    vW = (matModel * vec4(vertexPosition, 1.0)).xyz;
    vN = (matModel * vec4(vertexNormal, 0.0)).xyz;
    vUV = vertexTexCoord + uvShift;
    vC = vertexColor;
    gl_Position = mvp * vec4(vertexPosition, 1.0);
}
"#;

const LIT_FS: &str = r#"
varying vec3 vN;
varying vec3 vW;
varying vec2 vUV;
varying vec4 vC;
uniform sampler2D texture0;
uniform vec4 colDiffuse;
uniform vec3 sunDir;
uniform vec3 sunCol;
uniform vec3 shadeCol;
uniform vec3 skyCol;
uniform vec3 camPos;
uniform vec4 lights[6];
uniform vec3 lightCols[6];
uniform vec4 tint;
uniform float emissive;
uniform float rim;
void main() {
    vec4 tex = texture2D(texture0, vUV);
    vec3 base = tex.rgb * vC.rgb * colDiffuse.rgb;
    base = mix(base, tint.rgb, tint.a);
    vec3 n = normalize(vN);
    float ndl = dot(n, sunDir);
    float lit = smoothstep(-0.05, 0.1, ndl) * 0.55 + smoothstep(0.3, 0.45, ndl) * 0.45;
    vec3 light = mix(shadeCol, sunCol, lit) + skyCol * (0.55 + 0.45 * n.y);
    vec3 col = base * light;
    for (int i = 0; i < 6; i++) {
        vec4 l = lights[i];
        if (l.w > 0.0) {
            vec3 d = l.xyz - vW;
            float dist = length(d);
            float k = max(0.0, 1.0 - dist / l.w);
            col += base * lightCols[i] * k * k * 1.7 * (0.45 + 0.55 * max(dot(n, d / max(dist, 0.001)), 0.0));
        }
    }
    vec3 v = normalize(camPos - vW);
    float rr = 1.0 - max(dot(n, v), 0.0);
    col += vec3(1.0, 0.93, 0.8) * smoothstep(0.62, 0.78, rr) * rim * (0.4 + 0.6 * lit);
    col += base * emissive;
    FRAG = vec4(col, tex.a * vC.a * colDiffuse.a);
}
"#;

const INK_VS: &str = r#"
attribute vec3 vertexPosition;
attribute vec3 vertexNormal;
uniform mat4 mvp;
uniform float width;
void main() {
    vec4 c = mvp * vec4(vertexPosition, 1.0);
    vec4 c2 = mvp * vec4(vertexPosition + vertexNormal * 0.05, 1.0);
    vec2 s = c2.xy / c2.w - c.xy / c.w;
    float l = length(s);
    s = l > 0.000001 ? s / l : vec2(0.0);
    c.xy += s * width * vec2(2.0 / 640.0, 2.0 / 480.0) * c.w;
    gl_Position = c;
}
"#;

const INK_FS: &str = r#"
uniform vec4 colDiffuse;
void main() {
    FRAG = colDiffuse;
}
"#;

/// A rainbow sheen over a card face, for the holographic cards: bands of
/// colour that drift with time and the card's tilt.
const HOLO_FS: &str = r#"
varying vec2 fragTexCoord;
varying vec4 fragColor;
uniform sampler2D texture0;
uniform vec4 colDiffuse;
uniform float time;
uniform float strength;
uniform float tilt;
void main() {
    vec4 t = texture2D(texture0, fragTexCoord) * fragColor * colDiffuse;
    float p = fragTexCoord.x * 1.2 + fragTexCoord.y * 0.7 - time * 0.25 + tilt;
    vec3 rb = 0.5 + 0.5 * cos(6.2831 * (p + vec3(0.0, 0.33, 0.67)));
    float band = pow(0.5 + 0.5 * sin(p * 16.0), 4.0);
    float sweep = pow(max(0.0, 1.0 - abs(fract(p * 0.5) - 0.5) * 5.0), 2.0);
    vec3 col = t.rgb + rb * strength * (0.25 + 0.5 * band) + vec3(1.0) * sweep * strength * 0.6;
    FRAG = vec4(col * t.a, t.a);
}
"#;

/// The light of a scene.
#[derive(Clone, Copy)]
pub struct Lighting {
    pub sun_dir: V3,
    pub sun: Color,
    pub shade: Color,
    pub sky: Color,
    /// Point lights: position, radius (0 for none) and colour.
    pub lights: [(V3, f32, Color); LIGHTS],
}

impl Lighting {
    /// Warm late-afternoon light through the windows.
    pub fn day() -> Lighting {
        Lighting {
            sun_dir: v3(-0.35, 0.85, 0.4).norm(),
            sun: rgb(0xFFF1DC),
            shade: rgb(0x8D7FA0),
            sky: rgb(0x2A2633),
            lights: [(v3(0.0, 0.0, 0.0), 0.0, rgb(0)); LIGHTS],
        }
    }

    pub fn add_light(&mut self, p: V3, r: f32, c: Color) {
        if let Some(slot) = self.lights.iter_mut().find(|l| l.1 <= 0.0) {
            *slot = (p, r, c);
        }
    }
}

pub const fn rgb(hex: u32) -> Color {
    Color { r: (hex >> 16) as u8, g: (hex >> 8) as u8, b: hex as u8, a: 255 }
}

pub fn vec3(c: Color) -> [f32; 3] {
    [c.r as f32 / 255.0, c.g as f32 / 255.0, c.b as f32 / 255.0]
}

pub fn scaled(c: Color, k: f32) -> Color {
    let s = |x: u8| (x as f32 * k).clamp(0.0, 255.0) as u8;
    Color { r: s(c.r), g: s(c.g), b: s(c.b), a: c.a }
}

/// How a mesh is drawn beyond its geometry.
#[derive(Clone, Copy)]
pub struct Look {
    pub color: Color,
    /// Mixed over the surface colour: (colour, amount).
    pub tint: (Color, f32),
    pub emissive: f32,
    pub rim: f32,
    pub uv_shift: (f32, f32),
}

impl Default for Look {
    fn default() -> Look {
        Look { color: rgb(0xFFFFFF), tint: (rgb(0), 0.0), emissive: 0.0, rim: 0.0, uv_shift: (0.0, 0.0) }
    }
}

#[derive(Clone, Copy, Default)]
pub struct Counters {
    pub draws: u32,
    pub tris: u32,
}

pub struct Gl {
    pub lit: Uniforms,
    pub ink: Uniforms,
    pub holo: Uniforms,
    lit_mat: Material,
    ink_mat: Material,
    pub white: Texture2D,
    pub shadow: Texture2D,
    pub halo: Texture2D,
    pub frame: Cell<Counters>,
    pub last: Cell<Counters>,
    pub cam_pos: Cell<V3>,
}

fn material(shader: Shader) -> Material {
    let mut m = unsafe { ffi::LoadMaterialDefault() };
    m.shader = shader;
    m
}

fn solid_texture(size: i32, f: impl Fn(f32, f32) -> (u8, u8, u8, u8)) -> Texture2D {
    let mut px = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let (r, g, b, a) = f((x as f32 + 0.5) / size as f32, (y as f32 + 0.5) / size as f32);
            px.extend([r, g, b, a]);
        }
    }
    unsafe {
        let img = ffi::Image {
            data: px.as_mut_ptr().cast(),
            width: size,
            height: size,
            mipmaps: 1,
            format: ffi::PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8 as i32,
        };
        let t = ffi::LoadTextureFromImage(img);
        ffi::SetTextureFilter(t, ffi::TextureFilter::TEXTURE_FILTER_BILINEAR as i32);
        t
    }
}

/// A texture made in code from `f(u, v)`, tiling.
pub fn make_texture(size: i32, f: impl Fn(f32, f32) -> (u8, u8, u8, u8)) -> Texture2D {
    let t = solid_texture(size, f);
    unsafe {
        ffi::SetTextureWrap(t, ffi::TextureWrap::TEXTURE_WRAP_REPEAT as i32);
    }
    t
}

impl Gl {
    pub fn new() -> Gl {
        let lit = Uniforms::new(load_shader(Some(LIT_VS), LIT_FS));
        let ink = Uniforms::new(load_shader(Some(INK_VS), INK_FS));
        let holo = Uniforms::new(load_shader(None, HOLO_FS));
        let white = solid_texture(4, |_, _| (255, 255, 255, 255));
        let shadow = solid_texture(64, |u, v| {
            let d = ((u - 0.5) * (u - 0.5) + (v - 0.5) * (v - 0.5)).sqrt() * 2.0;
            let a = (1.0 - d).clamp(0.0, 1.0);
            (20, 14, 24, (a * a * (3.0 - 2.0 * a) * 150.0) as u8)
        });
        let halo = solid_texture(64, |u, v| {
            let d = ((u - 0.5) * (u - 0.5) + (v - 0.5) * (v - 0.5)).sqrt() * 2.0;
            let a = (1.0 - d).clamp(0.0, 1.0);
            (255, 255, 255, (a * a * 255.0) as u8)
        });
        Gl {
            halo,
            lit_mat: material(lit.shader),
            ink_mat: material(ink.shader),
            lit,
            ink,
            holo,
            white,
            shadow,
            frame: Cell::new(Counters::default()),
            last: Cell::new(Counters::default()),
            cam_pos: Cell::new(V3::default()),
        }
    }

    /// Ends the frame's counting; the stats panel reads `last`.
    pub fn end_frame(&self) {
        self.last.set(self.frame.replace(Counters::default()));
    }

    fn count(&self, tris: i32) {
        let mut c = self.frame.get();
        c.draws += 1;
        c.tris += tris.max(0) as u32;
        self.frame.set(c);
    }

    /// Sets the lights for what follows.
    pub fn light(&self, l: &Lighting, cam: V3) {
        let u = &self.lit;
        u.v3("sunDir", l.sun_dir.arr());
        u.v3("sunCol", vec3(l.sun));
        u.v3("shadeCol", vec3(l.shade));
        u.v3("skyCol", vec3(l.sky));
        u.v3("camPos", cam.arr());
        let pos: Vec<[f32; 4]> = l.lights.iter().map(|(p, r, _)| [p.x, p.y, p.z, *r]).collect();
        let col: Vec<[f32; 3]> = l.lights.iter().map(|(_, _, c)| vec3(*c)).collect();
        u.v4s("lights", &pos);
        u.v3s("lightCols", &col);
        self.cam_pos.set(cam);
    }

    /// Draws `mesh` with texture `tex` (None for plain colour) placed by `m`.
    pub fn draw(&self, mesh: Mesh, tex: Option<Texture2D>, m: &M4, look: &Look) {
        if mesh.vertexCount == 0 {
            return;
        }
        let u = &self.lit;
        let (tc, ta) = look.tint;
        let t = vec3(tc);
        u.v4("tint", [t[0], t[1], t[2], ta]);
        u.f("emissive", look.emissive);
        u.f("rim", look.rim);
        u.v2("uvShift", [look.uv_shift.0, look.uv_shift.1]);
        let mat = self.lit_mat;
        unsafe {
            (*mat.maps).texture = tex.unwrap_or(self.white);
            (*mat.maps).color = look.color;
            ffi::DrawMesh(mesh, mat, m.rl());
        }
        self.count(mesh.triangleCount);
    }

    /// The ink outline of `mesh`: its back faces pushed out `width` canvas pixels.
    pub fn outline(&self, mesh: Mesh, m: &M4, width: f32, c: Color) {
        self.ink.f("width", width);
        let mat = self.ink_mat;
        unsafe {
            (*mat.maps).color = c;
            ffi::rlDrawRenderBatchActive();
            ffi::rlSetCullFace(ffi::rlCullMode::RL_CULL_FACE_FRONT as i32);
            ffi::DrawMesh(mesh, mat, m.rl());
            ffi::rlDrawRenderBatchActive();
            ffi::rlSetCullFace(ffi::rlCullMode::RL_CULL_FACE_BACK as i32);
        }
        self.count(mesh.triangleCount);
    }

    /// A soft dark ellipse on the ground at `p`, `rx` by `rz` metres.
    pub fn shadow(&self, p: V3, rx: f32, rz: f32, alpha: f32) {
        let a = (alpha.clamp(0.0, 1.0) * 255.0) as u8;
        let y = p.y + 0.012;
        unsafe {
            ffi::rlSetTexture(self.shadow.id);
            ffi::rlBegin(ffi::RL_QUADS as i32);
            ffi::rlColor4ub(255, 255, 255, a);
            ffi::rlNormal3f(0.0, 1.0, 0.0);
            ffi::rlTexCoord2f(0.0, 0.0);
            ffi::rlVertex3f(p.x - rx, y, p.z - rz);
            ffi::rlTexCoord2f(0.0, 1.0);
            ffi::rlVertex3f(p.x - rx, y, p.z + rz);
            ffi::rlTexCoord2f(1.0, 1.0);
            ffi::rlVertex3f(p.x + rx, y, p.z + rz);
            ffi::rlTexCoord2f(1.0, 0.0);
            ffi::rlVertex3f(p.x + rx, y, p.z - rz);
            ffi::rlEnd();
            ffi::rlSetTexture(0);
        }
    }
}

impl Gl {
    /// A glowing disc facing the camera at `p`, `size` metres across,
    /// added onto what is behind it.
    pub fn halo(&self, view: &M4, p: V3, size: f32, c: Color) {
        let v = &view.0;
        let right = v3(v[0], v[4], v[8]) * (size / 2.0);
        let up = v3(v[1], v[5], v[9]) * (size / 2.0);
        let q = [p - right - up, p + right - up, p + right + up, p - right + up];
        unsafe {
            ffi::rlDrawRenderBatchActive();
            ffi::rlDisableDepthMask();
            ffi::BeginBlendMode(ffi::BlendMode::BLEND_ADDITIVE as i32);
            ffi::rlSetTexture(self.halo.id);
            ffi::rlBegin(ffi::RL_QUADS as i32);
            ffi::rlColor4ub(c.r, c.g, c.b, c.a);
            for (k, (u, w)) in [(0.0, 1.0), (1.0, 1.0), (1.0, 0.0), (0.0, 0.0)].iter().enumerate() {
                ffi::rlTexCoord2f(*u, *w);
                ffi::rlVertex3f(q[k].x, q[k].y, q[k].z);
            }
            ffi::rlEnd();
            ffi::rlSetTexture(0);
            ffi::rlDrawRenderBatchActive();
            ffi::EndBlendMode();
            ffi::rlEnableDepthMask();
        }
    }
}

/// A perspective camera drawing into part of the framebuffer.
#[derive(Clone, Copy, Debug)]
pub struct Camera {
    pub eye: V3,
    pub target: V3,
    /// Vertical field of view in radians.
    pub fovy: f32,
}

impl Camera {
    pub fn view(&self) -> M4 {
        M4::look_at(self.eye, self.target, v3(0.0, 1.0, 0.0))
    }

    pub fn proj(&self, aspect: f32) -> M4 {
        M4::perspective(self.fovy, aspect, 0.1, 80.0)
    }

    /// Projection and view together, for placing 2D things over the scene.
    pub fn view_proj(&self, aspect: f32) -> M4 {
        self.proj(aspect).mul(&self.view())
    }
}

/// Starts drawing 3D into the framebuffer rectangle `(x, y, w, h)` (pixels
/// from the top left of a target `fb_h` tall). End with `end3d`.
pub fn begin3d(cam: &Camera, x: i32, y: i32, w: i32, h: i32, fb_h: i32) {
    let p = cam.proj(w as f32 / h.max(1) as f32);
    let v = cam.view();
    unsafe {
        ffi::rlDrawRenderBatchActive();
        ffi::rlViewport(x, fb_h - y - h, w, h);
        ffi::rlMatrixMode(ffi::RL_PROJECTION as i32);
        ffi::rlPushMatrix();
        ffi::rlLoadIdentity();
        ffi::rlMultMatrixf(p.0.as_ptr());
        ffi::rlMatrixMode(ffi::RL_MODELVIEW as i32);
        ffi::rlLoadIdentity();
        ffi::rlMultMatrixf(v.0.as_ptr());
        ffi::rlEnableDepthTest();
        ffi::rlEnableBackfaceCulling();
    }
}

/// Ends `begin3d` and gives the whole target `fb_w`×`fb_h` back.
pub fn end3d(fb_w: i32, fb_h: i32) {
    unsafe {
        ffi::EndMode3D();
        ffi::rlViewport(0, 0, fb_w, fb_h);
    }
}

pub fn blend_alpha(f: impl FnOnce()) {
    unsafe {
        ffi::rlDrawRenderBatchActive();
        ffi::rlDisableDepthMask();
    }
    f();
    unsafe {
        ffi::rlDrawRenderBatchActive();
        ffi::rlEnableDepthMask();
    }
}
