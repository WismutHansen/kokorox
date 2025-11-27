#!/usr/bin/env python3
"""
Voice Mixer for Kokoro TTS PCA Analysis

This script generates mixed voice vectors from the Kokoro TTS voice library
for Principal Component Analysis. It supports:
1. Intra-language mixing (voices within same language)
2. Inter-language mixing (voices across different languages)

Voice Vector Format:
- Shape: (510, 1, 256) per voice
- Stored in NPZ format (NumPy Zip archive)
- Each voice indexed by token length (0-510)

Naming Convention:
- Format: {language_code}{gender}_{name}
- Language codes: af/am (American), bf/bm (British), ef/em (Spanish),
                  ff (French), if/im (Italian), jf/jm (Japanese),
                  pf/pm (Portuguese), hf/hm (Hindi), zf/zm (Chinese)
"""

import argparse
import json
import random
import sys
from dataclasses import dataclass, field
from datetime import datetime
from pathlib import Path
from typing import Optional

import numpy as np


# Voice metadata structure
@dataclass
class VoiceInfo:
    name: str
    language: str
    language_full: str
    gender: str
    data: Optional[np.ndarray] = field(default=None, repr=False)


# Language mappings
LANGUAGE_MAP = {
    "af": ("en-us", "American English"),
    "am": ("en-us", "American English"),
    "bf": ("en-gb", "British English"),
    "bm": ("en-gb", "British English"),
    "ef": ("es", "Spanish"),
    "em": ("es", "Spanish"),
    "ff": ("fr", "French"),
    "if": ("it", "Italian"),
    "im": ("it", "Italian"),
    "jf": ("ja", "Japanese"),
    "jm": ("ja", "Japanese"),
    "pf": ("pt", "Portuguese"),
    "pm": ("pt", "Portuguese"),
    "hf": ("hi", "Hindi"),
    "hm": ("hi", "Hindi"),
    "zf": ("zh", "Chinese"),
    "zm": ("zh", "Chinese"),
}

GENDER_MAP = {
    "f": "female",
    "m": "male",
}


def parse_voice_name(name: str) -> VoiceInfo:
    """Parse voice name into structured info."""
    prefix = name[:2]
    lang_code, lang_full = LANGUAGE_MAP.get(prefix, ("unknown", "Unknown"))
    gender = GENDER_MAP.get(prefix[1], "unknown")

    return VoiceInfo(
        name=name,
        language=lang_code,
        language_full=lang_full,
        gender=gender,
    )


def load_voices(voices_path: Path) -> dict[str, VoiceInfo]:
    """Load all voices from NPZ file."""
    print(f"Loading voices from: {voices_path}")

    npz = np.load(voices_path)
    voices = {}

    for name in npz.files:
        info = parse_voice_name(name)
        info.data = npz[name]
        voices[name] = info

    print(f"Loaded {len(voices)} voices")
    return voices


def mix_voices(
    voices: list[tuple[VoiceInfo, float]],
) -> np.ndarray:
    """
    Mix multiple voices with given weights.

    Args:
        voices: List of (VoiceInfo, weight) tuples. Weights should sum to 1.0.

    Returns:
        Mixed voice vector with shape (510, 1, 256)
    """
    if not voices:
        raise ValueError("No voices to mix")

    # Normalize weights
    total_weight = sum(w for _, w in voices)
    normalized = [(v, w / total_weight) for v, w in voices]

    # Get first voice data for shape reference
    first_voice_data = voices[0][0].data
    if first_voice_data is None:
        raise ValueError("Voice data not loaded")

    # Initialize with zeros
    result = np.zeros_like(first_voice_data, dtype=np.float32)

    # Weighted sum
    for voice_info, weight in normalized:
        if voice_info.data is None:
            raise ValueError(f"Voice data not loaded for {voice_info.name}")
        result += voice_info.data * weight

    return result


def generate_intra_language_mixes(
    voices: dict[str, VoiceInfo],
    num_mixes: int,
    min_voices: int = 2,
    max_voices: int = 4,
    seed: Optional[int] = None,
) -> list[dict]:
    """
    Generate random mixes within each language.

    Returns list of mix specifications with metadata.
    """
    if seed is not None:
        random.seed(seed)
        np.random.seed(seed)

    # Group voices by language
    by_language: dict[str, list[VoiceInfo]] = {}
    for info in voices.values():
        lang = info.language
        if lang not in by_language:
            by_language[lang] = []
        by_language[lang].append(info)

    mixes = []
    mix_id = 0

    # Generate mixes per language
    mixes_per_lang = max(1, num_mixes // len(by_language))

    for lang, lang_voices in by_language.items():
        if len(lang_voices) < 2:
            print(f"Skipping {lang}: only {len(lang_voices)} voice(s)")
            continue

        for _ in range(mixes_per_lang):
            # Random number of voices to mix
            n_voices = random.randint(
                min_voices, min(max_voices, len(lang_voices))
            )

            # Random selection
            selected = random.sample(lang_voices, n_voices)

            # Random weights (Dirichlet distribution for natural mixing)
            weights = np.random.dirichlet(np.ones(n_voices)).tolist()

            # Create mix
            voice_weights = [(v, w) for v, w in zip(selected, weights)]
            mixed_data = mix_voices(voice_weights)

            # Generate mix name
            mix_name = f"mix_intra_{lang}_{mix_id:04d}"

            mix_spec = {
                "id": mix_name,
                "type": "intra_language",
                "language": lang,
                "language_full": selected[0].language_full,
                "components": [
                    {"voice": v.name, "weight": round(w, 4)}
                    for v, w in voice_weights
                ],
                "genders": list(set(v.gender for v in selected)),
                "data": mixed_data,
            }

            mixes.append(mix_spec)
            mix_id += 1

    print(f"Generated {len(mixes)} intra-language mixes")
    return mixes


def generate_inter_language_mixes(
    voices: dict[str, VoiceInfo],
    num_mixes: int,
    min_voices: int = 2,
    max_voices: int = 5,
    seed: Optional[int] = None,
) -> list[dict]:
    """
    Generate random mixes across different languages.

    Returns list of mix specifications with metadata.
    """
    if seed is not None:
        random.seed(seed)
        np.random.seed(seed)

    voice_list = list(voices.values())
    mixes = []

    for mix_id in range(num_mixes):
        # Random number of voices
        n_voices = random.randint(min_voices, min(max_voices, len(voice_list)))

        # Select voices ensuring language diversity
        selected = []
        available = voice_list.copy()

        while len(selected) < n_voices and available:
            voice = random.choice(available)
            selected.append(voice)
            # Remove same-language voices to encourage diversity
            # (but keep some for natural mixing)
            if random.random() > 0.3:
                available = [v for v in available if v.language != voice.language]
            if not available:
                available = [v for v in voice_list if v not in selected]

        # Random weights
        weights = np.random.dirichlet(np.ones(len(selected))).tolist()

        # Create mix
        voice_weights = [(v, w) for v, w in zip(selected, weights)]
        mixed_data = mix_voices(voice_weights)

        # Determine languages involved
        languages = list(set(v.language for v in selected))

        mix_name = f"mix_inter_{mix_id:04d}"

        mix_spec = {
            "id": mix_name,
            "type": "inter_language",
            "languages": languages,
            "components": [
                {
                    "voice": v.name,
                    "weight": round(w, 4),
                    "language": v.language,
                }
                for v, w in voice_weights
            ],
            "genders": list(set(v.gender for v in selected)),
            "data": mixed_data,
        }

        mixes.append(mix_spec)

    print(f"Generated {len(mixes)} inter-language mixes")
    return mixes


def save_mixes(
    mixes: list[dict],
    output_dir: Path,
    prefix: str = "mixed_voices",
) -> tuple[Path, Path]:
    """
    Save mixed voices to NPZ file and metadata to JSON.

    Returns paths to (npz_file, json_file).
    """
    output_dir.mkdir(parents=True, exist_ok=True)

    # Prepare data for NPZ
    voice_data = {}
    metadata = []

    for mix in mixes:
        mix_id = mix["id"]
        voice_data[mix_id] = mix["data"]

        # Metadata without numpy array
        meta = {k: v for k, v in mix.items() if k != "data"}
        metadata.append(meta)

    # Save NPZ
    npz_path = output_dir / f"{prefix}.npz"
    np.savez_compressed(npz_path, **voice_data)
    print(f"Saved voice data to: {npz_path}")

    # Save metadata JSON
    json_path = output_dir / f"{prefix}_metadata.json"
    with open(json_path, "w") as f:
        json.dump(
            {
                "generated_at": datetime.now().isoformat(),
                "total_mixes": len(mixes),
                "mixes": metadata,
            },
            f,
            indent=2,
        )
    print(f"Saved metadata to: {json_path}")

    return npz_path, json_path


def main():
    parser = argparse.ArgumentParser(
        description="Generate mixed voice vectors for PCA analysis"
    )
    parser.add_argument(
        "--voices",
        type=Path,
        default=Path(__file__).parent.parent.parent / "data" / "voices-v1.0.bin",
        help="Path to voices NPZ file",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=Path(__file__).parent.parent / "data" / "mixed",
        help="Output directory for mixed voices",
    )
    parser.add_argument(
        "--intra-mixes",
        type=int,
        default=100,
        help="Number of intra-language mixes to generate",
    )
    parser.add_argument(
        "--inter-mixes",
        type=int,
        default=100,
        help="Number of inter-language mixes to generate",
    )
    parser.add_argument(
        "--min-voices",
        type=int,
        default=2,
        help="Minimum voices per mix",
    )
    parser.add_argument(
        "--max-voices",
        type=int,
        default=4,
        help="Maximum voices per mix",
    )
    parser.add_argument(
        "--seed",
        type=int,
        default=42,
        help="Random seed for reproducibility",
    )
    parser.add_argument(
        "--list-voices",
        action="store_true",
        help="List available voices and exit",
    )

    args = parser.parse_args()

    # Load voices
    voices = load_voices(args.voices)

    if args.list_voices:
        print("\nAvailable voices by language:\n")
        by_lang: dict[str, list[VoiceInfo]] = {}
        for info in voices.values():
            lang = info.language_full
            if lang not in by_lang:
                by_lang[lang] = []
            by_lang[lang].append(info)

        for lang in sorted(by_lang.keys()):
            print(f"{lang}:")
            for v in sorted(by_lang[lang], key=lambda x: x.name):
                print(f"  - {v.name} ({v.gender})")
            print()
        return

    # Generate mixes
    intra_mixes = generate_intra_language_mixes(
        voices,
        num_mixes=args.intra_mixes,
        min_voices=args.min_voices,
        max_voices=args.max_voices,
        seed=args.seed,
    )

    inter_mixes = generate_inter_language_mixes(
        voices,
        num_mixes=args.inter_mixes,
        min_voices=args.min_voices,
        max_voices=args.max_voices,
        seed=args.seed + 1000,  # Different seed for variety
    )

    # Save separately
    save_mixes(intra_mixes, args.output, "intra_language_mixes")
    save_mixes(inter_mixes, args.output, "inter_language_mixes")

    # Also save combined
    all_mixes = intra_mixes + inter_mixes
    save_mixes(all_mixes, args.output, "all_mixes")

    print(f"\nTotal mixes generated: {len(all_mixes)}")
    print(f"  - Intra-language: {len(intra_mixes)}")
    print(f"  - Inter-language: {len(inter_mixes)}")


if __name__ == "__main__":
    main()
