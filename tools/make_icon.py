import pathlib
import struct
import sys
import zlib

ROOT = pathlib.Path(__file__).resolve().parent.parent
SCALE = 16
ART = [
    "................",
    ".....KKKKKK.....",
    "....KGGGGGGK....",
    "...KPPPPPPPPK...",
    "...KPPPGGPPPK...",
    "..KPPPGYYGPPPK..",
    "..KPPGYWWYGPPK..",
    "..KPPPGYYGPPPK..",
    "..KPPPPGGPPPPK..",
    ".KPPPPPPPPPPPPK.",
    ".KPDPPPPPPPPDPK.",
    ".KPDDPPPPPPDDPK.",
    "KPDDPPPPPPPPDDPK",
    "KDDPPDPPPPDPPDDK",
    "KGKGGKGGGGKGGKGK",
    ".K.KK.KKKK.KK.K.",
]
COLORS = {
    "K": (24, 16, 40, 255),
    "P": (108, 58, 214, 255),
    "D": (70, 34, 150, 255),
    "G": (242, 193, 78, 255),
    "Y": (255, 226, 140, 255),
    "W": (255, 250, 230, 255),
    ".": (0, 0, 0, 0),
}


def chunk(kind, data):
    return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data))


def png(art, scale):
    size = len(art) * scale
    rows = bytearray()
    for line in art:
        assert len(line) == len(art), line
        row = b"".join(bytes(COLORS[c]) * scale for c in line)
        rows += (b"\x00" + row) * scale
    header = struct.pack(">IIBBBBB", size, size, 8, 6, 0, 0, 0)
    return b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", header) + chunk(b"IDAT", zlib.compress(bytes(rows), 9)) + chunk(b"IEND", b"")


def main():
    target = ROOT / "icon.png"
    target.write_bytes(png(ART, SCALE))
    print(target, target.stat().st_size, "bytes")


if __name__ == "__main__":
    sys.exit(main())
