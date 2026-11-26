import hashlib
import json
from pathlib import Path
import shutil
import sys
import tomllib
import zipfile


app = Path(sys.argv[1])
target = sys.argv[2]
spec = tomllib.loads(Path("gizlet.toml").read_text(encoding="utf-8"))
package, = spec["packages"]
if package["target"] != target:
    sys.exit(f"Built for {target}, but gizlet.toml declares {package['target']}")

manifest = {"schema": 1}
for key in ("id", "name", "names", "version", "description", "descriptions", "runtime", "color"):
    manifest[key] = spec[key]
for key in ("target", "tier", "entry"):
    manifest[key] = package[key]
manifest["icon"] = "icon.png"
shutil.copyfile(spec["icon"], app / manifest["icon"])
(app / "manifest.json").write_text(
    json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
)

output = app.parent / "{id}-{version}-{target}-{tier}.gzl".format(**manifest)
checksum_path = app / "CHECKSUMS"
paths = sorted(path for path in app.rglob("*") if path.is_file() and path != checksum_path)
checksums = []
for path in paths:
    with path.open("rb") as source:
        digest = hashlib.file_digest(source, "sha256").hexdigest()
    checksums.append(f"{digest}  {path.relative_to(app).as_posix()}\n")
checksum_path.write_text("".join(checksums), encoding="utf-8")
with zipfile.ZipFile(output, "w", zipfile.ZIP_DEFLATED, allowZip64=False) as archive:
    for path in [*paths, checksum_path]:
        archive.write(path, path.relative_to(app).as_posix())
print(output)
