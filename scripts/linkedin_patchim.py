#!/usr/bin/env python3
"""LinkedIn 1080×1080 plate: patchim ㄱ (기역) vs ㄴ (니은).

Same plate as swimming-bookstore/learn-japanese katakana ネ/レ: cream wash,
tracked kicker, serif title, navy table, bookstore footer.

Left column is 악. Right column is 안. Under each syllable: the batchim,
then the reading, with the batchim sound highlighted. ㄱ and ㄴ are the
batchim. 기 and 그 are vowels, not batchim.
"""

from __future__ import annotations

from pathlib import Path

import cairo

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "docs" / "linkedin-patchim-giyeok-nieun.png"
SVG = ROOT / "docs" / "linkedin-patchim-giyeok-nieun.svg"

W = H = 1080
CREAM = (1.0, 0xF9 / 255, 0xEF / 255)
NAVY = (0x1D / 255, 0x35 / 255, 0x50 / 255)
TAUPE = (0x6A / 255, 0x5C / 255, 0x4E / 255)
CREAM_HEX = "#fff9ef"
NAVY_HEX = "#1d3550"
TAUPE_HEX = "#6a5c4e"

SERIF = "Liberation Serif"
SERIF_CJK = "Noto Serif CJK KR"
SANS = "Lato"

LEFT, TOP, RIGHT, BOTTOM = 56, 278, 1024, 930
MID = 540
# syllable, batchim, reading, index of the batchim sound in the reading
COLS = [
    ("악", "ㄱ", "ak", 1),
    ("안", "ㄴ", "an", 1),
]
TITLE = [
    ("Patchim: can you tell ", SERIF, 48, True),
    ("ㄱ", SERIF_CJK, 52, True),
    (" from ", SERIF, 48, True),
    ("ㄴ", SERIF_CJK, 52, True),
]
EXAMPLE = [
    ("Example: ", SERIF, 28, False),
    ("안", SERIF_CJK, 30, False),
    (" (inside)", SERIF, 28, False),
]


def set_font(cr: cairo.Context, family: str, size: float, bold: bool = False) -> None:
    cr.select_font_face(
        family,
        cairo.FONT_SLANT_NORMAL,
        cairo.FONT_WEIGHT_BOLD if bold else cairo.FONT_WEIGHT_NORMAL,
    )
    cr.set_font_size(size)


def center(cr: cairo.Context, x: float, y: float, text: str, family: str, size: float, fill, bold: bool = False) -> None:
    set_font(cr, family, size, bold=bold)
    cr.set_source_rgb(*fill)
    ext = cr.text_extents(text)
    cr.move_to(x - (ext.x_bearing + ext.width / 2), y - (ext.y_bearing + ext.height / 2))
    cr.show_text(text)


def advance(cr: cairo.Context, text: str, family: str, size: float, bold: bool = False) -> float:
    set_font(cr, family, size, bold=bold)
    return cr.text_extents(text).x_advance


def center_runs(cr: cairo.Context, x: float, baseline: float, runs: list[tuple[str, str, float, bool]], fill) -> float:
    """Draw mixed-font runs on one baseline, centered on x. Returns the start x."""
    widths = [advance(cr, text, family, size, bold) for text, family, size, bold in runs]
    cursor = x - sum(widths) / 2
    cr.set_source_rgb(*fill)
    for (text, family, size, bold), width in zip(runs, widths):
        set_font(cr, family, size, bold=bold)
        cr.move_to(cursor, baseline)
        cr.show_text(text)
        cursor += width
    return x - sum(widths) / 2


def render_png() -> None:
    surf = cairo.ImageSurface(cairo.FORMAT_ARGB32, W, H)
    cr = cairo.Context(surf)
    cr.set_source_rgb(*CREAM)
    cr.paint()

    set_font(cr, SANS, 20, bold=True)
    cr.set_source_rgb(*TAUPE)
    kicker = "KOREAN ESSENTIAL"
    gap = 7.2
    widths = [cr.text_extents(ch).width for ch in kicker]
    tw = sum(widths) + gap * (len(kicker) - 1)
    x = (W - tw) / 2
    y = 84
    for ch, wch in zip(kicker, widths):
        xb, yb, _tw, th, _xa, _ya = cr.text_extents(ch)
        cr.move_to(x - xb, y - (yb + th / 2))
        cr.show_text(ch)
        x += wch + gap

    center_runs(cr, 540, 176, TITLE, NAVY)
    center_runs(cr, 540, 228, EXAMPLE, TAUPE)

    cr.set_source_rgba(*NAVY, 0.045)
    cr.rectangle(MID, TOP, RIGHT - MID, BOTTOM - TOP)
    cr.fill()

    centers = [(LEFT + MID) / 2, (MID + RIGHT) / 2]
    reading_baseline = 832
    laid = []
    set_font(cr, SERIF, 44, bold=True)
    for (syllable, batchim, reading, mark), cx in zip(COLS, centers):
        head, marked = reading[:mark], reading[mark:]
        hb = cr.text_extents(head)
        mb = cr.text_extents(marked)
        gap = 10
        total = hb.x_advance + gap + mb.x_advance
        start = cx - total / 2
        laid.append((syllable, batchim, cx, start, head, marked, hb, mb))
    pad_x, pad_y = 7, 5
    box_top = min(reading_baseline + mb.y_bearing for *_, mb in laid) - pad_y
    box_height = max(reading_baseline + mb.y_bearing + mb.height for *_, mb in laid) + pad_y - box_top
    for syllable, batchim, cx, start, head, marked, hb, mb in laid:
        center(cr, cx, 500, syllable, SERIF_CJK, 188, NAVY)
        center(cr, cx, 700, batchim, SERIF_CJK, 84, NAVY)
        set_font(cr, SERIF, 44, bold=True)
        cr.set_source_rgb(*NAVY)
        cr.rectangle(
            start + hb.x_advance + 10 + mb.x_bearing - pad_x,
            box_top,
            mb.width + pad_x * 2,
            box_height,
        )
        cr.fill()
        cr.set_source_rgb(*NAVY)
        cr.move_to(start, reading_baseline)
        cr.show_text(head)
        cr.set_source_rgb(*CREAM)
        cr.move_to(start + hb.x_advance + 10, reading_baseline)
        cr.show_text(marked)

    cr.set_source_rgb(*NAVY)
    cr.set_line_width(2)
    cr.rectangle(LEFT, TOP, RIGHT - LEFT, BOTTOM - TOP)
    cr.stroke()
    cr.move_to(MID, TOP)
    cr.line_to(MID, BOTTOM)
    cr.stroke()

    center(cr, 540, 992, "© 수영 책방 Swimming Bookstore", SERIF_CJK, 20, TAUPE)
    surf.write_to_png(str(OUT))


def esc(text: str) -> str:
    return text.replace("&", "&amp;").replace("<", "&lt;")


def svg_runs(cr: cairo.Context, baseline: float, runs: list[tuple[str, str, float, bool]], fill: str) -> str:
    """One SVG text line. x on the first tspan is the centered start."""
    widths = [advance(cr, text, family, size, bold) for text, family, size, bold in runs]
    cursor = 540 - sum(widths) / 2
    parts = [f'  <text y="{baseline}" fill="{fill}" font-weight="700">']
    for i, ((text, family, size, bold), width) in enumerate(zip(runs, widths)):
        weight = ' font-weight="700"' if bold else ' font-weight="400"'
        x_attr = f' x="{cursor:.1f}"' if i == 0 else ""
        face = "Liberation Serif, serif" if family == SERIF else "Noto Serif CJK KR, serif"
        parts.append(
            f'    <tspan{x_attr} font-family="{face}" font-size="{size:.0f}"{weight}>{esc(text)}</tspan>'
        )
        cursor += width
    parts.append("  </text>")
    return "\n".join(parts)


def render_svg() -> None:
    """Hand SVG of the same plate. Cairo SVG text does not embed CJK reliably."""
    cr = cairo.Context(cairo.ImageSurface(cairo.FORMAT_ARGB32, W, H))
    left_cx = (LEFT + MID) / 2
    right_cx = (MID + RIGHT) / 2
    centers = [left_cx, right_cx]
    reading_baseline = 832
    pad_x, pad_y = 7, 5
    set_font(cr, SERIF, 44, bold=True)
    marked_ext = [cr.text_extents(reading[mark:]) for _s, _b, reading, mark in COLS]
    box_top = min(reading_baseline + mb.y_bearing for mb in marked_ext) - pad_y
    box_height = max(reading_baseline + mb.y_bearing + mb.height for mb in marked_ext) + pad_y - box_top
    marks = []
    for (_syllable, _batchim, reading, mark), cx, mb in zip(COLS, centers, marked_ext):
        head, marked = reading[:mark], reading[mark:]
        hb = cr.text_extents(head)
        gap = 10
        total = hb.x_advance + gap + mb.x_advance
        start = cx - total / 2
        marks.append(
            (
                start + hb.x_advance + gap + mb.x_bearing - pad_x,
                box_top,
                mb.width + pad_x * 2,
                box_height,
                start,
                start + hb.x_advance + gap,
                head,
                marked,
            )
        )

    lines = [
        '<svg xmlns="http://www.w3.org/2000/svg" width="1080" height="1080" viewBox="0 0 1080 1080" role="img" aria-label="Patchim: can you tell ㄱ from ㄴ">',
        f'  <rect width="1080" height="1080" fill="{CREAM_HEX}"/>',
        "",
        f'  <text x="540" y="90" text-anchor="middle" font-family="Lato, Outfit, ui-sans-serif, system-ui, sans-serif" font-size="20" font-weight="700" letter-spacing="7.2" fill="{TAUPE_HEX}">KOREAN ESSENTIAL</text>',
        svg_runs(cr, 176, TITLE, NAVY_HEX),
        svg_runs(cr, 236, EXAMPLE, TAUPE_HEX).replace(' font-weight="700"', "", 1),
        "",
        f'  <rect x="{LEFT}" y="{TOP}" width="{RIGHT - LEFT}" height="{BOTTOM - TOP}" fill="{CREAM_HEX}" stroke="{NAVY_HEX}" stroke-width="2"/>',
        f'  <rect x="{MID}" y="{TOP}" width="{RIGHT - MID}" height="{BOTTOM - TOP}" fill="{NAVY_HEX}" fill-opacity="0.045"/>',
        f'  <line x1="{MID}" y1="{TOP}" x2="{MID}" y2="{BOTTOM}" stroke="{NAVY_HEX}" stroke-width="2"/>',
        "",
        f'  <text x="{left_cx}" y="500" text-anchor="middle" dominant-baseline="central" font-family="Noto Serif CJK KR, serif" font-size="188" fill="{NAVY_HEX}">악</text>',
        f'  <text x="{right_cx}" y="500" text-anchor="middle" dominant-baseline="central" font-family="Noto Serif CJK KR, serif" font-size="188" fill="{NAVY_HEX}">안</text>',
        f'  <text x="{left_cx}" y="700" text-anchor="middle" dominant-baseline="central" font-family="Noto Serif CJK KR, serif" font-size="84" fill="{NAVY_HEX}">ㄱ</text>',
        f'  <text x="{right_cx}" y="700" text-anchor="middle" dominant-baseline="central" font-family="Noto Serif CJK KR, serif" font-size="84" fill="{NAVY_HEX}">ㄴ</text>',
        "",
    ]
    for rx, ry, rw, rh, _hx, _mx, _head, _marked in marks:
        lines.append(f'  <rect x="{rx:.1f}" y="{ry:.1f}" width="{rw:.1f}" height="{rh:.1f}" fill="{NAVY_HEX}"/>')
    for _rx, _ry, _rw, _rh, hx, mx, head, marked in marks:
        lines.append(
            f'  <text x="{hx:.1f}" y="{reading_baseline}" font-family="Liberation Serif, serif" font-size="44" font-weight="700" fill="{NAVY_HEX}">{esc(head)}</text>'
        )
        lines.append(
            f'  <text x="{mx:.1f}" y="{reading_baseline}" font-family="Liberation Serif, serif" font-size="44" font-weight="700" fill="{CREAM_HEX}">{esc(marked)}</text>'
        )
    lines += [
        "",
        f'  <text x="540" y="998" text-anchor="middle" font-family="Noto Serif CJK KR, Liberation Serif, serif" font-size="20" fill="{TAUPE_HEX}">{esc("© 수영 책방 Swimming Bookstore")}</text>',
        "</svg>",
        "",
    ]
    SVG.write_text("\n".join(lines))


def main() -> None:
    OUT.parent.mkdir(parents=True, exist_ok=True)
    render_png()
    render_svg()
    print(OUT)
    print(SVG)


if __name__ == "__main__":
    main()
