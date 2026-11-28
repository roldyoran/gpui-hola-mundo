#!/usr/bin/env python3
"""Genera assets/icon.svg, assets/icon.png y assets/icon.ico.

Requiere Pillow: `pip install pillow` o
`uv run --with pillow python tools/make_icon.py`.

El ICO usa entradas BMP/DIB (no PNG comprimido) para que `rc.exe` del
Windows SDK lo acepte sin sorpresas al embeberlo con `winresource`.
"""

from pathlib import Path

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "assets"

DARK = (9, 9, 11, 255)
BORDER = (39, 39, 42, 255)
WHITE = (250, 250, 250, 255)
GREEN = (52, 211, 153, 255)

SVG = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" role="img" aria-label="PNGTuber Lite">
  <rect width="256" height="256" rx="56" fill="#09090b"/>
  <rect x="1.5" y="1.5" width="253" height="253" rx="54.5" fill="none" stroke="#27272a" stroke-width="3"/>
  <path d="M92 170 L92 214 L135 178 Z" fill="#fafafa"/>
  <rect x="46" y="58" width="164" height="120" rx="34" fill="#fafafa"/>
  <rect x="88" y="100" width="14" height="36" rx="7" fill="#09090b"/>
  <rect x="121" y="86" width="14" height="64" rx="7" fill="#09090b"/>
  <rect x="154" y="104" width="14" height="28" rx="7" fill="#09090b"/>
  <circle cx="196" cy="60" r="14" fill="#34d399"/>
</svg>
"""

SIZES = [16, 24, 32, 48, 64, 128, 256]


def draw(size: int, supersample: int = 4) -> Image.Image:
    side = size * supersample
    image = Image.new("RGBA", (side, side), (0, 0, 0, 0))
    canvas = ImageDraw.Draw(image)

    def scale(value: float) -> int:
        return round(value * side / 256.0)

    canvas.rounded_rectangle([0, 0, side - 1, side - 1], radius=scale(56), fill=DARK)
    width = max(1, scale(3))
    inset = width / 2
    canvas.rounded_rectangle(
        [inset, inset, side - 1 - inset, side - 1 - inset],
        radius=scale(54.5),
        outline=BORDER,
        width=width,
    )
    canvas.polygon(
        [(scale(92), scale(170)), (scale(92), scale(214)), (scale(135), scale(178))],
        fill=WHITE,
    )
    canvas.rounded_rectangle(
        [scale(46), scale(58), scale(210), scale(178)], radius=scale(34), fill=WHITE
    )
    for x, y, w, h in [(88, 100, 14, 36), (121, 86, 14, 64), (154, 104, 14, 28)]:
        canvas.rounded_rectangle(
            [scale(x), scale(y), scale(x + w), scale(y + h)], radius=scale(7), fill=DARK
        )
    canvas.ellipse([scale(182), scale(46), scale(210), scale(74)], fill=GREEN)
    return image.resize((size, size), Image.Resampling.LANCZOS)


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    frames = {size: draw(size) for size in SIZES}

    frames[256].save(OUT / "icon.png")
    frames[256].save(
        OUT / "icon.ico",
        format="ICO",
        sizes=[(size, size) for size in SIZES],
        append_images=[frames[size] for size in SIZES if size != 256],
        bitmap_format="bmp",
    )
    (OUT / "icon.svg").write_text(SVG, encoding="utf-8")

    with Image.open(OUT / "icon.ico") as icon:
        print("ICO sizes:", sorted(icon.ico.sizes()))
    print(f"Escrito en {OUT}")


if __name__ == "__main__":
    main()
