#!/usr/bin/env python3
"""Record a 9:16 YouTube Short of one lesson via ~/demo RecordSession."""

from __future__ import annotations

import argparse
import http.server
import os
import socket
import subprocess
import sys
import threading
import time
from functools import partial
from pathlib import Path

sys.path.insert(0, str(Path.home() / "demo"))
from record.session import RecordSession, ffmpeg_bin

ROOT = Path(__file__).resolve().parent.parent / "dist"
DOCS = Path(__file__).resolve().parent.parent / "docs"

# Width must be ≥640 (demo RecordSession). 640×960 fits a 1080p screen.
CAPTURE_W = 640
CAPTURE_H = 960
OUT_W = 1080
OUT_H = 1920
PAPER = "0xF6F1E8"
FPS = 30
SERIES = {
    "toesa": (1, list(range(1, 32))),
    "chamgyoyuk": (2, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]),
}


def free_port() -> int:
    s = socket.socket()
    s.bind(("127.0.0.1", 0))
    p = s.getsockname()[1]
    s.close()
    return p


def serve(port: int) -> http.server.HTTPServer:
    handler = partial(http.server.SimpleHTTPRequestHandler, directory=str(ROOT))
    httpd = http.server.HTTPServer(("127.0.0.1", port), handler)
    threading.Thread(target=httpd.serve_forever, daemon=True).start()
    return httpd


def slug_for(series: str, day: int) -> str:
    return f"{series}-day{day:02d}"


def scale_up(src: Path, dest: Path) -> None:
    tmp = dest.with_suffix(".tmp.mp4")
    subprocess.check_call(
        [
            ffmpeg_bin(),
            "-y",
            "-i",
            str(src),
            "-vf",
            (
                f"scale={OUT_W}:{OUT_H}:force_original_aspect_ratio=decrease:flags=lanczos,"
                f"pad={OUT_W}:{OUT_H}:(ow-iw)/2:(oh-ih)/2:color={PAPER}"
            ),
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
            "-crf",
            "18",
            "-preset",
            "fast",
            "-movflags",
            "+faststart",
            "-an",
            str(tmp),
        ],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    tmp.replace(dest)


def record_day(series_n: int, episode: int, series: str, port: int, cdp: int, hold: float) -> Path:
    slug = slug_for(series, episode)
    raw = DOCS / f"{slug}.raw.mp4"
    out = DOCS / f"{slug}.mp4"
    title = f"learn-korean-{slug}"
    url = f"http://127.0.0.1:{port}/index.html?record=1&autoplay=1#/{series_n}/{episode}"
    rec = RecordSession(
        url=url,
        out=raw,
        title=title,
        width=CAPTURE_W,
        height=CAPTURE_H,
        fps=FPS,
        max_sec=90,
        cdp_port=cdp,
        user_data_dir=Path("/tmp") / title,
    )
    rec.attach_chrome()
    try:
        rec.wait_js(
            "!!document.querySelector('.stage.record[data-ready=\"1\"]')",
            25,
            "ready",
        )
        rec.start_capture()
        if hold > 0:
            rec.hold(hold)
        else:
            rec.wait_js(
                "document.querySelector('.stage.record')?.getAttribute('data-done') === '1'",
                80,
                "done",
            )
            rec.hold(1.6)
    finally:
        rec.__exit__(None, None, None)
    if not raw.exists() or raw.stat().st_size < 1000:
        raise SystemExit(f"empty capture {raw}")
    scale_up(raw, out)
    raw.unlink(missing_ok=True)
    print(out, out.stat().st_size)
    return out


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--day", type=int, default=0, help="episode within --series")
    parser.add_argument("--all", action="store_true")
    parser.add_argument("--series", default="toesa", choices=sorted(SERIES))
    parser.add_argument("--hold", type=float, default=0, help="seconds; 0 = wait for done")
    parser.add_argument("--port", type=int, default=0)
    parser.add_argument("--cdp", type=int, default=9333)
    args = parser.parse_args()

    os.environ.setdefault("DISPLAY", ":0.0")
    if not ROOT.exists():
        raise SystemExit(f"missing {ROOT}; run trunk build --release")
    DOCS.mkdir(exist_ok=True)
    port = args.port or free_port()
    httpd = serve(port)
    if args.all:
        jobs = [(n, ep, name) for name, (n, eps) in SERIES.items() for ep in eps]
    else:
        series_n, eps = SERIES[args.series]
        jobs = [(series_n, args.day or eps[0], args.series)]
    try:
        for i, (series_n, episode, name) in enumerate(jobs):
            record_day(series_n, episode, name, port, args.cdp + i, args.hold)
            time.sleep(0.4)
    finally:
        httpd.shutdown()


if __name__ == "__main__":
    main()
