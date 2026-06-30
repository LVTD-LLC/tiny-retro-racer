#!/usr/bin/env python3
"""Generate a Tiny Retro Racer macOS .iconset without external dependencies."""

from __future__ import annotations

import os
import struct
import sys
import zlib


ICONSET_FILES = (
    (16, "icon_16x16.png"),
    (32, "icon_16x16@2x.png"),
    (32, "icon_32x32.png"),
    (64, "icon_32x32@2x.png"),
    (128, "icon_128x128.png"),
    (256, "icon_128x128@2x.png"),
    (256, "icon_256x256.png"),
    (512, "icon_256x256@2x.png"),
    (512, "icon_512x512.png"),
    (1024, "icon_512x512@2x.png"),
)

BASE_SIZE = 64


def color(hex_value: str, alpha: int = 255) -> tuple[int, int, int, int]:
    value = hex_value.lstrip("#")
    return (
        int(value[0:2], 16),
        int(value[2:4], 16),
        int(value[4:6], 16),
        alpha,
    )


def mix(
    first: tuple[int, int, int, int],
    second: tuple[int, int, int, int],
    amount: float,
) -> tuple[int, int, int, int]:
    return (
        round(first[0] + (second[0] - first[0]) * amount),
        round(first[1] + (second[1] - first[1]) * amount),
        round(first[2] + (second[2] - first[2]) * amount),
        round(first[3] + (second[3] - first[3]) * amount),
    )


def inside_rounded_rect(
    x: int,
    y: int,
    left: int,
    top: int,
    right: int,
    bottom: int,
    radius: int,
) -> bool:
    if not (left <= x < right and top <= y < bottom):
        return False

    cx = min(max(x, left + radius), right - radius - 1)
    cy = min(max(y, top + radius), bottom - radius - 1)
    return (x - cx) * (x - cx) + (y - cy) * (y - cy) <= radius * radius


def draw_rect(
    pixels: list[list[tuple[int, int, int, int]]],
    left: int,
    top: int,
    right: int,
    bottom: int,
    fill: tuple[int, int, int, int],
) -> None:
    for y in range(max(0, top), min(BASE_SIZE, bottom)):
        for x in range(max(0, left), min(BASE_SIZE, right)):
            pixels[y][x] = fill


def make_base_icon() -> list[list[tuple[int, int, int, int]]]:
    transparent = (0, 0, 0, 0)
    pixels = [[transparent for _ in range(BASE_SIZE)] for _ in range(BASE_SIZE)]

    top = color("#252b31")
    bottom = color("#070a0e")
    border = color("#5f6972")

    for y in range(BASE_SIZE):
        shade = mix(top, bottom, y / (BASE_SIZE - 1))
        for x in range(BASE_SIZE):
            if inside_rounded_rect(x, y, 2, 2, 62, 62, 12):
                noise = 8 if (x * 7 + y * 5) % 29 == 0 else 0
                pixels[y][x] = (
                    min(shade[0] + noise, 255),
                    min(shade[1] + noise, 255),
                    min(shade[2] + noise, 255),
                    255,
                )
            if inside_rounded_rect(x, y, 2, 2, 62, 62, 12) and not inside_rounded_rect(
                x, y, 4, 4, 60, 60, 10
            ):
                pixels[y][x] = border

    cx, cy = 32, 36
    road = color("#444d55")
    road_shadow = color("#2a3036")
    turf = color("#28c85d")
    inner_paint = color("#91ff8a")
    white_curb = color("#f2f5f7")
    red_curb = color("#ff365b")

    for y in range(BASE_SIZE):
        for x in range(BASE_SIZE):
            outer = ((x - cx) / 25.0) ** 2 + ((y - cy) / 19.0) ** 2
            inner = ((x - cx) / 14.5) ** 2 + ((y - cy) / 9.0) ** 2

            if inner < 1.0:
                pixels[y][x] = turf if (x + y) % 5 else mix(turf, color("#167a39"), 0.2)

            if outer <= 1.0 and inner >= 1.0:
                pixels[y][x] = road if y < cy + 3 else road_shadow

            if outer <= 1.0 and outer > 0.88 and inner >= 1.0:
                pixels[y][x] = red_curb if ((x + y) // 3) % 2 else white_curb

            if inner >= 1.0 and inner < 1.22 and outer <= 1.0:
                pixels[y][x] = inner_paint

    for y in range(39, 51):
        for x in range(45, 50):
            outer = ((x - cx) / 25.0) ** 2 + ((y - cy) / 19.0) ** 2
            inner = ((x - cx) / 14.5) ** 2 + ((y - cy) / 9.0) ** 2
            if outer <= 1.0 and inner >= 1.0:
                pixels[y][x] = color("#f7f9fb") if ((x + y) // 2) % 2 else color("#11161b")

    outline = color("#11161b")
    body = color("#ff375f")
    body_dark = color("#b71f42")
    glass = color("#57d7ff")
    headlight = color("#ffeb66")
    wheel = color("#05070a")

    draw_rect(pixels, 26, 19, 39, 39, outline)
    draw_rect(pixels, 28, 20, 37, 38, body)
    draw_rect(pixels, 29, 31, 36, 38, body_dark)
    draw_rect(pixels, 30, 23, 35, 28, glass)
    draw_rect(pixels, 30, 31, 35, 35, mix(glass, color("#1b4f7a"), 0.45))
    draw_rect(pixels, 24, 24, 28, 31, wheel)
    draw_rect(pixels, 37, 24, 41, 31, wheel)
    draw_rect(pixels, 25, 33, 28, 39, wheel)
    draw_rect(pixels, 37, 33, 40, 39, wheel)
    draw_rect(pixels, 29, 20, 31, 22, headlight)
    draw_rect(pixels, 34, 20, 36, 22, headlight)

    return pixels


def scale_nearest(
    pixels: list[list[tuple[int, int, int, int]]],
    size: int,
) -> list[list[tuple[int, int, int, int]]]:
    scaled = []
    for y in range(size):
        source_y = min(BASE_SIZE - 1, (y * BASE_SIZE + BASE_SIZE // 2) // size)
        row = []
        for x in range(size):
            source_x = min(BASE_SIZE - 1, (x * BASE_SIZE + BASE_SIZE // 2) // size)
            row.append(pixels[source_y][source_x])
        scaled.append(row)
    return scaled


def png_chunk(kind: bytes, data: bytes) -> bytes:
    checksum = zlib.crc32(kind)
    checksum = zlib.crc32(data, checksum)
    return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", checksum)


def write_png(
    path: str,
    pixels: list[list[tuple[int, int, int, int]]],
) -> None:
    height = len(pixels)
    width = len(pixels[0])
    raw = bytearray()

    for row in pixels:
        raw.append(0)
        for red, green, blue, alpha in row:
            raw.extend((red, green, blue, alpha))

    png = bytearray(b"\x89PNG\r\n\x1a\n")
    png.extend(
        png_chunk(
            b"IHDR",
            struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0),
        )
    )
    png.extend(png_chunk(b"IDAT", zlib.compress(bytes(raw), level=9)))
    png.extend(png_chunk(b"IEND", b""))

    with open(path, "wb") as output:
        output.write(png)


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: generate-macos-icon.py OUTPUT.iconset", file=sys.stderr)
        return 64

    output_dir = sys.argv[1]
    os.makedirs(output_dir, exist_ok=True)

    base_icon = make_base_icon()
    for size, filename in ICONSET_FILES:
        write_png(os.path.join(output_dir, filename), scale_nearest(base_icon, size))

    print(output_dir)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
