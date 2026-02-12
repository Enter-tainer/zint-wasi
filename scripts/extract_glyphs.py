#!/usr/bin/env python3
"""Extract glyph outlines from zint's embedded TTF fonts and generate typst data file.

Usage:
    PYTHONPATH=/tmp/fonttools-lib python3 extract_glyphs.py

Reads:
    zint-wasm-sys/zint/backend/fonts/normal_ttf.h  (Arimo)
    zint-wasm-sys/zint/backend/fonts/upcean_ttf.h  (OCRB)

Outputs:
    typst-package/glyphs.typ
"""

import io
import re
import sys

sys.path.insert(0, "/tmp/fonttools-lib")

from fontTools.ttLib import TTFont
from fontTools.pens.recordingPen import RecordingPen


ZINT_ROOT = "/home/ubuntu/freeman/data/threads/zint-wasi-pr-24-push/zint-wasi"
NORMAL_TTF_H = f"{ZINT_ROOT}/zint-wasm-sys/zint/backend/fonts/normal_ttf.h"
UPCEAN_TTF_H = f"{ZINT_ROOT}/zint-wasm-sys/zint/backend/fonts/upcean_ttf.h"

# Characters needed for barcode HRT
CHARSET = (
    "0123456789"
    "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
    "abcdefghijklmnopqrstuvwxyz"
    " -+$/.%*:<>()[]{}!@#&=_,;?^"
)


def parse_c_hex_array(filepath: str) -> bytes:
    """Parse a C header file containing a static hex byte array into bytes."""
    with open(filepath, "r") as f:
        content = f.read()

    # Find all 0xNN hex values
    hex_values = re.findall(r"0x([0-9a-fA-F]{2})", content)
    return bytes(int(h, 16) for h in hex_values)


def recording_to_commands(recording: RecordingPen) -> list:
    """Convert RecordingPen operations to our tuple format.

    RecordingPen records:
    - ('moveTo', ((x, y),))
    - ('lineTo', ((x, y),))
    - ('curveTo', ((cx1, cy1), (cx2, cy2), (x, y)))
    - ('qCurveTo', ((cx, cy), (x, y)))
    - ('closePath', ())
    - ('endPath', ())
    """
    commands = []
    for op, args in recording.value:
        if op == "moveTo":
            x, y = args[0]
            commands.append(("M", round(x), round(y)))
        elif op == "lineTo":
            x, y = args[0]
            commands.append(("L", round(x), round(y)))
        elif op == "curveTo":
            (cx1, cy1), (cx2, cy2), (x, y) = args
            commands.append((
                "C",
                round(cx1), round(cy1),
                round(cx2), round(cy2),
                round(x), round(y),
            ))
        elif op == "qCurveTo":
            # TrueType qCurveTo can have multiple off-curve points.
            # All but the last point are off-curve; implicit on-curve points
            # exist at midpoints between consecutive off-curve points.
            if len(args) == 2:
                # Simple case: one control point + endpoint
                (cx, cy), (x, y) = args
                commands.append(("Q", round(cx), round(cy), round(x), round(y)))
            else:
                # Multiple off-curve points: decompose into individual Q segments
                off_curves = args[:-1]
                end_pt = args[-1]
                for i, (cx, cy) in enumerate(off_curves):
                    if i < len(off_curves) - 1:
                        # Implicit on-curve point at midpoint of two off-curves
                        nx, ny = off_curves[i + 1]
                        ox = (cx + nx) / 2
                        oy = (cy + ny) / 2
                        commands.append(("Q", round(cx), round(cy), round(ox), round(oy)))
                    else:
                        # Last off-curve to actual endpoint
                        x, y = end_pt
                        commands.append(("Q", round(cx), round(cy), round(x), round(y)))
        elif op == "closePath":
            commands.append(("Z",))
        elif op == "endPath":
            pass  # ignore open paths
    return commands


def extract_font_data(ttf_bytes: bytes, charset: str) -> dict:
    """Extract glyph paths and metrics from a TTF font."""
    font = TTFont(io.BytesIO(ttf_bytes))

    # Get font metrics
    units_per_em = font["head"].unitsPerEm
    ascent = font["OS/2"].sTypoAscender
    descent = font["OS/2"].sTypoDescender  # negative value

    glyph_set = font.getGlyphSet()
    cmap = font.getBestCmap()

    glyphs = {}
    for char in charset:
        code_point = ord(char)
        glyph_name = cmap.get(code_point)
        if glyph_name is None:
            continue

        # Get advance width
        glyph_obj = glyph_set[glyph_name]
        advance = glyph_obj.width

        # Get path commands via RecordingPen
        pen = RecordingPen()
        glyph_obj.draw(pen)
        commands = recording_to_commands(pen)

        if not commands:
            # Space or empty glyph — store with empty path
            glyphs[char] = {"advance": advance, "path": []}
        else:
            glyphs[char] = {"advance": advance, "path": commands}

    font.close()
    return {
        "units_per_em": units_per_em,
        "ascent": ascent,
        "descent": descent,
        "glyphs": glyphs,
    }


def format_command_tuple(cmd: tuple) -> str:
    """Format a single command tuple as typst code."""
    # e.g. ("M", 531, 1409) -> '("M", 531, 1409)'
    parts = []
    for v in cmd:
        if isinstance(v, str):
            parts.append(f'"{v}"')
        else:
            parts.append(str(v))
    return "(" + ", ".join(parts) + ")"


def format_typst_font(name: str, data: dict) -> str:
    """Format font data as typst code."""
    lines = []
    lines.append(f"#let _{name} = (")
    lines.append(f"  units-per-em: {data['units_per_em']},")
    lines.append(f"  ascent: {data['ascent']},")
    lines.append(f"  descent: {data['descent']},")
    lines.append("  glyphs: (")

    for char, glyph in sorted(data["glyphs"].items(), key=lambda x: ord(x[0])):
        # Use the char as key, escape special chars for typst
        if char == '"':
            key_repr = '"\\\""'
        elif char == "\\":
            key_repr = '"\\\\"'
        else:
            key_repr = f'"{char}"'

        path = glyph["path"]
        if not path:
            path_repr = "()"
        else:
            cmd_strs = [format_command_tuple(cmd) for cmd in path]
            path_repr = "(" + ", ".join(cmd_strs) + ")"

        lines.append(
            f"    {key_repr}: (advance: {glyph['advance']}, path: {path_repr}),"
        )

    lines.append("  ),")
    lines.append(")")
    return "\n".join(lines)


def main():
    print("Extracting Arimo (normal) font...")
    normal_bytes = parse_c_hex_array(NORMAL_TTF_H)
    print(f"  TTF size: {len(normal_bytes)} bytes")
    arimo_data = extract_font_data(normal_bytes, CHARSET)
    print(f"  Extracted {len(arimo_data['glyphs'])} glyphs")
    print(
        f"  Metrics: unitsPerEm={arimo_data['units_per_em']}, "
        f"ascent={arimo_data['ascent']}, descent={arimo_data['descent']}"
    )

    print("Extracting OCRB (UPC/EAN) font...")
    upcean_bytes = parse_c_hex_array(UPCEAN_TTF_H)
    print(f"  TTF size: {len(upcean_bytes)} bytes")
    ocrb_data = extract_font_data(upcean_bytes, CHARSET)
    print(f"  Extracted {len(ocrb_data['glyphs'])} glyphs")
    print(
        f"  Metrics: unitsPerEm={ocrb_data['units_per_em']}, "
        f"ascent={ocrb_data['ascent']}, descent={ocrb_data['descent']}"
    )

    # Generate typst file
    output = []
    output.append(
        "// Auto-generated glyph data from Arimo and OCRB fonts."
    )
    output.append(
        "// Do not edit manually. Regenerate with: python3 scripts/extract_glyphs.py"
    )
    output.append("")
    output.append(format_typst_font("arimo", arimo_data))
    output.append("")
    output.append(format_typst_font("ocrb", ocrb_data))
    output.append("")

    output_content = "\n".join(output)

    # Write to workspace
    output_path = "/home/ubuntu/freeman/data/threads/zint-wasi-pr-24-push/data/workspaces/glyph-native/typst-package/glyphs.typ"
    with open(output_path, "w") as f:
        f.write(output_content)
    print(f"\nWritten to {output_path}")
    print(f"File size: {len(output_content)} bytes")


if __name__ == "__main__":
    main()
