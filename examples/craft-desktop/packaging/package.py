"""Stage a release binary and its notices, then build the native package."""
import argparse
import json
import shutil
import subprocess
import tomllib
from pathlib import Path

APP = Path(__file__).resolve().parents[1]
REPO = APP.parents[1]
VERSION = tomllib.loads((APP / "Cargo.toml").read_text(encoding="utf-8"))["package"]["version"]


def notices(destination, target):
    raw = subprocess.check_output(
        ["cargo", "metadata", "--locked", "--format-version", "1", "--filter-platform", target],
        cwd=APP,
    )
    metadata = json.loads(raw)
    active = {node["id"] for node in metadata["resolve"]["nodes"]}
    lines = ["CANexus Studio third-party license notices", ""]
    for package in sorted(metadata["packages"], key=lambda item: (item["name"], item["version"])):
        if package["id"] not in active or not package["source"]:
            continue
        name = f"{package['name']}-{package['version']}"
        root = Path(package["manifest_path"]).parent
        files = {p for p in root.iterdir() if p.is_file() and
                 p.name.upper().startswith(("LICENSE", "LICENCE", "COPYING", "NOTICE", "ATTRIBUTION"))}
        if package["source"].startswith("git+"):
            checkout = next((directory for directory in (root, *root.parents)
                             if (directory / ".git").exists()), root)
            files.update(p for p in checkout.iterdir() if p.is_file() and
                         p.name.upper().startswith(("LICENSE", "LICENCE", "COPYING", "NOTICE", "ATTRIBUTION")))
        if package["license_file"]:
            files.add(root / package["license_file"])
        if package["name"] == "epaint_default_fonts":
            files.update((root / "fonts").glob("*.txt"))
        lines.append(f"{name}: {package['license'] or 'See included license file'}")
        for file in sorted(files):
            origin = "package" if file.parent == root else ("fonts" if file.parent.name == "fonts" else "upstream")
            target_dir = destination / "licenses" / name / origin
            target_dir.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(file, target_dir / file.name)
    (destination / "THIRD-PARTY.txt").write_text("\n".join(lines) + "\n", encoding="utf-8")
    shutil.copyfile(REPO / "LICENSE", destination / "LICENSE.txt")


def linux(binary, output):
    staging = output / "linux-stage"
    control = staging / "DEBIAN"
    docs = staging / "usr/share/doc/canexus-studio"
    desktop = staging / "usr/share/applications"
    bin_dir = staging / "usr/bin"
    for directory in (control, docs, desktop, bin_dir):
        directory.mkdir(parents=True)
    installed = bin_dir / "canexus-studio"
    shutil.copyfile(binary, installed)
    installed.chmod(0o755)
    notices(docs, "x86_64-unknown-linux-gnu")
    (control / "control").write_text(
        "Package: canexus-studio\n"
        f"Version: {VERSION}\n"
        "Section: utils\nPriority: optional\nArchitecture: amd64\n"
        "Maintainer: CANexus Studio contributors\n"
        "Depends: libc6 (>= 2.39), libgcc-s1, libegl1, libgl1, libgl1-mesa-dri, libx11-6, "
        "libxcb1, libxkbcommon0, libxkbcommon-x11-0, libwayland-client0\n"
        "Description: CANexus Studio virtual CAN and CAN FD analyzer\n", encoding="utf-8",
    )
    (desktop / "canexus-studio.desktop").write_text(
        "[Desktop Entry]\nType=Application\nName=CANexus Studio\n"
        "Exec=canexus-studio\nTerminal=false\nCategories=Development;Engineering;\n", encoding="utf-8",
    )
    subprocess.run(
        ["dpkg-deb", "--root-owner-group", "--build", str(staging),
         str(output / f"CANexus-Studio-{VERSION}-linux-amd64.deb")], check=True,
    )


def windows(binary, output, iscc):
    staging = output / "windows-stage"
    staging.mkdir()
    shutil.copyfile(binary, staging / "canexus-studio.exe")
    notices(staging, "x86_64-pc-windows-msvc")
    subprocess.run(
        [iscc, f"/DAppVersion={VERSION}", f"/DStage={staging}", f"/O{output}",
         str(APP / "packaging/windows.iss")], check=True,
    )


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("platform", choices=["linux", "windows"])
    parser.add_argument("binary", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--iscc", default="ISCC.exe")
    args = parser.parse_args()
    args.output.mkdir(exist_ok=True, parents=True)
    if args.platform == "linux":
        linux(args.binary.resolve(), args.output.resolve())
    else:
        windows(args.binary.resolve(), args.output.resolve(), args.iscc)
