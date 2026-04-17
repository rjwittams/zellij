#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# ///

import argparse
import base64
import os
import re
import select
import shutil
import sys
import termios
import time
import tty
from dataclasses import dataclass
from pathlib import Path

ESC = "\x1b"
CSI = f"{ESC}["
OSC = f"{ESC}]"
APC_END = f"{ESC}\\"
PLACEHOLDER = "\U0010EEEE"
DEFAULT_PNG = Path(__file__).resolve().parent.parent / "assets" / "logo.png"

DIACRITICS = [
    '\u0305', '\u030D', '\u030E', '\u0310', '\u0312', '\u033D', '\u033E', '\u033F',
    '\u0346', '\u034A', '\u034B', '\u034C', '\u0350', '\u0351', '\u0352', '\u0357',
    '\u035B', '\u0363', '\u0364', '\u0365', '\u0366', '\u0367', '\u0368', '\u0369',
    '\u036A', '\u036B', '\u036C', '\u036D', '\u036E', '\u036F', '\u0483', '\u0484',
    '\u0485', '\u0486', '\u0487', '\u0592', '\u0593', '\u0594', '\u0595', '\u0597',
    '\u0598', '\u0599', '\u059C', '\u059D', '\u059E', '\u059F', '\u05A0', '\u05A1',
    '\u05A8', '\u05A9', '\u05AB', '\u05AC', '\u05AF', '\u05C4', '\u0610', '\u0611',
    '\u0612', '\u0613', '\u0614', '\u0615', '\u0616', '\u0617', '\u0657', '\u0658',
    '\u0659', '\u065A', '\u065B', '\u065D', '\u065E', '\u06D6', '\u06D7', '\u06D8',
    '\u06D9', '\u06DA', '\u06DB', '\u06DC', '\u06DF', '\u06E0', '\u06E1', '\u06E2',
    '\u06E4', '\u06E7', '\u06E8', '\u06EB', '\u06EC', '\u0730', '\u0732', '\u0733',
    '\u0735', '\u0736', '\u073A', '\u073D', '\u073F', '\u0740', '\u0741', '\u0743',
    '\u0745', '\u0747', '\u0749', '\u074A', '\u07EB', '\u07EC', '\u07ED', '\u07EE',
    '\u07EF', '\u07F0', '\u07F1', '\u07F3', '\u0816', '\u0817', '\u0818', '\u0819',
    '\u081B', '\u081C', '\u081D', '\u081E', '\u081F', '\u0820', '\u0821', '\u0822',
    '\u0823', '\u0825', '\u0826', '\u0827', '\u0829', '\u082A', '\u082B', '\u082C',
    '\u082D', '\u0951', '\u0953', '\u0954', '\u0F82', '\u0F83', '\u0F86', '\u0F87',
    '\u135D', '\u135E', '\u135F', '\u17DD', '\u193A', '\u1A17', '\u1A75', '\u1A76',
    '\u1A77', '\u1A78', '\u1A79', '\u1A7A', '\u1A7B', '\u1A7C', '\u1B6B', '\u1B6D',
    '\u1B6E', '\u1B6F', '\u1B70', '\u1B71', '\u1B72', '\u1B73', '\u1CD0', '\u1CD1',
    '\u1CD2', '\u1CDA', '\u1CDB', '\u1CE0', '\u1DC0', '\u1DC1', '\u1DC3', '\u1DC4',
    '\u1DC5', '\u1DC6', '\u1DC7', '\u1DC8', '\u1DC9', '\u1DCB', '\u1DCC', '\u1DD1',
    '\u1DD2', '\u1DD3', '\u1DD4', '\u1DD5', '\u1DD6', '\u1DD7', '\u1DD8', '\u1DD9',
    '\u1DDA', '\u1DDB', '\u1DDC', '\u1DDD', '\u1DDE', '\u1DDF', '\u1DE0', '\u1DE1',
    '\u1DE2', '\u1DE3', '\u1DE4', '\u1DE5', '\u1DE6', '\u1DFE', '\u20D0', '\u20D1',
    '\u20D4', '\u20D5', '\u20D6', '\u20D7', '\u20DB', '\u20DC', '\u20E1', '\u20E7',
    '\u20E9', '\u20F0', '\u2CEF', '\u2CF0', '\u2CF1', '\u2DE0', '\u2DE1', '\u2DE2',
    '\u2DE3', '\u2DE4', '\u2DE5', '\u2DE6', '\u2DE7', '\u2DE8', '\u2DE9', '\u2DEA',
    '\u2DEB', '\u2DEC', '\u2DED', '\u2DEE', '\u2DEF', '\u2DF0', '\u2DF1', '\u2DF2',
    '\u2DF3', '\u2DF4', '\u2DF5', '\u2DF6', '\u2DF7', '\u2DF8', '\u2DF9', '\u2DFA',
    '\u2DFB', '\u2DFC', '\u2DFD', '\u2DFE', '\u2DFF', '\uA66F', '\uA67C', '\uA67D',
    '\uA6F0', '\uA6F1', '\uA8E0', '\uA8E1', '\uA8E2', '\uA8E3', '\uA8E4', '\uA8E5',
    '\uA8E6', '\uA8E7', '\uA8E8', '\uA8E9', '\uA8EA', '\uA8EB', '\uA8EC', '\uA8ED',
    '\uA8EE', '\uA8EF', '\uA8F0', '\uA8F1', '\uAAB0', '\uAAB2', '\uAAB3', '\uAAB7',
    '\uAAB8', '\uAABE', '\uAABF', '\uAAC1', '\uFE20', '\uFE21', '\uFE22', '\uFE23',
    '\uFE24', '\uFE25', '\uFE26', '\U00010A0F', '\U00010A38', '\U0001D185', '\U0001D186',
    '\U0001D187', '\U0001D188', '\U0001D189', '\U0001D1AA', '\U0001D1AB', '\U0001D1AC',
    '\U0001D1AD', '\U0001D242', '\U0001D243', '\U0001D244',
]


@dataclass
class Caps:
    kitty_basic_query: bool = False
    sixel: bool = False
    cell_size: tuple[int, int] | None = None
    primary_da: str | None = None
    secondary_da: str | None = None
    xtversion: str | None = None
    dsr_ok: bool = False
    query_raw: str = ""


def out(s: str):
    sys.stdout.write(s)


def flush():
    sys.stdout.flush()


def apc(control: str, payload=b""):
    if isinstance(payload, bytes):
        payload = base64.b64encode(payload).decode()
    out(f"{ESC}_G{control};{payload}{APC_END}")


def chunked_apc(prefix: str, payload: bytes, chunk=3072):
    b64 = base64.b64encode(payload).decode()
    parts = [b64[i:i + chunk] for i in range(0, len(b64), chunk)] or [""]
    for i, part in enumerate(parts):
        more = 1 if i < len(parts) - 1 else 0
        control = f"{prefix},m={more}" if i == 0 else f"m={more}"
        out(f"{ESC}_G{control};{part}{APC_END}")


def goto(x: int, y: int):
    out(f"{CSI}{y};{x}H")


def clear_screen():
    out(f"{CSI}2J{CSI}H")


def reset_attrs():
    out(f"{CSI}0m")


def delete_all():
    out(f"{ESC}_Gq=2,a=d,d=A{APC_END}")
    flush()


def png_data(path: Path) -> bytes:
    return path.read_bytes()


def rgb_gradient(w: int, h: int) -> bytes:
    b = bytearray()
    for y in range(h):
        for x in range(w):
            b.extend((x * 255 // max(1, w - 1), y * 255 // max(1, h - 1), 180))
    return bytes(b)


def rgba_gradient(w: int, h: int) -> bytes:
    b = bytearray()
    for y in range(h):
        for x in range(w):
            b.extend((x * 255 // max(1, w - 1), y * 255 // max(1, h - 1), 180, 255))
    return bytes(b)


def rgba_alpha_checker(w: int, h: int, tile: int = 8) -> bytes:
    b = bytearray()
    for y in range(h):
        for x in range(w):
            checker = ((x // tile) + (y // tile)) % 2
            alpha = 255 if checker == 0 else 0
            b.extend((255, 255, 255, alpha))
    return bytes(b)


def read_replies(timeout: float = 0.3) -> str:
    fd = sys.stdin.fileno()
    old = termios.tcgetattr(fd)
    chunks = []
    try:
        tty.setcbreak(fd)
        end = time.time() + timeout
        while time.time() < end:
            remain = max(0.0, end - time.time())
            r, _, _ = select.select([fd], [], [], remain)
            if not r:
                break
            data = os_read(fd, 4096)
            if not data:
                break
            chunks.append(data.decode(errors="replace"))
            end = time.time() + 0.05
    finally:
        termios.tcsetattr(fd, termios.TCSADRAIN, old)
    return "".join(chunks)


def os_read(fd: int, n: int) -> bytes:
    import os
    return os.read(fd, n)


def run_query(control: str, payload: bytes, timeout: float = 0.5) -> str:
    fd = sys.stdin.fileno()
    old = termios.tcgetattr(fd)
    chunks: list[str] = []
    try:
        tty.setcbreak(fd)
        chunked_apc(control, payload)
        flush()
        end = time.time() + timeout
        while time.time() < end:
            remain = max(0.0, end - time.time())
            r, _, _ = select.select([fd], [], [], remain)
            if not r:
                break
            data = os_read(fd, 4096)
            if not data:
                break
            chunks.append(data.decode(errors="replace"))
            end = time.time() + 0.05
    finally:
        termios.tcsetattr(fd, termios.TCSADRAIN, old)
    return "".join(chunks)


def probe_caps() -> Caps:
    caps = Caps()
    # kitty basic graphics query
    out(f"{ESC}_Gi=31,s=1,v=1,a=q,t=d,f=24;AAAA{APC_END}")
    # primary DA (includes sixel for many terminals)
    out(f"{CSI}0c")
    # secondary DA
    out(f"{CSI}>0c")
    # XT version string query (supported by some terminals / by zellij)
    out(f"{CSI}>q")
    # cell size in pixels
    out(f"{CSI}16t")
    # DSR end marker
    out(f"{CSI}5n")
    flush()
    raw = read_replies(0.8)
    caps.query_raw = raw
    caps.kitty_basic_query = "_Gi=31;OK" in raw
    caps.sixel = any(s in raw for s in ("?4;", "?4c", ";4;", ";4c"))
    caps.dsr_ok = "\x1b[0n" in raw
    m = re.search(r"\x1b\[6;(\d+);(\d+)t", raw)
    if m:
        caps.cell_size = (int(m.group(2)), int(m.group(1)))
    m = re.search(r"\x1b\[\?([^c]+)c", raw)
    if m:
        caps.primary_da = m.group(1)
    m = re.search(r"\x1b\[>([^c]+)c", raw)
    if m:
        caps.secondary_da = m.group(1)
    m = re.search(r"\x1bP>\|([^\x1b]+)\x1b\\", raw)
    if m:
        caps.xtversion = m.group(1)
    return caps


def draw_box(x: int, y: int, w: int, h: int, title: str):
    title_text = f"[{title}]"
    top = "+" + "-" * w + "+"
    if len(title_text) + 4 <= len(top):
        top = top[:2] + title_text + top[2 + len(title_text):]
    goto(x, y)
    out(top)
    for row in range(1, h + 1):
        goto(x, y + row)
        out("|" + " " * w + "|")
    goto(x, y + h + 1)
    out("+" + "-" * w + "+")
    flush()


def label(text: str, x: int, y: int):
    goto(x, y)
    reset_attrs()
    out(text)
    flush()


def transmit_png(path: Path, image_id: int):
    chunked_apc(f"q=2,a=t,f=100,i={image_id}", png_data(path))
    flush()


def display_png(
    image_id: int,
    cols=14,
    rows=7,
    x=3,
    y=5,
    placement_id: int | None = None,
    include_cols: bool = True,
    include_rows: bool = True,
):
    goto(x, y)
    control = f"q=2,a=p,C=1,i={image_id}"
    if placement_id is not None:
        control += f",p={placement_id}"
    if include_cols:
        control += f",c={cols}"
    if include_rows:
        control += f",r={rows}"
    apc(control, b"")
    flush()


def explicit_png(
    path: Path,
    image_id=100,
    cols=14,
    rows=7,
    x=3,
    y=5,
    include_cols: bool = True,
    include_rows: bool = True,
):
    goto(x, y)
    control = f"q=2,a=T,C=1,f=100,i={image_id}"
    if include_cols:
        control += f",c={cols}"
    if include_rows:
        control += f",r={rows}"
    chunked_apc(control, png_data(path))
    flush()


def explicit_rgb(image_id=101, w=56, h=28, cols=14, rows=7, x=3, y=5):
    goto(x, y)
    chunked_apc(f"q=2,a=T,C=1,f=24,s={w},v={h},i={image_id},c={cols},r={rows}", rgb_gradient(w, h))
    flush()


def explicit_rgba(image_id=102, w=56, h=28, cols=14, rows=7, x=27, y=5, alpha_demo: bool = False):
    goto(x, y)
    payload = rgba_alpha_checker(w, h) if alpha_demo else rgba_gradient(w, h)
    chunked_apc(f"q=2,a=T,C=1,f=32,s={w},v={h},i={image_id},c={cols},r={rows}", payload)
    flush()


def placeholder_rgba(image_id=103, w=56, h=56, cols=14, rows=7, x=3, y=16):
    chunked_apc(f"q=2,a=T,C=1,U=1,f=32,s={w},v={h},i={image_id},c={cols},r={rows}", rgba_gradient(w, h))
    low = image_id & 0xFFFFFF
    r = (low >> 16) & 0xFF
    g = (low >> 8) & 0xFF
    b = low & 0xFF
    high = (image_id >> 24) & 0xFF
    hi = DIACRITICS[high] if high else ""
    for row in range(rows):
        goto(x, y + row)
        out(f"{CSI}38;2;{r};{g};{b}m")
        for col in range(cols):
            out(f"{PLACEHOLDER}{DIACRITICS[row]}{DIACRITICS[col]}{hi}")
        out(f"{CSI}39m")
    flush()


def placeholder_rgb(image_id=152, w=40, h=40, cols=10, rows=4, x=20, y=16):
    chunked_apc(f"q=2,a=T,C=1,U=1,f=24,s={w},v={h},i={image_id},c={cols},r={rows}", rgb_gradient(w, h))
    low = image_id & 0xFFFFFF
    r = (low >> 16) & 0xFF
    g = (low >> 8) & 0xFF
    b = low & 0xFF
    high = (image_id >> 24) & 0xFF
    hi = DIACRITICS[high] if high else ""
    for row in range(rows):
        goto(x, y + row)
        out(f"{CSI}38;2;{r};{g};{b}m")
        for col in range(cols):
            out(f"{PLACEHOLDER}{DIACRITICS[row]}{DIACRITICS[col]}{hi}")
        out(f"{CSI}39m")
    flush()


def divider(title: str):
    clear_screen()
    reset_attrs()
    goto(1, 1)
    out(title)
    flush()


def wait_for_enter(prompt: str = "Press Enter for next stage..."):
    reset_attrs()
    if prompt:
        goto(1, 29)
        out(prompt)
    flush()
    try:
        input()
    except EOFError:
        time.sleep(1.0)


def prompt_line(message: str):
    out(f"{CSI}38;5;45m[prompt]{CSI}39m {message}\n")
    flush()


def terminal_size() -> tuple[int, int]:
    size = shutil.get_terminal_size(fallback=(80, 24))
    return size.columns, size.lines


def wait_for_resize(description: str, predicate) -> tuple[int, int] | None:
    prompt_line(f"{description} Type s + Enter to skip.")
    while True:
        cols, rows = terminal_size()
        if predicate(cols, rows):
            prompt_line(f"Reached checkpoint at {cols}x{rows}.")
            return cols, rows
        r, _, _ = select.select([sys.stdin], [], [], 0.2)
        if r:
            try:
                line = sys.stdin.readline().strip().lower()
            except EOFError:
                line = ""
            if line == "s":
                prompt_line("Resize checkpoint skipped.")
                return None


def read_key() -> str:
    fd = sys.stdin.fileno()
    old = termios.tcgetattr(fd)
    try:
        tty.setcbreak(fd)
        while True:
            r, _, _ = select.select([fd], [], [], None)
            if not r:
                continue
            data = os_read(fd, 8)
            if not data:
                return ""
            s = data.decode(errors="ignore")
            if s.startswith("\x1b[A"):
                return "up"
            if s.startswith("\x1b[B"):
                return "down"
            if s:
                return s[0]
    finally:
        termios.tcsetattr(fd, termios.TCSADRAIN, old)


def stage_detect() -> Caps:
    divider("Stage 1/12: capability and environment probe")
    label("Expect: a capability summary only. Later stages use this to set expectations and skips.", 1, 3)
    caps = probe_caps()
    env_rows = [
        ("TERM", os.environ.get("TERM")),
        ("TERM_PROGRAM", os.environ.get("TERM_PROGRAM")),
        ("ZELLIJ_SESSION_NAME", os.environ.get("ZELLIJ_SESSION_NAME")),
        ("TMUX", os.environ.get("TMUX")),
    ]
    y = 5
    label("Environment:", 1, y)
    y += 1
    for key, value in env_rows:
        label(f"  {key:<18} {value!r}", 1, y)
        y += 1
    y += 1
    label("Replies / coarse capabilities:", 1, y)
    y += 1
    label(f"  kitty basic query   {'yes' if caps.kitty_basic_query else 'no'}", 1, y)
    y += 1
    label(f"  sixel via DA        {'yes' if caps.sixel else 'no'}", 1, y)
    y += 1
    label(f"  cell size           {caps.cell_size!r}", 1, y)
    y += 1
    label(f"  primary DA          {caps.primary_da!r}", 1, y)
    y += 1
    label(f"  secondary DA        {caps.secondary_da!r}", 1, y)
    y += 1
    label(f"  XT version          {caps.xtversion!r}", 1, y)
    y += 1
    label(f"  DSR status ok       {'yes' if caps.dsr_ok else 'no'}", 1, y)
    y += 2
    label("Notes:", 1, y)
    y += 1
    label("  - kitty basic query is only coarse protocol detection, not the full feature matrix.", 1, y)
    y += 1
    label("  - later stages explicitly test: PNG, chunked RGB, chunked RGBA, placeholders, aspect rules, multi-placement delete, erase interactions, and delete-all-visible.", 1, y)
    return caps


def stage_query_semantics(caps: Caps):
    divider("Stage 2/12: kitty query / response semantics")
    if not caps.kitty_basic_query:
        label("SKIP: kitty graphics query did not succeed.", 1, 4)
        return
    label("Expect: query results below showing OK/error behavior, echoed ids, and quiet-mode suppression.", 1, 3)
    label("The query-only box should remain empty: a=q should not store an image that can later be displayed.", 1, 4)

    valid_payload = rgb_gradient(1, 1)

    success_reply = run_query("q=0,a=q,t=d,f=24,s=1,v=1,i=41", valid_payload)
    failure_reply = run_query("q=0,a=q,t=d,f=24,s=1,v=1,i=42,I=1", valid_payload)
    quiet_success_reply = run_query("q=1,a=q,t=d,f=24,s=1,v=1,i=43", valid_payload)
    quiet_failure_reply = run_query("q=2,a=q,t=d,f=24,s=1,v=1,i=44,I=1", valid_payload)

    success_ok = "OK" in success_reply and "i=41" in success_reply
    failure_has_error = bool(failure_reply) and "OK" not in failure_reply
    quiet_success_suppressed = quiet_success_reply == ""
    quiet_failure_suppressed = quiet_failure_reply == ""

    def verdict(ok: bool) -> str:
        return "PASS" if ok else "FAIL"

    y = 6
    label(f"success reply        {success_reply!r}", 1, y)
    label(f"{verdict(success_ok)}: expect echoed i=41 and OK", 58, y)
    y += 2
    label(f"failure reply        {failure_reply!r}", 1, y)
    label(f"{verdict(failure_has_error)}: expect non-OK error reply for invalid i+I query", 58, y)
    y += 2
    label(f"quiet success q=1    {quiet_success_reply!r}", 1, y)
    label(f"{verdict(quiet_success_suppressed)}: expect success reply suppression", 58, y)
    y += 2
    label(f"quiet failure q=2    {quiet_failure_reply!r}", 1, y)
    label(f"{verdict(quiet_failure_suppressed)}: expect failure reply suppression", 58, y)
    y += 2
    draw_box(2, y, 12, 4, "query only")
    display_png(41, cols=12, rows=4, x=3, y=y + 1, placement_id=1)
    y += 6
    label("Visual check: the query-only box above should stay empty if a=q is non-storing.", 1, y)
    y += 2
    label(
        f"Summary: success={verdict(success_ok)}, invalid={verdict(failure_has_error)}, q=1={verdict(quiet_success_suppressed)}, q=2={verdict(quiet_failure_suppressed)}",
        1,
        y,
    )


def stage_explicit_png(path: Path, caps: Caps):
    divider("Stage 3/12: explicit PNG placement")
    if not caps.kitty_basic_query:
        label("SKIP: kitty graphics query did not succeed.", 1, 4)
        return
    label("Expect: the Zellij logo fills most of the boxed area below.", 1, 3)
    draw_box(2, 4, 14, 7, "png")
    explicit_png(path)


def stage_explicit_rgb(caps: Caps):
    divider("Stage 4/12: explicit RGB chunked placement")
    if not caps.kitty_basic_query:
        label("SKIP: kitty graphics query did not succeed.", 1, 4)
        return
    label("Expect: a full gradient block fills most of the boxed area below using f=24 RGB payloads.", 1, 3)
    draw_box(2, 4, 14, 7, "rgb")
    explicit_rgb()


def stage_explicit_rgba(caps: Caps):
    divider("Stage 5/12: explicit RGBA chunked placement")
    if not caps.kitty_basic_query:
        label("SKIP: kitty graphics query did not succeed.", 1, 4)
        return
    label("Expect: the left box shows an RGB gradient background with an RGBA alpha-checker over it.", 1, 3)
    label("Transparent RGBA tiles should reveal the background; opaque tiles should appear as white squares.", 1, 4)
    label("The right box is a plain RGBA gradient control using the same f=32 path.", 1, 5)
    draw_box(2, 7, 14, 7, "rgba alpha")
    draw_box(26, 7, 14, 7, "rgba control")
    explicit_rgb(image_id=102, w=56, h=28, cols=14, rows=7, x=3, y=8)
    explicit_rgba(image_id=103, w=56, h=28, cols=14, rows=7, x=3, y=8, alpha_demo=True)
    explicit_rgba(image_id=104, w=56, h=28, cols=14, rows=7, x=27, y=8)


def stage_placeholder(caps: Caps):
    divider("Stage 6/12: Unicode placeholder placement")
    if not caps.kitty_basic_query:
        label("SKIP: kitty graphics query did not succeed.", 1, 4)
        return
    label("Expect: a roughly square gradient filling most of the boxed area below; raw placeholders should not remain visible.", 1, 3)
    draw_box(2, 15, 14, 7, "placeholder")
    placeholder_rgba()


def stage_aspect(path: Path, caps: Caps):
    divider("Stage 7/12: aspect comparison for explicit placement")
    if not caps.kitty_basic_query:
        label("SKIP: kitty graphics query did not succeed.", 1, 4)
        return
    label("Spec note: if both c and r are given, the image is scaled to fit that rectangle.", 1, 3)
    label("If only one of c or r is given, the other is computed to preserve aspect ratio.", 1, 4)
    label("Compare bounded fit (c+r) with c-only and r-only behavior.", 1, 5)
    draw_box(2, 7, 14, 7, "png c+r")
    draw_box(26, 7, 14, 7, "png c only")
    draw_box(50, 7, 14, 7, "png r only")
    explicit_png(path, image_id=110, cols=14, rows=7, x=3, y=8, include_cols=True, include_rows=True)
    explicit_png(path, image_id=111, cols=14, rows=7, x=27, y=8, include_cols=True, include_rows=False)
    explicit_png(path, image_id=112, cols=14, rows=7, x=51, y=8, include_cols=False, include_rows=True)


def stage_multi_delete(path: Path, caps: Caps):
    divider("Stage 8/12: multi-placement and targeted delete")
    if not caps.kitty_basic_query:
        label("SKIP: kitty graphics query did not succeed.", 1, 4)
        return
    label("Expect first: the same logo appears in both boxes below from one transmitted asset.", 1, 3)
    label("Then targeted delete removes only the right placement, leaving the left one visible.", 1, 4)
    draw_box(2, 7, 14, 7, "keep")
    draw_box(26, 7, 14, 7, "delete p=2")
    transmit_png(path, image_id=120)
    display_png(120, cols=14, rows=7, x=3, y=8, placement_id=1)
    display_png(120, cols=14, rows=7, x=27, y=8, placement_id=2)
    wait_for_enter("Press Enter to delete only the right placement...")
    out(f"{ESC}_Gq=2,a=d,d=i,i=120,p=2{APC_END}")
    flush()
    label("Targeted delete sent. Expect only the right placement to disappear.", 1, 17)


def stage_erase(caps: Caps):
    divider("Stage 9/12: erase interactions for placeholder flow")
    if not caps.kitty_basic_query:
        label("SKIP: kitty graphics query did not succeed.", 1, 4)
        return
    label("Expect first: four placeholder-backed squares appear below, one per test case.", 1, 3)
    label("Then top is overwritten with visible text, then EL 2, EL 1, and EL 0 are applied on separate terminal lines.", 1, 4)
    draw_box(2, 6, 8, 3, "overwrite")
    draw_box(2, 11, 8, 3, "EL 2")
    draw_box(2, 16, 8, 3, "EL 1")
    draw_box(2, 21, 8, 3, "EL 0")
    placeholder_rgba(image_id=130, w=32, h=24, cols=8, rows=3, x=3, y=7)
    placeholder_rgba(image_id=131, w=32, h=24, cols=8, rows=3, x=3, y=12)
    placeholder_rgba(image_id=132, w=32, h=24, cols=8, rows=3, x=3, y=17)
    placeholder_rgba(image_id=133, w=32, h=24, cols=8, rows=3, x=3, y=22)
    wait_for_enter("Press Enter to apply overwrite / erase-line operations...")
    goto(3, 7)
    out("OVERWRTE")
    goto(5, 13)
    out(f"{CSI}2K")
    goto(7, 18)
    out(f"{CSI}1K")
    goto(5, 23)
    out(f"{CSI}K")
    flush()
    label("Applied: overwrite on top row, EL 2 on second case, EL 1 on third case, EL 0 on bottom case.", 1, 26)


def stage_resize_reflow(path: Path, caps: Caps):
    divider("Stage 10/12: resize and reflow coherence")
    if not caps.kitty_basic_query:
        label("SKIP: kitty graphics query did not succeed.", 1, 4)
        return
    cols, rows = terminal_size()
    out("\n")
    prompt_line("Goal A: placeholder image should stay between BEFORE and AFTER text as width changes.")
    prompt_line("Goal B: explicit image clipping and post-resize scroll should remain coherent.")
    prompt_line(f"Baseline detected size: {cols}x{rows}")
    out("\n")

    out("BEFORE before before before before before before before before\n")
    out("BEFORE marker text above the placeholder image gap\n")
    flush()
    placeholder_rgba(image_id=140, w=40, h=40, cols=10, rows=4, x=1, y=9)
    goto(1, 14)
    out("AFTER after after after after after after after after\n")
    out("AFTER marker text below the placeholder image gap\n\n")
    out("EXPLICIT IMAGE BELOW\n")
    flush()
    explicit_png(path, image_id=141, cols=14, rows=6, x=1, y=18)
    goto(1, 25)
    for i in range(1, 7):
        out(f"PRE-SCROLL {i:02d}\n")
    out("\n")
    flush()

    narrow = wait_for_resize("Resize narrower until cols <= 60.", lambda c, r: c <= 60)
    if narrow is not None:
        prompt_line("Expect: placeholder image still separates BEFORE and AFTER; explicit image clips sanely.")
    else:
        prompt_line("Continuing without a narrow checkpoint.")

    wide = wait_for_resize("Resize wider again until cols >= 90.", lambda c, r: c >= 90)
    if wide is not None:
        prompt_line("Expect: no stale remnants or logical jumps remain after re-expansion.")
    else:
        prompt_line("Continuing without a wide checkpoint.")

    prompt_line("Emitting post-resize scroll markers.")
    for i in range(7, 23):
        out(f"POST-RESIZE SCROLL {i:02d}: image/text relationship should remain coherent after resize.\n")
    flush()
    prompt_line("Inspect post-resize scroll behavior, then press Enter for the final delete stage.")
    wait_for_enter("")


def stage_scroll_region(path: Path, caps: Caps):
    divider("Stage 11/12: scroll-region coherence")
    if not caps.kitty_basic_query:
        label("SKIP: kitty graphics query did not succeed.", 1, 4)
        return

    region_left = 2
    region_top = 7
    region_width = 40
    region_height = 14
    region_bottom = region_top + region_height - 1
    scroll_top = region_top + 1
    scroll_bottom = region_bottom - 1
    scroll_left = region_left + 1
    scroll_rows = scroll_bottom - scroll_top + 1

    label("HEADER above scroll region (should remain fixed)", 1, 3)
    label("FOOTER below scroll region (should remain fixed)", 1, 24)
    label("+-[scroll region]----------------+", region_left, region_top)
    label("+--------------------------------+", region_left, region_bottom)
    label("Use Down/Up or j/k to scroll one line at a time. q finishes this stage.", 1, 5)
    label("Only the band between the horizontal borders should move.", 1, 6)

    out(f"{CSI}{scroll_top};{scroll_bottom}r")

    # A fixed, bounded scene: short lines that fit the region, with two reserved image lanes.
    scene_lines = [
        "L01 top text",
        "L02 top text",
        "L03 before img A",
        "",
        "",
        "",
        "",
        "L04 between images",
        "L05 before img B",
        "",
        "",
        "",
        "",
        "L06 after img B",
        "L07 lower text",
        "L08 lower text",
        "L09 lower text",
        "L10 lower text",
    ]
    scene_top_index = 0

    def render_scene_window(start_index: int):
        for row_offset in range(scroll_rows):
            goto(scroll_left, scroll_top + row_offset)
            out(" " * region_width)
            goto(scroll_left, scroll_top + row_offset)
            line = scene_lines[start_index + row_offset]
            out(line[:region_width])
        flush()

    render_scene_window(scene_top_index)

    # Place two explicit images into reserved blank lanes of the fixed scene.
    explicit_png(path, image_id=150, cols=10, rows=4, x=5, y=11)
    placeholder_rgb(image_id=152, w=40, h=40, cols=10, rows=4, x=20, y=16)

    status_row = 23

    def set_status(message: str):
        goto(1, status_row)
        out(f"{CSI}2K")
        goto(1, status_row)
        out(f"{CSI}38;5;45m[status]{CSI}39m {message}")
        flush()

    set_status("Scroll to bottom, back to top, then down again. Left image is explicit; right image is placeholder-based.")

    max_top_index = max(0, len(scene_lines) - scroll_rows)
    while True:
        key = read_key().lower()
        if key == "q":
            break
        if key in {"j", "down"}:
            if scene_top_index >= max_top_index:
                set_status("Already at the bottom. Reverse upward or press q.")
                continue
            scene_top_index += 1
            goto(scroll_left, scroll_bottom)
            out(" " * region_width)
            goto(scroll_left, scroll_bottom)
            out(scene_lines[scene_top_index + scroll_rows - 1][:region_width])
            out("\n")
            flush()
            if scene_top_index == max_top_index:
                set_status("Reached the bottom. Now scroll upward back toward the top.")
        elif key in {"k", "up"}:
            if scene_top_index == 0:
                set_status("Already at the top. Scroll downward or press q.")
                continue
            scene_top_index -= 1
            out("\x1bM")
            goto(scroll_left, scroll_top)
            out(" " * region_width)
            goto(scroll_left, scroll_top)
            out(scene_lines[scene_top_index][:region_width])
            flush()
            if scene_top_index == 0:
                set_status("Returned to the top. Scroll downward again if you want a second pass.")
        else:
            set_status("Use Down/Up or j/k to scroll one line at a time; q finishes this stage.")

    out(f"{CSI}r")
    flush()
    set_status("Probe complete. Header/footer and border should have remained fixed.")


def stage_delete(caps: Caps):
    divider("Stage 12/12: delete all visible kitty images")
    label("Expect: all kitty images disappear after delete-all-visible.", 1, 3)
    if caps.kitty_basic_query:
        delete_all()
    label("Delete command sent.", 1, 5)


def all_stages(path: Path):
    caps = stage_detect()
    wait_for_enter()
    stage_query_semantics(caps)
    wait_for_enter()
    stage_explicit_png(path, caps)
    wait_for_enter()
    stage_explicit_rgb(caps)
    wait_for_enter()
    stage_explicit_rgba(caps)
    wait_for_enter()
    stage_placeholder(caps)
    wait_for_enter()
    stage_aspect(path, caps)
    wait_for_enter()
    stage_multi_delete(path, caps)
    wait_for_enter()
    stage_erase(caps)
    wait_for_enter()
    stage_resize_reflow(path, caps)
    wait_for_enter()
    stage_scroll_region(path, caps)
    wait_for_enter()
    stage_delete(caps)
    goto(1, 7)
    out("Done.\n")
    flush()


def main():
    p = argparse.ArgumentParser()
    p.add_argument("png", nargs="?", default=str(DEFAULT_PNG), help="png path (defaults to assets/logo.png)")
    p.add_argument("--mode", choices=["all", "detect", "query", "explicit-png", "explicit-rgb", "explicit-rgba", "placeholder", "aspect", "multi-delete", "erase", "resize", "scroll-region", "delete"], default="all")
    args = p.parse_args()

    path = Path(args.png)
    if args.mode in {"all", "explicit-png"} and not path.exists():
        raise SystemExit(f"png not found: {path}")

    if args.mode == "all":
        all_stages(path)
        return

    caps = stage_detect() if args.mode != "detect" else stage_detect()
    if args.mode == "detect":
        return
    wait_for_enter()
    if args.mode == "query":
        stage_query_semantics(caps)
    elif args.mode == "explicit-png":
        stage_explicit_png(path, caps)
    elif args.mode == "explicit-rgb":
        stage_explicit_rgb(caps)
    elif args.mode == "explicit-rgba":
        stage_explicit_rgba(caps)
    elif args.mode == "placeholder":
        stage_placeholder(caps)
    elif args.mode == "aspect":
        stage_aspect(path, caps)
    elif args.mode == "multi-delete":
        stage_multi_delete(path, caps)
    elif args.mode == "erase":
        stage_erase(caps)
    elif args.mode == "resize":
        stage_resize_reflow(path, caps)
    elif args.mode == "scroll-region":
        stage_scroll_region(path, caps)
    elif args.mode == "delete":
        stage_delete(caps)
    wait_for_enter("Press Enter to exit...")


if __name__ == "__main__":
    main()
