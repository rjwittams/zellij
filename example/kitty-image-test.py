#!/usr/bin/env python3
import argparse
import base64
import os
import sys

ESC = b"\x1b"
ST = b"\x1b\\"


def apc(params: str, payload: bytes | None = None) -> bytes:
    out = bytearray()
    out.extend(ESC)
    out.extend(b"_G")
    out.extend(params.encode("ascii"))
    if payload is not None:
        out.extend(b";")
        out.extend(payload)
    out.extend(ST)
    return bytes(out)


def write_stdout(data: bytes) -> None:
    sys.stdout.buffer.write(data)
    sys.stdout.buffer.flush()


def transmit_png(path: str, image_id: int, quiet: int) -> None:
    with open(path, "rb") as f:
        data = f.read()
    payload = base64.b64encode(data)
    params = f"a=T,f=100,i={image_id},q={quiet}"
    write_stdout(apc(params, payload))


HELP = """\
Minimal kitty graphics protocol PNG test.

This emits a single kitty graphics APC sequence:
  a=T   transmit and display immediately
  f=100 image format is PNG
  i=ID  explicit kitty image id
  q=N   response verbosity (debugging)

Payload:
  The PNG file bytes are base64-encoded and sent as the APC payload.

Example:
  python3 example/kitty-image-test.py ~/Downloads/recovered_logo.png

Useful notes:
  - This is intended as a minimal direct protocol test, not an app integration test.
  - It currently uses a single transmit+display command with inline base64 PNG data.
  - q=2 is useful while debugging because the terminal may emit success/error responses.
"""


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="Minimal kitty graphics protocol PNG test",
        epilog=HELP,
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument("image", help="path to a PNG file")
    parser.add_argument("--id", type=int, default=1, help="kitty image id for i=ID (default: 1)")
    parser.add_argument(
        "--quiet",
        type=int,
        default=2,
        choices=[0, 1, 2],
        help="kitty q=N response verbosity: 0=more, 1=errors only, 2=report success/failure",
    )
    args = parser.parse_args()

    if not os.path.exists(args.image):
        print(f"missing file: {args.image}", file=sys.stderr)
        sys.exit(1)

    transmit_png(args.image, args.id, args.quiet)
