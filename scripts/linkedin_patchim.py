#!/usr/bin/env python3
"""LinkedIn 1080×1080 plate: patchim ㄱ (기역) vs ㄴ (니은).

Same plate as swimming-bookstore/learn-japanese katakana ネ/レ: cream wash,
tracked kicker, serif title, navy table, bookstore footer.

Left column is 악, then ㄱ (기역). Right column is 안, then ㄴ (니은).
The reading marks the batchim sound. No example line.
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
# Latin and jamo share one size so the header sits on one line
TITLE_SIZE = 48
TITLE = [
    ("Patchim: can you tell ", SERIF, TITLE_SIZE, True),
    ("ㄱ", SERIF_CJK, TITLE_SIZE, True),
    (" from ", SERIF, TITLE_SIZE, True),
    ("ㄴ", SERIF_CJK, TITLE_SIZE, True),
]
# jamo and its name share one size so the row sits on one line
NAME_SIZE = 48
NAMES = [
    [
        ("ㄱ", SERIF_CJK, NAME_SIZE, True),
        (" (", SERIF, NAME_SIZE, True),
        ("기역", SERIF_CJK, NAME_SIZE, True),
        (")", SERIF, NAME_SIZE, True),
    ],
    [
        ("ㄴ", SERIF_CJK, NAME_SIZE, True),
        (" (", SERIF, NAME_SIZE, True),
        ("니은", SERIF_CJK, NAME_SIZE, True),
        (")", SERIF, NAME_SIZE, True),
    ],
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


def fit_jamo(cr: cairo.Context, text: str, family: str, size: float, bold: bool) -> tuple[float, float]:
    """Scale a lone jamo to the Latin cap height and drop it onto the baseline.

    Noto's ㄱ/ㄴ sit above the baseline and shorter than the caps, so a shared
    font size still looks high. Returns (size, baseline shift downward).
    """
    if text not in ("ㄱ", "ㄴ"):
        return size, 0.0
    set_font(cr, SERIF, size, bold=True)
    cap_h = cr.text_extents("H").height
    set_font(cr, family, size, bold=bold)
    ink_h = cr.text_extents(text).height
    if ink_h <= 0:
        return size, 0.0
    fitted = size * cap_h / ink_h
    set_font(cr, family, fitted, bold=bold)
    ext = cr.text_extents(text)
    return fitted, -(ext.y_bearing + ext.height)


def advance(cr: cairo.Context, text: str, family: str, size: float, bold: bool = False) -> float:
    set_font(cr, family, size, bold=bold)
    return cr.text_extents(text).x_advance


def center_runs(cr: cairo.Context, x: float, baseline: float, runs: list[tuple[str, str, float, bool]], fill, fit: bool = False) -> float:
    """Draw mixed-font runs on one baseline, centered on x. Returns the start x."""
    fitted = []
    for text, family, size, bold in runs:
        shift = 0.0
        if fit:
            size, shift = fit_jamo(cr, text, family, size, bold)
        fitted.append((text, family, size, bold, shift))
    widths = [advance(cr, text, family, size, bold) for text, family, size, bold, _shift in fitted]
    cursor = x - sum(widths) / 2
    cr.set_source_rgb(*fill)
    for (text, family, size, bold, shift), width in zip(fitted, widths):
        set_font(cr, family, size, bold=bold)
        cr.move_to(cursor, baseline + shift)
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

    center_runs(cr, 540, 176, TITLE, NAVY, fit=True)

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
        center_runs(cr, cx, 712, NAMES[0] if cx == centers[0] else NAMES[1], NAVY)
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


def svg_runs(cr: cairo.Context, baseline: float, runs: list[tuple[str, str, float, bool]], fill: str, center_x: float = 540, fit: bool = False) -> str:
    """One SVG text line. x on the first tspan is the centered start."""
    fitted = []
    for text, family, size, bold in runs:
        shift = 0.0
        if fit:
            size, shift = fit_jamo(cr, text, family, size, bold)
        fitted.append((text, family, size, bold, shift))
    widths = [advance(cr, text, family, size, bold) for text, family, size, bold, _shift in fitted]
    cursor = center_x - sum(widths) / 2
    parts = [f'  <text fill="{fill}" font-weight="700">']
    for i, ((text, family, size, bold, shift), width) in enumerate(zip(fitted, widths)):
        weight = ' font-weight="700"' if bold else ' font-weight="400"'
        x_attr = f' x="{cursor:.1f}"' if i == 0 else ""
        face = "Liberation Serif, serif" if family == SERIF else "Noto Serif CJK KR, serif"
        parts.append(
            f'    <tspan{x_attr} y="{baseline + shift:.1f}" font-family="{face}" font-size="{size:.1f}"{weight}>{esc(text)}</tspan>'
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
        svg_runs(cr, 176, TITLE, NAVY_HEX, fit=True),
        "",
        f'  <rect x="{LEFT}" y="{TOP}" width="{RIGHT - LEFT}" height="{BOTTOM - TOP}" fill="{CREAM_HEX}" stroke="{NAVY_HEX}" stroke-width="2"/>',
        f'  <rect x="{MID}" y="{TOP}" width="{RIGHT - MID}" height="{BOTTOM - TOP}" fill="{NAVY_HEX}" fill-opacity="0.045"/>',
        f'  <line x1="{MID}" y1="{TOP}" x2="{MID}" y2="{BOTTOM}" stroke="{NAVY_HEX}" stroke-width="2"/>',
        "",
        f'  <text x="{left_cx}" y="500" text-anchor="middle" dominant-baseline="central" font-family="Noto Serif CJK KR, serif" font-size="188" fill="{NAVY_HEX}">악</text>',
        f'  <text x="{right_cx}" y="500" text-anchor="middle" dominant-baseline="central" font-family="Noto Serif CJK KR, serif" font-size="188" fill="{NAVY_HEX}">안</text>',
        svg_runs(cr, 712, NAMES[0], NAVY_HEX, left_cx),
        svg_runs(cr, 712, NAMES[1], NAVY_HEX, right_cx),
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
