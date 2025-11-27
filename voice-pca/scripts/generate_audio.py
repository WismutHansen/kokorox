#!/usr/bin/env python3
"""
Audio Generator for Mixed Voice Vectors

This script generates test audio files for all mixed voice vectors using
the Kokoro TTS system. It creates audio samples that can be used for:
1. Subjective evaluation in Label Studio
2. PCA analysis correlation with perceptual features

Usage:
    uv run python scripts/generate_audio.py --help
"""

import argparse
import json
import subprocess
import sys
from pathlib import Path
from concurrent.futures import ThreadPoolExecutor, as_completed
from typing import Optional

import numpy as np


# Test sentences for different languages
TEST_SENTENCES = {
    "en-us": [
        "Hello, this is a test of the voice synthesis system.",
        "The quick brown fox jumps over the lazy dog.",
        "Technology has transformed how we communicate and work.",
    ],
    "en-gb": [
        "Good afternoon, how may I assist you today?",
        "The weather in London is quite lovely this time of year.",
        "Shall we proceed with the demonstration?",
    ],
    "es": [
        "Hola, esta es una prueba del sistema de sintesis de voz.",
        "El rapido zorro marron salta sobre el perro perezoso.",
        "La tecnologia ha transformado nuestra forma de comunicarnos.",
    ],
    "fr": [
        "Bonjour, ceci est un test du systeme de synthese vocale.",
        "Le renard brun rapide saute par-dessus le chien paresseux.",
        "La technologie a transforme notre facon de communiquer.",
    ],
    "it": [
        "Ciao, questa e una prova del sistema di sintesi vocale.",
        "La volpe marrone veloce salta sopra il cane pigro.",
        "La tecnologia ha trasformato il nostro modo di comunicare.",
    ],
    "ja": [
        "こんにちは、これは音声合成システムのテストです。",
        "素早い茶色のキツネは怠惰な犬を飛び越えます。",
        "テクノロジーは私たちのコミュニケーション方法を変革しました。",
    ],
    "zh": [
        "你好，这是语音合成系统的测试。",
        "敏捷的棕色狐狸跳过了懒惰的狗。",
        "技术改变了我们的沟通方式。",
    ],
    "pt": [
        "Ola, este e um teste do sistema de sintese de voz.",
        "A raposa marrom rapida salta sobre o cao preguicoso.",
        "A tecnologia transformou a forma como nos comunicamos.",
    ],
    "hi": [
        "नमस्ते, यह वॉयस सिंथेसिस सिस्टम का एक टेस्ट है।",
        "तेज भूरी लोमड़ी आलसी कुत्ते के ऊपर से कूदती है।",
        "प्रौद्योगिकी ने हमारे संवाद करने के तरीके को बदल दिया है।",
    ],
    # Fallback for mixed/unknown
    "default": [
        "Hello, this is a test of the voice synthesis system.",
        "The quick brown fox jumps over the lazy dog.",
        "Technology has transformed how we communicate and work.",
    ],
}


def get_test_sentence(language: str, index: int = 0) -> str:
    """Get test sentence for a given language."""
    # Normalize language code
    lang_key = language.split("-")[0] if "-" in language else language

    # Get sentences for language or fallback
    sentences = TEST_SENTENCES.get(lang_key, TEST_SENTENCES["default"])

    return sentences[index % len(sentences)]


def create_custom_voices_npz(
    mixed_voices_path: Path,
    original_voices_path: Path,
    output_path: Path,
) -> Path:
    """
    Create a combined NPZ file with original + mixed voices.

    The koko CLI loads voices from an NPZ file, so we need to combine
    our mixed voices with the original voices into a single file.
    """
    print(f"Creating combined voices NPZ at: {output_path}")

    # Load original voices
    original = np.load(original_voices_path)
    original_voices = {name: original[name] for name in original.files}
    print(f"  Original voices: {len(original_voices)}")

    # Load mixed voices
    mixed = np.load(mixed_voices_path)
    mixed_voices = {name: mixed[name] for name in mixed.files}
    print(f"  Mixed voices: {len(mixed_voices)}")

    # Combine
    combined = {**original_voices, **mixed_voices}
    print(f"  Total voices: {len(combined)}")

    # Save combined NPZ
    np.savez_compressed(output_path, **combined)
    print(f"  Saved to: {output_path}")

    return output_path


def generate_audio_for_voice(
    voice_id: str,
    metadata: dict,
    output_dir: Path,
    voices_path: Path,
    koko_binary: Path,
    sentence_index: int = 0,
) -> dict:
    """
    Generate audio for a single mixed voice.

    Returns a result dict with status and paths.
    """
    result = {
        "voice_id": voice_id,
        "status": "pending",
        "audio_path": None,
        "error": None,
    }

    try:
        # Determine language for test sentence
        if metadata.get("type") == "intra_language":
            language = metadata.get("language", "en-us")
        else:
            # For inter-language, use English as common ground
            languages = metadata.get("languages", ["en-us"])
            # Prefer English if available
            language = "en-us" if "en-us" in languages else languages[0]

        # For PCA analysis, we use English for all voices to ensure consistent
        # comparison across the voice embedding space. This avoids issues with
        # espeak-rs language support bugs and makes perceptual comparison fair.
        #
        # Known espeak-rs issues:
        # - en-gb, pt-pt, zh don't work with espeak_SetVoiceByName
        # - kokorox phonemizer maps cmn->zh internally but espeak-rs doesn't support "zh"
        espeak_language = "en-us"

        # Always use English test sentence for consistent voice comparison
        test_sentence = get_test_sentence("en-us", sentence_index)

        # Output path
        audio_path = output_dir / f"{voice_id}.wav"

        # Build koko command
        cmd = [
            str(koko_binary),
            "--data", str(voices_path),
            "--style", voice_id,
            "--lan", espeak_language,  # Use mapped language for espeak compatibility
            "--force-style",
            "text",
            test_sentence,
            "--output", str(audio_path),
        ]

        # Run koko
        proc = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=60,
        )

        # Check if audio was actually generated (koko may crash after saving due to mutex issue)
        # The returncode may be non-zero due to cleanup crash, but audio is still valid
        if audio_path.exists() and audio_path.stat().st_size > 1000:
            result["status"] = "success"
            result["audio_path"] = str(audio_path)
        elif proc.returncode != 0:
            result["status"] = "error"
            result["error"] = proc.stderr[:500] if proc.stderr else "Unknown error"
        else:
            result["status"] = "error"
            result["error"] = "Audio file not created or too small"

    except subprocess.TimeoutExpired:
        result["status"] = "timeout"
        result["error"] = "Generation timed out after 60s"
    except Exception as e:
        result["status"] = "error"
        result["error"] = str(e)

    return result


def main():
    parser = argparse.ArgumentParser(
        description="Generate audio samples for mixed voice vectors"
    )
    parser.add_argument(
        "--mixed-voices",
        type=Path,
        default=Path(__file__).parent.parent / "data" / "mixed" / "all_mixes.npz",
        help="Path to mixed voices NPZ file",
    )
    parser.add_argument(
        "--metadata",
        type=Path,
        default=Path(__file__).parent.parent / "data" / "mixed" / "all_mixes_metadata.json",
        help="Path to mixed voices metadata JSON",
    )
    parser.add_argument(
        "--original-voices",
        type=Path,
        default=Path(__file__).parent.parent.parent / "data" / "voices-v1.0.bin",
        help="Path to original voices NPZ file",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=Path(__file__).parent.parent / "data" / "audio",
        help="Output directory for audio files",
    )
    parser.add_argument(
        "--koko",
        type=Path,
        default=None,
        help="Path to koko binary (auto-detected if not specified)",
    )
    parser.add_argument(
        "--workers",
        type=int,
        default=4,
        help="Number of parallel workers",
    )
    parser.add_argument(
        "--sentence-index",
        type=int,
        default=0,
        help="Which test sentence to use (0-2)",
    )
    parser.add_argument(
        "--limit",
        type=int,
        default=None,
        help="Limit number of voices to process (for testing)",
    )
    parser.add_argument(
        "--voice-filter",
        type=str,
        default=None,
        help="Only process voices matching this prefix",
    )

    args = parser.parse_args()

    # Find koko binary
    if args.koko:
        koko_binary = args.koko
    else:
        # Try common locations
        possible_paths = [
            Path(__file__).parent.parent.parent / "target" / "release" / "koko",
            Path(__file__).parent.parent.parent / "target" / "debug" / "koko",
            Path.home() / ".cargo" / "bin" / "koko",
            Path("/usr/local/bin/koko"),
        ]
        koko_binary = None
        for p in possible_paths:
            if p.exists():
                koko_binary = p
                break

        if not koko_binary:
            print("ERROR: Could not find koko binary. Please specify with --koko")
            print("Try building first: cargo build --release")
            sys.exit(1)

    print(f"Using koko binary: {koko_binary}")

    # Create output directory
    args.output.mkdir(parents=True, exist_ok=True)

    # Create combined voices file
    combined_voices_path = args.output.parent / "voices" / "combined_voices.npz"
    combined_voices_path.parent.mkdir(parents=True, exist_ok=True)
    if not combined_voices_path.exists() or args.mixed_voices.stat().st_mtime > combined_voices_path.stat().st_mtime:
        create_custom_voices_npz(
            args.mixed_voices,
            args.original_voices,
            combined_voices_path,
        )
    else:
        print(f"Using existing combined voices: {combined_voices_path}")

    # Load metadata
    with open(args.metadata) as f:
        metadata = json.load(f)

    mixes = metadata["mixes"]
    print(f"Total mixes in metadata: {len(mixes)}")

    # Apply filters
    if args.voice_filter:
        mixes = [m for m in mixes if m["id"].startswith(args.voice_filter)]
        print(f"After filter '{args.voice_filter}': {len(mixes)}")

    if args.limit:
        mixes = mixes[:args.limit]
        print(f"Limited to: {len(mixes)}")

    # Generate audio
    print(f"\nGenerating audio for {len(mixes)} voices...")
    print(f"Output directory: {args.output}")
    print(f"Using {args.workers} workers\n")

    results = []
    success_count = 0
    error_count = 0

    # Create metadata lookup
    meta_lookup = {m["id"]: m for m in mixes}

    with ThreadPoolExecutor(max_workers=args.workers) as executor:
        futures = {}

        for mix in mixes:
            voice_id = mix["id"]
            future = executor.submit(
                generate_audio_for_voice,
                voice_id=voice_id,
                metadata=mix,
                output_dir=args.output,
                voices_path=combined_voices_path,
                koko_binary=koko_binary,
                sentence_index=args.sentence_index,
            )
            futures[future] = voice_id

        for future in as_completed(futures):
            voice_id = futures[future]
            result = future.result()
            results.append(result)

            if result["status"] == "success":
                success_count += 1
                print(f"  [OK] {voice_id}")
            else:
                error_count += 1
                print(f"  [FAIL] {voice_id}: {result['error'][:100]}")

    # Summary
    print(f"\n{'='*50}")
    print(f"Generation complete!")
    print(f"  Success: {success_count}")
    print(f"  Errors: {error_count}")
    print(f"  Total: {len(results)}")

    # Save generation results
    results_path = args.output / "generation_results.json"
    with open(results_path, "w") as f:
        json.dump(
            {
                "total": len(results),
                "success": success_count,
                "errors": error_count,
                "results": results,
            },
            f,
            indent=2,
        )
    print(f"\nResults saved to: {results_path}")


if __name__ == "__main__":
    main()
