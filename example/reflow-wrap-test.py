#!/usr/bin/env python3
import argparse
import shutil
import sys

RESET = "\x1b[0m"
BOLD = "\x1b[1m"
PAIR_COLORS = [
    39,   # blue
    42,   # green
    172,  # orange
    135,  # purple
    203,  # red
    75,   # cyan-ish
    179,  # yellow-brown
    141,  # violet
]
FG_PREFIX = "\x1b[38;5;244m"


def visible_len(s: str) -> int:
    return len(s)


def fit_visible(prefix: str, body: str, width: int) -> str:
    prefix_len = visible_len(prefix)
    if prefix_len >= width:
        return prefix[:width]
    remaining = width - prefix_len
    if len(body) < remaining:
        body = body + (" " * (remaining - len(body)))
    else:
        body = body[:remaining]
    return prefix + body


def hundreds_pattern(width: int) -> str:
    return "".join(str((i // 100) % 10) for i in range(width))


def tens_pattern(width: int) -> str:
    return "".join(str((i // 10) % 10) for i in range(width))


def units_pattern(width: int) -> str:
    return "".join(str(i % 10) for i in range(width))


def pair_color(pair_index: int) -> str:
    return f"\x1b[38;5;{PAIR_COLORS[pair_index % len(PAIR_COLORS)]}m"


def build_row(pair_index: int, row_kind: str, width: int) -> str:
    prefix = f"{pair_index:03d}{row_kind}:"
    body_width = max(0, width - len(prefix))
    if row_kind == "H":
        body = hundreds_pattern(body_width)
    elif row_kind == "T":
        body = tens_pattern(body_width)
    else:
        body = units_pattern(body_width)
    color = pair_color(pair_index)
    visible = fit_visible(prefix, body, width)
    return f"{FG_PREFIX}{visible[:len(prefix)]}{RESET}{color}{visible[len(prefix):]}{RESET}"


def main() -> int:
    parser = argparse.ArgumentParser(description="Emit fixed-width colored line pairs for wrap/reflow testing.")
    parser.add_argument("--width", type=int, help="Visible line width. Defaults to current terminal width.")
    parser.add_argument("--height", type=int, help="Total visible lines to emit. Defaults to current terminal height.")
    parser.add_argument(
        "--pairs",
        type=int,
        help="Number of line triples to emit. Overrides --height/3 if provided.",
    )
    parser.add_argument(
        "--no-alt-screen",
        action="store_true",
        help="Do not clear the screen before printing.",
    )
    args = parser.parse_args()

    term_size = shutil.get_terminal_size((80, 24))
    width = args.width or term_size.columns
    height = args.height or term_size.lines
    pairs = args.pairs if args.pairs is not None else max(1, height // 3)

    if not args.no_alt_screen:
        sys.stdout.write("\x1b[2J\x1b[H")

    sys.stdout.write(f"{BOLD}reflow-wrap-test width={width} height={height} triples={pairs}{RESET}\n")

    emitted = 1
    for pair_index in range(pairs):
        if emitted >= height + 1:
            break
        sys.stdout.write(build_row(pair_index, "H", width) + "\n")
        emitted += 1
        if emitted >= height + 1:
            break
        sys.stdout.write(build_row(pair_index, "T", width) + "\n")
        emitted += 1
        if emitted >= height + 1:
            break
        sys.stdout.write(build_row(pair_index, "U", width) + "\n")
        emitted += 1

    sys.stdout.flush()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
