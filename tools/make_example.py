import json
import pathlib
import struct
import zlib

ROOT = pathlib.Path(__file__).resolve().parent.parent
OUT = ROOT / "examples" / "cosmetics" / "party_hat"
SIZE = 32
FRAMES = 2
CUBES = [
    ("cone", [-3, 32, -3], [6, 2, 6], [0, 0]),
    ("cone", [-2, 34, -2], [4, 2, 4], [0, 8]),
    ("cone", [-1, 36, -1], [2, 2, 2], [0, 14]),
]
STRIPES = [(0xFF, 0x4F, 0x9A), (0x3C, 0xE8, 0xFF), (0xFF, 0xD2, 0x4A)]


def png(width, height, pixels):
    rows = b"".join(b"\x00" + bytes(c for p in pixels[y * width:(y + 1) * width] for c in p) for y in range(height))
    chunk = lambda kind, data: struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data))
    header = struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)
    return b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", header) + chunk(b"IDAT", zlib.compress(rows, 9)) + chunk(b"IEND", b"")


def texture():
    pixels = []
    for frame in range(FRAMES):
        for y in range(SIZE):
            for x in range(SIZE):
                if y < 20 and x < 24:
                    r, g, b = STRIPES[(x // 2 + frame) % len(STRIPES)]
                    pixels.append((r, g, b, 0xFF))
                elif 20 <= y < 22 and x < 4:
                    pixels.append((0xFF, 0xFF, 0xFF, 0xFF))
                else:
                    pixels.append((0, 0, 0, 0))
    return png(SIZE, SIZE * FRAMES, pixels)


def geometry():
    cone = {"name": "cone", "pivot": [0, 24, 0], "cubes": [{"origin": o, "size": s, "uv": uv} for _, o, s, uv in CUBES]}
    pompom = {"name": "pompom", "parent": "cone", "pivot": [0, 38, 0], "cubes": [{"origin": [-0.5, 38, -0.5], "size": [1, 1, 1], "uv": [0, 20]}]}
    description = {"identifier": "geometry.party_hat", "texture_width": SIZE, "texture_height": SIZE}
    return {"format_version": "1.12.0", "minecraft:geometry": [{"description": description, "bones": [cone, pompom]}]}


def animation():
    wobble = {"0.0": [0, 0, -12], "0.5": [0, 0, 12], "1.0": [0, 0, -12]}
    return {"format_version": "1.8.0", "animations": {"animation.party_hat.idle": {"loop": True, "animation_length": 1.0, "bones": {"pompom": {"rotation": wobble}}}}}


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / "cosmetic.json").write_text(json.dumps({"name": "Party Hat", "slot": "hat"}, indent=2) + "\n", encoding="utf-8")
    (OUT / "party_hat.geo.json").write_text(json.dumps(geometry(), indent=2) + "\n", encoding="utf-8")
    (OUT / "party_hat.animation.json").write_text(json.dumps(animation(), indent=2) + "\n", encoding="utf-8")
    (OUT / "party_hat.png").write_bytes(texture())
    (OUT / "party_hat.png.mcmeta").write_text(json.dumps({"animation": {"frametime": 6}}) + "\n", encoding="utf-8")
    print(OUT)


if __name__ == "__main__":
    main()
