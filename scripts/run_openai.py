#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = [
#     "openai",
# ]
# ///
"""
Tiny CLI for OpenAI TTS running against a local proxy.

USAGE
  tts.py "Hello world"                  # defaults → http://localhost:3333
  tts.py "Hello, my friend!" -p 8080 -o out.wav
  tts.py -h                             # full help
"""

from __future__ import annotations
from pathlib import Path
from argparse import ArgumentParser, Namespace
from openai import OpenAI


def parse_args() -> Namespace:
    ap = ArgumentParser(prog="tts.py", description="OpenAI Text-to-Speech helper")
    ap.add_argument("text", help="Text to read aloud")
    ap.add_argument(
        "-p",
        "--port",
        type=int,
        default=3333,
        help="Port of the local TTS proxy (default: 3000",
    )
    ap.add_argument(
        "-m",
        "--model",
        default="anything can go here",
        help="Model name sent to the server",
    )
    ap.add_argument(
        "-v",
        "--voice",
        default="am_michael",
        help="Voice ID (server must know it)",
    )
    ap.add_argument(
        "-o",
        "--out",
        default="tmp/speech.wav",
        help="Destination file path",
    )
    return ap.parse_args()


def main() -> None:
    args = parse_args()

    # Build base URL from chosen port
    base_url = f"http://localhost:{args.port}/v1"
    client = OpenAI(base_url=base_url, api_key="dummy-key-ignored-by-proxy")

    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)  # create tmp/, tmp/foo/, …

    response = client.audio.speech.create(
        model=args.model,
        voice=args.voice,
        input=args.text,
    )
    response.write_to_file(out_path)
    print(f"Saved → {out_path.resolve()}")


if __name__ == "__main__":
    main()
