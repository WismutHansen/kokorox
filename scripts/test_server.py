#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = [
#     "openai",
#     "requests",
#     "rich",
# ]
# ///

"""
cli_tts.py – A small CLI utility to talk to the Kokoro‑backed TTS server that
mimics the OpenAI `/v1/audio/speech` endpoint.

Sub‑commands
------------
• **speak**   (default)  Synthesize text → wav/mp3.
• **ping**                Quick health‑check; prints "OK" if server responds.
• **voices**              Show a hard‑coded list of voice IDs.
• **langs**               Show a hard‑coded list of language codes.

Examples
~~~~~~~~
$ cli_tts.py speak "Hello world"               # http://localhost:3333 (WAV)
$ cli_tts.py speak "Salut" -p 8080 -f mp3 -o out.mp3
$ cli_tts.py ping -p 8080                      # prints OK / error

Install deps once:
    uv pip install requests rich               # or pip, pipx, etc.
"""

from __future__ import annotations

import sys
from argparse import ArgumentParser, Namespace
from pathlib import Path
from typing import Any, Final

import requests
from rich import print  # nice colours for free
from rich.table import Table

DEFAULT_PORT: Final[int] = 3333
DEFAULT_VOICE: Final[str] = "af_sky"
DEFAULT_MODEL: Final[str] = "kokoro"  # ignored by server but required
DEFAULT_FORMAT: Final[str] = "wav"
SUPPORTED_FORMATS: Final[set[str]] = {"wav", "mp3"}
SUPPORTED_VOICES: Final[list[str]] = [
    "af_alloy",
    "af_aoede",
    "af_bella",
    "af_heart",
    "af_jessica",
    "af_kore",
    "af_nicole",
    "af_nova",
    "af_olive",
    "af_river",
    "af_sarah",
    "af_sky",
    "am_adam",
    "am_echo",
    "am_eric",
    "am_fenrir",
    "am_liam",
    "am_michael",
    "am_onyx",
    "am_puck",
    "am_santa",
    "bf_alice",
    "bf_emma",
    "bf_isabella",
    "bf_lily",
    "bm_daniel",
    "bm_fable",
    "bm_george",
    "bm_lewis",
    "ef_dora",
    "em_alex",
    "em_santa",
    "en_sarah",
    "es_alejandro",
    "ff_siwis",
    "hf_alpha",
    "hf_beta",
    "hm_omega",
    "hm_psi",
    "if_sara",
    "im_nicola",
    "jf_alpha",
    "jf_gongitsune",
    "jf_nezumi",
    "jf_tebukuro",
    "jm_kumo",
    "pf_dora",
    "pm_alex",
    "pm_santa",
    "zf_xiaobei",
    "zf_xiaoni",
    "zf_xiaoxiao",
    "zf_xiaoyi",
    "zm_yunjian",
    "zm_yunxi",
    "zm_yunxia",
    "zm_yunyang",
]
SUPPORTED_LANGS: Final[list[str]] = [
    "en-us",
    "de-de",
    "es-es",
    "fr-fr",
    "it-it",
]

###############################################################################
# Argument parsing helpers
###############################################################################


def make_parser() -> ArgumentParser:
    p = ArgumentParser(prog="cli_tts.py", description="CLI for the Kokoro TTS server")
    sub = p.add_subparsers(dest="cmd", required=False)

    # SPEAK (default) --------------------------------------------------------
    sp_speak = sub.add_parser("speak", help="Convert text to speech")
    sp_speak.add_argument("text", help="Text to synthesise")
    sp_speak.add_argument("-p", "--port", type=int, default=DEFAULT_PORT)
    sp_speak.add_argument(
        "-f", "--format", choices=SUPPORTED_FORMATS, default=DEFAULT_FORMAT
    )
    sp_speak.add_argument("-v", "--voice", default=DEFAULT_VOICE)
    sp_speak.add_argument("-s", "--speed", type=float, default=1.0)
    sp_speak.add_argument("-i", "--initial-silence", type=int)
    sp_speak.add_argument("-l", "--language")
    sp_speak.add_argument("-a", "--auto-detect", action="store_true")
    sp_speak.add_argument("-m", "--model", default=DEFAULT_MODEL)
    sp_speak.add_argument("-o", "--out", default="tmp/speech.wav")

    # PING -------------------------------------------------------------------
    sp_ping = sub.add_parser("ping", help="Health‑check the server")
    sp_ping.add_argument("-p", "--port", type=int, default=DEFAULT_PORT)

    # VOICES & LANGS ---------------------------------------------------------
    sub.add_parser("voices", help="List available voice ids")
    sub.add_parser("langs", help="List supported language codes")

    return p


###############################################################################
# Command implementations
###############################################################################


def cmd_ping(args: Namespace) -> None:
    url = f"http://localhost:{args.port}/"
    try:
        r = requests.get(url, timeout=5)
        if r.status_code == 200 and r.text.strip() == "OK":
            print("[bold green]Server is up![/]")
        else:
            print(f"[red]Unexpected response {r.status_code}: {r.text!r}[/]")
            sys.exit(1)
    except requests.RequestException as e:
        print(f"[red]Ping failed:[/] {e}")
        sys.exit(1)


def cmd_speak(args: Namespace) -> None:
    base_url = f"http://localhost:{args.port}/v1/audio/speech"

    # Build payload matching SpeechRequest ----------------------------------
    payload: dict[str, Any] = {
        "model": args.model,
        "input": args.text,
        "voice": args.voice,
        "response_format": args.format,
        "speed": args.speed,
    }
    if args.initial_silence is not None:
        payload["initial_silence"] = args.initial_silence
    if args.language:
        payload["language"] = args.language
    if args.auto_detect:
        payload["auto_detect"] = True

    # Send POST --------------------------------------------------------------
    try:
        r = requests.post(base_url, json=payload, timeout=60)
    except requests.RequestException as e:
        print(f"[red]Connection error:[/] {e}")
        sys.exit(1)

    if r.status_code == 405:
        print(
            "[red]ERROR 405 Method Not Allowed – did you use POST?\n"
            "Make sure you are calling the correct endpoint (/v1/audio/speech) \
              and that the server is up on the given port.[/]"
        )
        sys.exit(1)

    if r.status_code != 200:
        print(f"[red]Server returned {r.status_code}:[/] {r.text}")
        sys.exit(1)

    # Save bytes -------------------------------------------------------------
    out_path = Path(args.out)
    if out_path.suffix.lower() not in (".wav", ".mp3"):
        out_path = out_path.with_suffix(f".{args.format}")
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_bytes(r.content)
    print(f"[green]Saved → {out_path.resolve()}[/]")


def cmd_list(items: list[str], title: str) -> None:
    tb = Table(title=title)
    tb.add_column("#")
    tb.add_column("ID")
    for idx, item in enumerate(items, 1):
        tb.add_row(str(idx), item)
    print(tb)


###############################################################################
# Main entry
###############################################################################


def main() -> None:
    parser = make_parser()
    args = parser.parse_args()

    # Default to speak if no sub‑command supplied ---------------------------
    if args.cmd is None:
        args.cmd = "speak"

    if args.cmd == "ping":
        cmd_ping(args)
    elif args.cmd == "voices":
        cmd_list(SUPPORTED_VOICES, "Available voices")
    elif args.cmd == "langs":
        cmd_list(SUPPORTED_LANGS, "Supported languages")
    elif args.cmd == "speak":
        cmd_speak(args)
    else:
        parser.error(f"Unknown command {args.cmd!r}")


if __name__ == "__main__":
    main()
