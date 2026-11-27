#!/usr/bin/env python3
"""Fetches the game's CC0 models and textures and bakes them into `assets/`.

    python3 scripts/assets.py fetch   # download pinned sources into build/asset-src
    python3 scripts/assets.py bake    # write assets/models, assets/heroes, assets/textures

Every source is pinned: KayKit files to a Git commit, Poly Pizza models to
their static file, ambientCG materials to their release archive, and each
download is checked against the SHA-256 below. The baked files are committed
(Git LFS), so building the game does not need this script; it documents and
repeats where every model came from.

`bake` writes:
- `assets/models/props.mesh`: every static model, flattened into parts (see
  `src/mesh.rs` for the format), with node transforms applied and material
  colours baked into the vertex colours.
- `assets/heroes/*.glb`: the KayKit characters with only the animations the
  game plays.
- `assets/textures/*.png`: atlases and tiling materials, downscaled with `sips`
  when it is available (macOS).
- `assets/licenses/`: each pack's license and `CREDITS.md`.
"""

import base64
import hashlib
import io
import json
import math
import os
import shutil
import struct
import subprocess
import sys
import urllib.request
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SRC = ROOT / "build" / "asset-src"
OUT = ROOT / "assets"

GH = "https://raw.githubusercontent.com/KayKit-Game-Assets"
KAYKIT = {
    "restaurant": ("KayKit-Restaurant-Bits-1.0", "153c8a7535b48237854cb54ff6890679f8c574d1",
                   "addons/kaykit_restaurant_bits/Assets/gltf", "restaurantbits_texture.png"),
    "furniture": ("KayKit-Furniture-Bits-1.0", "96d5930a8dbdb363409bbc2d3341718b00e17c9c",
                  "addons/kaykit_furniture_bits/Assets/gltf", "furniturebits_texture.png"),
    "halloween": ("KayKit-Halloween-Bits-1.0", "6dc69bf6b2fa766a985754f35ec6a0324090e6c6",
                  "addons/kaykit_halloween_bits/Assets/gltf", "halloweenbits_texture.png"),
}

# Static models, by pack, as (name in the game, file stem).
PROPS = {
    "restaurant": [
        "plate", "chair_stool", "pillar_A", "kitchencounter_straight_A_backsplash",
        "kitchencounter_straight_decorated", "kitchencounter_sink_backsplash", "stove_multi_decorated",
        "extractorhood", "fridge_A_decorated", "shelf_papertowel_decorated", "dishrack_plates",
        "jar_A_medium", "jar_B_small", "jar_C_large", "menu", "table_round_A_small", "chair_A",
        "kitchentable_A", "crate", "bowl_small", "kitchencabinet",
    ],
    "furniture": [
        "shelf_B_large_decorated", "pictureframe_large_A", "pictureframe_medium", "cactus_medium_A",
        "cactus_small_A", "lamp_standing", "rug_rectangle_stripes_A", "pillow_A", "pillow_B", "table_low",
        "cabinet_small_decorated",
    ],
    "halloween": ["lantern_hanging", "lantern_standing", "post_lantern", "candle_triple", "fence", "fence_pillar"],
}

HEROES = {
    "Rogue_Hooded": ("KayKit-Character-Pack-Adventures-1.0", "672074b73ba276876a19e8816ecdc5241817ab47",
                     "addons/kaykit_character_pack_adventures"),
    "Knight": ("KayKit-Character-Pack-Adventures-1.0", "672074b73ba276876a19e8816ecdc5241817ab47",
               "addons/kaykit_character_pack_adventures"),
    "Mage": ("KayKit-Character-Pack-Adventures-1.0", "672074b73ba276876a19e8816ecdc5241817ab47",
             "addons/kaykit_character_pack_adventures"),
    "Skeleton_Minion": ("KayKit-Character-Pack-Skeletons-1.0", "15b62b9bad122f72926c10fb14d622c73819fa54",
                        "addons/kaykit_character_pack_skeletons"),
}

# The clips the game plays; everything else is left out of the baked files.
ANIMS = [
    "Idle", "Unarmed_Idle", "Walking_A", "Running_A", "PickUp", "Interact", "Use_Item", "Cheer",
    "Hit_A", "Lie_Down", "Lie_Idle", "Jump_Idle", "Death_C_Skeletons", "Skeletons_Awaken_Standing",
]

# Poly Pizza models (all CC0 1.0), as (name in the game, page, creator, file id).
POLY = [
    ("salmon_nigiri", "wM171KITJX", "Quaternius", "da378858-b019-4877-b4ce-f2a4ef9ff2e0"),
    ("maguro_nigiri", "PWAQTFyAyc", "Quaternius", "672c2c39-a9f6-4069-8af4-bf4a48f384ac"),
    ("octopus_nigiri", "ZInAhXSRBG", "Quaternius", "c3035904-b8a5-4a74-b0c0-3ecce439104f"),
    ("ebi_nigiri", "C1EqYHLk95", "Quaternius", "7156fa3c-1e14-4938-905b-57bdf887686b"),
    ("tamago_nigiri", "ghaDsCFZU8", "Quaternius", "d6cd96b4-1a33-44aa-804b-aedde9867a0e"),
    ("uni_roll", "6xtivel0gf", "Quaternius", "d72172b8-3e58-4663-b880-5d3eb2aebd40"),
    ("salmon_sashimi", "2BeZAlQ73h", "Quaternius", "7db94500-7f2f-470c-a9e9-b3435a18bafc"),
    ("salmon_roll", "IFy5J2ypXo", "Quaternius", "6176076f-af4c-477e-944c-7e0edb35483d"),
    ("dango", "c4iK3MOB3A", "Quaternius", "080178d0-b264-453e-b217-e5c29a15de92"),
    ("soy", "t3mJ20Q4YH", "Kenney", "1d9a7303-2313-4257-bfff-ca64afb79d37"),
]

# Animated characters from Poly Pizza (all CC0 1.0), copied whole: the chef is
# Quaternius's "Man" in a hat made in code.
POLY_HEROES = [
    ("Chef", "HMnuH5geEG", "Quaternius", "3746be88-6799-4817-929b-6bc067c47caa"),
    ("Woman", "QeIc9Mzjzs", "Quaternius", "f3ef54fe-e8db-4ac3-9e1b-d3dca2398dcf"),
    ("Student", "DgOCW9ZCRJ", "Quaternius", "1a8a9d55-9aa9-43c4-a031-d926e251d80a"),
]

# ambientCG materials (CC0 1.0), as (name in the game, asset id).
MATERIALS = [
    ("floor_wood", "WoodFloor051"),
    ("tatami", "Tatami001"),
    ("deck", "Planks037A"),
    ("plaster", "Plaster001"),
    ("counter", "Wood095"),
]

SHA256 = json.loads((Path(__file__).with_name("assets.lock.json")).read_text()) if (
    Path(__file__).with_name("assets.lock.json").is_file()) else {}


def sources():
    """Every download as (local path under SRC, url)."""
    out = []
    for pack, (repo, sha, folder, tex) in KAYKIT.items():
        base = f"{GH}/{repo}/{sha}"
        out.append((f"{pack}/LICENSE.txt", f"{base}/LICENSE.txt"))
        out.append((f"{pack}/{tex}", f"{base}/{folder}/{tex}"))
        for stem in PROPS[pack]:
            out.append((f"{pack}/{stem}.gltf", f"{base}/{folder}/{stem}.gltf"))
            out.append((f"{pack}/{stem}.bin", f"{base}/{folder}/{stem}.bin"))
    for name, (repo, sha, folder) in HEROES.items():
        base = f"{GH}/{repo}/{sha}"
        out.append((f"heroes/{name}.glb", f"{base}/{folder}/Characters/gltf/{name}.glb"))
        out.append((f"heroes/{repo}-LICENSE.txt", f"{base}/LICENSE.txt"))
    for name, _page, _who, fid in POLY:
        out.append((f"poly/{name}.glb", f"https://static.poly.pizza/{fid}.glb"))
    for name, _page, _who, fid in POLY_HEROES:
        out.append((f"poly-heroes/{name}.glb", f"https://static.poly.pizza/{fid}.glb"))
    for name, aid in MATERIALS:
        out.append((f"ambientcg/{aid}.zip", f"https://ambientcg.com/get?file={aid}_1K-JPG.zip"))
    return sorted(set(out))


def fetch():
    hashes = {}
    for rel, url in sources():
        path = SRC / rel
        if not path.is_file():
            path.parent.mkdir(parents=True, exist_ok=True)
            req = urllib.request.Request(url, headers={"User-Agent": "overeaten-assets/1"})
            with urllib.request.urlopen(req, timeout=60) as r:
                data = r.read()
            path.write_bytes(data)
            print(f"fetched {rel} ({len(data) // 1024} KB)")
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
        want = SHA256.get(rel)
        if want and want != digest:
            sys.exit(f"{rel}: SHA-256 {digest} does not match the lock {want}")
        hashes[rel] = digest
    lock = Path(__file__).with_name("assets.lock.json")
    if not SHA256:
        lock.write_text(json.dumps(hashes, indent=1, sort_keys=True) + "\n")
        print(f"wrote {lock.relative_to(ROOT)}")
    elif set(hashes) != set(SHA256):
        gone = set(SHA256) - set(hashes)
        if gone:
            sys.exit(f"the lock lists files the sources no longer have: {sorted(gone)}")
        lock.write_text(json.dumps(hashes, indent=1, sort_keys=True) + "\n")
        print(f"pinned {len(set(hashes) - set(SHA256))} new files in {lock.relative_to(ROOT)}")


# ---- glTF reading ------------------------------------------------------

COMP = {5120: ("b", 1), 5121: ("B", 1), 5122: ("h", 2), 5123: ("H", 2), 5125: ("I", 4), 5126: ("f", 4)}
NCOMP = {"SCALAR": 1, "VEC2": 2, "VEC3": 3, "VEC4": 4, "MAT4": 16}
NORM = {5120: 127.0, 5121: 255.0, 5122: 32767.0, 5123: 65535.0}


class Gltf:
    def __init__(self, path):
        raw = path.read_bytes()
        self.dir = path.parent
        self.bin = None
        if raw[:4] == b"glTF":
            n = struct.unpack_from("<I", raw, 12)[0]
            self.j = json.loads(raw[20:20 + n])
            off = 20 + n
            if off < len(raw):
                blen = struct.unpack_from("<I", raw, off)[0]
                self.bin = raw[off + 8:off + 8 + blen]
        else:
            self.j = json.loads(raw)
        self.buffers = []
        for b in self.j.get("buffers", []):
            uri = b.get("uri")
            if uri is None:
                self.buffers.append(self.bin)
            elif uri.startswith("data:"):
                self.buffers.append(base64.b64decode(uri.split(",", 1)[1]))
            else:
                self.buffers.append((self.dir / uri).read_bytes())
        used = set(self.j.get("extensionsUsed", [])) - {"KHR_materials_emissive_strength", "KHR_texture_transform"}
        if used:
            print(f"  note: {path.name} uses {sorted(used)}")

    def view(self, i):
        v = self.j["bufferViews"][i]
        b = self.buffers[v["buffer"]]
        o = v.get("byteOffset", 0)
        return b[o:o + v["byteLength"]], v.get("byteStride")

    def read(self, i):
        a = self.j["accessors"][i]
        fmt, size = COMP[a["componentType"]]
        nc = NCOMP[a["type"]]
        count = a["count"]
        data, stride = self.view(a["bufferView"])
        off = a.get("byteOffset", 0)
        stride = stride or size * nc
        out = []
        for k in range(count):
            vals = struct.unpack_from("<" + fmt * nc, data, off + k * stride)
            if a.get("normalized"):
                vals = tuple(x / NORM[a["componentType"]] for x in vals)
            out.append(vals)
        return out

    def image(self, tex_index):
        """The bytes and extension of a texture's image."""
        img = self.j["images"][self.j["textures"][tex_index]["source"]]
        if "uri" in img:
            if img["uri"].startswith("data:"):
                return base64.b64decode(img["uri"].split(",", 1)[1]), ".png"
            return (self.dir / img["uri"]).read_bytes(), Path(img["uri"]).suffix
        data, _ = self.view(img["bufferView"])
        return bytes(data), ".jpg" if img.get("mimeType") == "image/jpeg" else ".png"


def mat_mul(a, b):
    return [sum(a[r + 4 * k] * b[k + 4 * c] for k in range(4)) for c in range(4) for r in range(4)]


def node_matrix(n):
    if "matrix" in n:
        return list(n["matrix"])
    tx, ty, tz = n.get("translation", (0, 0, 0))
    x, y, z, w = n.get("rotation", (0, 0, 0, 1))
    sx, sy, sz = n.get("scale", (1, 1, 1))
    r = [1 - 2 * (y * y + z * z), 2 * (x * y + z * w), 2 * (x * z - y * w),
         2 * (x * y - z * w), 1 - 2 * (x * x + z * z), 2 * (y * z + x * w),
         2 * (x * z + y * w), 2 * (y * z - x * w), 1 - 2 * (x * x + y * y)]
    return [r[0] * sx, r[1] * sx, r[2] * sx, 0, r[3] * sy, r[4] * sy, r[5] * sy, 0,
            r[6] * sz, r[7] * sz, r[8] * sz, 0, tx, ty, tz, 1]


def apply(m, p, w):
    return tuple(m[r] * p[0] + m[4 + r] * p[1] + m[8 + r] * p[2] + m[12 + r] * w for r in range(3))


def parts_of(g, textures):
    """The model's primitives in world space, merged by texture:
    {texture name or "": [positions, normals, uvs, colors, indices]}."""
    j = g.j
    groups = {}
    scene = j.get("scenes", [{}])[j.get("scene", 0)]
    stack = [(i, [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1]) for i in scene.get("nodes", [])]
    while stack:
        ni, parent = stack.pop()
        n = j["nodes"][ni]
        m = mat_mul(parent, node_matrix(n))
        stack.extend((c, m) for c in n.get("children", []))
        if "mesh" not in n or "skin" in n:
            continue
        for prim in j["meshes"][n["mesh"]]["primitives"]:
            if prim.get("mode", 4) != 4:
                continue
            at = prim["attributes"]
            pos = g.read(at["POSITION"])
            nrm = g.read(at["NORMAL"]) if "NORMAL" in at else [(0, 1, 0)] * len(pos)
            uv = g.read(at["TEXCOORD_0"]) if "TEXCOORD_0" in at else [(0, 0)] * len(pos)
            col = g.read(at["COLOR_0"]) if "COLOR_0" in at else [(1, 1, 1, 1)] * len(pos)
            idx = [i[0] for i in g.read(prim["indices"])] if "indices" in prim else list(range(len(pos)))
            mat = j.get("materials", [])[prim["material"]] if "material" in prim else {}
            pbr = mat.get("pbrMetallicRoughness", {})
            factor = pbr.get("baseColorFactor", [1, 1, 1, 1])
            tex = ""
            if "baseColorTexture" in pbr:
                tex = textures(g, pbr["baseColorTexture"]["index"])
            det = (m[0] * (m[5] * m[10] - m[6] * m[9]) - m[4] * (m[1] * m[10] - m[2] * m[9])
                   + m[8] * (m[1] * m[6] - m[2] * m[5]))
            if det < 0:
                idx = [idx[k + d] for k in range(0, len(idx), 3) for d in (0, 2, 1)]
            gr = groups.setdefault(tex, [[], [], [], [], []])
            base = len(gr[0])
            for p, nn, t, c in zip(pos, nrm, uv, col):
                gr[0].append(apply(m, p, 1))
                q = apply(m, nn, 0)
                ln = math.sqrt(sum(x * x for x in q)) or 1
                gr[1].append(tuple(x / ln for x in q))
                gr[2].append(t)
                c = tuple(c) + (1,) * (4 - len(c))
                # Material colours are in linear light; the game draws in sRGB.
                srgb = [min(1, max(0, fc)) ** (1 / 2.2) for fc in factor[:3]]
                gr[3].append(tuple(int(round(255 * min(1, max(0, c[k] * (srgb[k] if k < 3 else factor[3])))))
                                   for k in range(4)))
            gr[4].extend(base + i for i in idx)
    return groups


def write_pack(path, models):
    """models: [(name, {tex: part})]. See src/mesh.rs for the format."""
    tex_names = sorted({t for _, parts in models for t in parts if t})
    out = io.BytesIO()
    out.write(b"OEM1")
    out.write(struct.pack("<I", len(tex_names)))
    for t in tex_names:
        b = t.encode()
        out.write(struct.pack("<H", len(b)) + b)
    entries = [(name, t, p) for name, parts in models for t, p in sorted(parts.items())]
    out.write(struct.pack("<I", len(entries)))
    tris = 0
    for name, t, (pos, nrm, uv, col, idx) in entries:
        if len(pos) > 65535:
            sys.exit(f"{name}: too many vertices for 16-bit indices")
        b = name.encode()
        out.write(struct.pack("<H", len(b)) + b)
        out.write(struct.pack("<i", tex_names.index(t) if t else -1))
        out.write(struct.pack("<II", len(pos), len(idx)))
        for arr, fmt in ((pos, "3f"), (nrm, "3f"), (uv, "2f")):
            out.write(b"".join(struct.pack("<" + fmt, *v) for v in arr))
        out.write(b"".join(struct.pack("<4B", *c) for c in col))
        out.write(struct.pack(f"<{len(idx)}H", *idx))
        if len(idx) % 2:
            out.write(b"\0\0")
        tris += len(idx) // 3
    path.write_bytes(out.getvalue())
    print(f"wrote {path.relative_to(ROOT)}: {len(models)} models, {len(entries)} parts, {tris} triangles, "
          f"{len(out.getvalue()) // 1024} KB")


def shrink(src, dst, size):
    """Copies an image to a PNG at most `size` pixels wide."""
    dst.parent.mkdir(parents=True, exist_ok=True)
    if shutil.which("sips"):
        subprocess.run(["sips", "-s", "format", "png", "-Z", str(size), str(src), "--out", str(dst)],
                       check=True, stdout=subprocess.DEVNULL)
    elif src.suffix == ".png":
        shutil.copyfile(src, dst)
    else:
        sys.exit("sips is needed to convert JPEG textures")


# ---- Characters ----------------------------------------------------------

def strip_glb(src, dst, keep):
    """Writes `src` without the animations not named in `keep`, dropping the
    data only they used."""
    g = Gltf(src)
    j = json.loads(json.dumps(g.j))
    names = [a.get("name") for a in j.get("animations", [])]
    j["animations"] = [a for a in j.get("animations", []) if a.get("name") in keep]
    missing = [k for k in keep if k not in names]
    acc_used = set()
    for m in j.get("meshes", []):
        for p in m["primitives"]:
            acc_used.update(p["attributes"].values())
            if "indices" in p:
                acc_used.add(p["indices"])
            for t in p.get("targets", []):
                acc_used.update(t.values())
    for s in j.get("skins", []):
        if "inverseBindMatrices" in s:
            acc_used.add(s["inverseBindMatrices"])
    for a in j["animations"]:
        for s in a["samplers"]:
            acc_used.update((s["input"], s["output"]))
    acc_map = {old: new for new, old in enumerate(sorted(acc_used))}
    views_used = {j["accessors"][a]["bufferView"] for a in acc_used if "bufferView" in j["accessors"][a]}
    views_used |= {img["bufferView"] for img in j.get("images", []) if "bufferView" in img}
    view_map = {}
    blob = bytearray()
    new_views = []
    for old in sorted(views_used):
        data, _ = g.view(old)
        v = dict(j["bufferViews"][old])
        while len(blob) % 4:
            blob.append(0)
        v["buffer"] = 0
        v["byteOffset"] = len(blob)
        blob.extend(data)
        view_map[old] = len(new_views)
        new_views.append(v)
    j["accessors"] = [dict(j["accessors"][old], bufferView=view_map[j["accessors"][old]["bufferView"]])
                      for old in sorted(acc_used)]
    j["bufferViews"] = new_views
    for img in j.get("images", []):
        if "bufferView" in img:
            img["bufferView"] = view_map[img["bufferView"]]
    for m in j.get("meshes", []):
        for p in m["primitives"]:
            p["attributes"] = {k: acc_map[v] for k, v in p["attributes"].items()}
            if "indices" in p:
                p["indices"] = acc_map[p["indices"]]
            p["targets"] = [{k: acc_map[v] for k, v in t.items()} for t in p.get("targets", [])]
            if not p["targets"]:
                del p["targets"]
    for s in j.get("skins", []):
        if "inverseBindMatrices" in s:
            s["inverseBindMatrices"] = acc_map[s["inverseBindMatrices"]]
    for a in j["animations"]:
        for s in a["samplers"]:
            s["input"], s["output"] = acc_map[s["input"]], acc_map[s["output"]]
    while len(blob) % 4:
        blob.append(0)
    j["buffers"] = [{"byteLength": len(blob)}]
    js = json.dumps(j, separators=(",", ":")).encode()
    while len(js) % 4:
        js += b" "
    total = 12 + 8 + len(js) + 8 + len(blob)
    dst.parent.mkdir(parents=True, exist_ok=True)
    with open(dst, "wb") as f:
        f.write(struct.pack("<4sII", b"glTF", 2, total))
        f.write(struct.pack("<I4s", len(js), b"JSON") + js)
        f.write(struct.pack("<I4s", len(blob), b"BIN\0") + bytes(blob))
    kept = [a["name"] for a in j["animations"]]
    print(f"wrote {dst.relative_to(ROOT)}: {len(kept)} of {len(names)} animations, {total // 1024} KB"
          + (f" (missing {missing})" if len(missing) < len(keep) and missing else ""))


def bake():
    if not SRC.is_dir():
        sys.exit("run `fetch` first")
    (OUT / "models").mkdir(parents=True, exist_ok=True)
    tex_dir = OUT / "textures"
    lic = OUT / "licenses"
    lic.mkdir(parents=True, exist_ok=True)

    embedded = {}

    def textures(g, index):
        """Names a model's texture and writes it beside the pack."""
        img = g.j["images"][g.j["textures"][index]["source"]]
        uri = img.get("uri", "")
        for pack, (_, _, _, atlas) in KAYKIT.items():
            if uri == atlas:
                return f"{pack}.png"
        data, ext = g.image(index)
        key = hashlib.sha256(data).hexdigest()[:10]
        if key not in embedded:
            name = f"embedded-{key}.png"
            tmp = SRC / "tmp" / f"{key}{ext}"
            tmp.parent.mkdir(parents=True, exist_ok=True)
            tmp.write_bytes(data)
            shrink(tmp, tex_dir / name, 256)
            embedded[key] = name
        return embedded[key]

    models = []
    for pack, stems in PROPS.items():
        shrink(SRC / pack / KAYKIT[pack][3], tex_dir / f"{pack}.png", 512)
        shutil.copyfile(SRC / pack / "LICENSE.txt", lic / f"KayKit-{pack}-LICENSE.txt")
        for stem in stems:
            models.append((stem, parts_of(Gltf(SRC / pack / f"{stem}.gltf"), textures)))
    for name, *_ in POLY:
        models.append((name, parts_of(Gltf(SRC / "poly" / f"{name}.glb"), textures)))
    write_pack(OUT / "models" / "props.mesh", models)

    for name, (repo, *_rest) in HEROES.items():
        strip_glb(SRC / "heroes" / f"{name}.glb", OUT / "heroes" / f"{name}.glb", ANIMS)
        shutil.copyfile(SRC / "heroes" / f"{repo}-LICENSE.txt", lic / f"{repo}-LICENSE.txt")

    for name, _page, _who, _fid in POLY_HEROES:
        shutil.copyfile(SRC / "poly-heroes" / f"{name}.glb", OUT / "heroes" / f"{name}.glb")
        print(f"copied assets/heroes/{name}.glb")

    for name, aid in MATERIALS:
        with zipfile.ZipFile(SRC / "ambientcg" / f"{aid}.zip") as z:
            color = next(n for n in z.namelist() if n.endswith("_Color.jpg"))
            tmp = SRC / "tmp" / f"{aid}.jpg"
            tmp.parent.mkdir(parents=True, exist_ok=True)
            tmp.write_bytes(z.read(color))
        shrink(tmp, tex_dir / f"{name}.png", 512)

    lines = [
        "# Credits", "",
        "Every model and texture in `assets/` is released under CC0 1.0 (public domain). Thank you!", "",
        "| Asset | Author | Source |", "| --- | --- | --- |",
    ]
    for pack, (repo, sha, *_r) in KAYKIT.items():
        lines.append(f"| {', '.join(PROPS[pack])} | Kay Lousberg (KayKit) | "
                     f"[{repo}](https://github.com/KayKit-Game-Assets/{repo}/tree/{sha}) |")
    for name, (repo, sha, _f) in HEROES.items():
        lines.append(f"| {name} (hero) | Kay Lousberg (KayKit) | "
                     f"[{repo}](https://github.com/KayKit-Game-Assets/{repo}/tree/{sha}) |")
    for name, page, who, _fid in POLY:
        lines.append(f"| {name} | {who} | [Poly Pizza](https://poly.pizza/m/{page}) |")
    for name, page, who, _fid in POLY_HEROES:
        lines.append(f"| {name} (character) | {who} | [Poly Pizza](https://poly.pizza/m/{page}) |")
    for name, aid in MATERIALS:
        lines.append(f"| {name} texture | ambientCG | [{aid}](https://ambientcg.com/view?id={aid}) |")
    (lic / "CREDITS.md").write_text("\n".join(lines) + "\n")
    print(f"wrote {(lic / 'CREDITS.md').relative_to(ROOT)}")


if __name__ == "__main__":
    cmd = sys.argv[1] if len(sys.argv) > 1 else ""
    if cmd == "fetch":
        fetch()
    elif cmd == "bake":
        bake()
    else:
        sys.exit(__doc__)
