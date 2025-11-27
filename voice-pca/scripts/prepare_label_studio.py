#!/usr/bin/env python3
"""
Prepare Label Studio Import Data

This script creates JSON files for importing into Label Studio:
1. Audio file references
2. Pre-filled metadata for each sample
3. Proper format for Label Studio's data manager

Usage:
    uv run python scripts/prepare_label_studio.py
"""

import argparse
import json
from pathlib import Path
from urllib.parse import quote


def create_label_studio_tasks(
    metadata_path: Path,
    audio_dir: Path,
    output_path: Path,
    audio_url_prefix: str = "/data/local-files/?d=",
) -> int:
    """
    Create Label Studio tasks JSON from metadata and audio files.

    Args:
        metadata_path: Path to all_mixes_metadata.json
        audio_dir: Directory containing audio files
        output_path: Output JSON file path
        audio_url_prefix: URL prefix for audio files in Label Studio

    Returns:
        Number of tasks created
    """
    # Load metadata
    with open(metadata_path) as f:
        metadata = json.load(f)

    mixes = metadata["mixes"]
    tasks = []

    for mix in mixes:
        mix_id = mix["id"]
        audio_file = audio_dir / f"{mix_id}.wav"

        # Skip if audio doesn't exist
        if not audio_file.exists():
            continue

        # Create components summary for display
        components = mix.get("components", [])
        if components:
            comp_strs = []
            for c in components[:4]:  # Limit display
                weight_pct = int(c["weight"] * 100)
                comp_strs.append(f"{c['voice']}:{weight_pct}%")
            components_summary = ", ".join(comp_strs)
            if len(components) > 4:
                components_summary += f" (+{len(components)-4} more)"
        else:
            components_summary = "Unknown"

        # Determine mix type and language info
        mix_type = mix.get("type", "unknown")
        if mix_type == "intra_language":
            language = mix.get("language", "unknown")
            type_display = f"Intra-language ({language})"
        else:
            languages = mix.get("languages", [])
            type_display = f"Inter-language ({', '.join(languages[:3])})"

        # Audio URL for Label Studio
        # Use relative path from audio_dir
        audio_url = f"{audio_url_prefix}{quote(str(audio_file.absolute()))}"

        task = {
            "data": {
                "audio": audio_url,
                "id": mix_id,
                "type": type_display,
                "components_summary": components_summary,
                # Store full metadata for reference
                "mix_type": mix.get("type"),
                "languages": mix.get("languages", [mix.get("language")]),
                "genders": mix.get("genders", []),
                "component_count": len(components),
            },
            "meta": {
                "original_id": mix_id,
                "generated_from": metadata_path.name,
            },
        }

        tasks.append(task)

    # Save tasks
    with open(output_path, "w") as f:
        json.dump(tasks, f, indent=2)

    return len(tasks)


def create_local_serving_config(
    audio_dir: Path,
    output_path: Path,
):
    """
    Create a simple HTML file to serve audio locally for testing.
    """
    html = f"""<!DOCTYPE html>
<html>
<head>
    <title>Voice PCA Audio Browser</title>
    <style>
        body {{ font-family: system-ui, sans-serif; max-width: 1200px; margin: 0 auto; padding: 20px; }}
        .sample {{ border: 1px solid #ddd; padding: 15px; margin: 10px 0; border-radius: 5px; }}
        .sample:hover {{ background: #f9f9f9; }}
        .id {{ font-weight: bold; font-family: monospace; }}
        .meta {{ color: #666; font-size: 0.9em; margin-top: 5px; }}
        audio {{ width: 100%; margin-top: 10px; }}
        h1 {{ border-bottom: 2px solid #333; padding-bottom: 10px; }}
        .stats {{ background: #f0f0f0; padding: 15px; border-radius: 5px; margin-bottom: 20px; }}
    </style>
</head>
<body>
    <h1>Voice PCA Audio Samples</h1>
    <div class="stats">
        <p>Audio directory: <code>{audio_dir.absolute()}</code></p>
        <p>Use this page to preview samples before Label Studio annotation.</p>
    </div>
    <div id="samples"></div>
    <script>
        // This would be populated dynamically in a real server setup
        // For now, just show instructions
        document.getElementById('samples').innerHTML = `
            <p>To browse samples:</p>
            <ol>
                <li>Start a local server: <code>cd {audio_dir.absolute()} && python -m http.server 8080</code></li>
                <li>Audio files will be available at <code>http://localhost:8080/</code></li>
            </ol>
            <p>Or use Label Studio with local file serving enabled.</p>
        `;
    </script>
</body>
</html>
"""

    with open(output_path, "w") as f:
        f.write(html)


def main():
    parser = argparse.ArgumentParser(
        description="Prepare data for Label Studio import"
    )
    parser.add_argument(
        "--metadata",
        type=Path,
        default=Path(__file__).parent.parent / "data" / "mixed" / "all_mixes_metadata.json",
        help="Path to metadata JSON",
    )
    parser.add_argument(
        "--audio-dir",
        type=Path,
        default=Path(__file__).parent.parent / "data" / "audio",
        help="Directory containing audio files",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=Path(__file__).parent.parent / "data" / "label_studio_tasks.json",
        help="Output JSON file for Label Studio",
    )
    parser.add_argument(
        "--url-prefix",
        type=str,
        default="/data/local-files/?d=",
        help="URL prefix for audio files in Label Studio",
    )

    args = parser.parse_args()

    # Check inputs
    if not args.metadata.exists():
        print(f"ERROR: Metadata file not found: {args.metadata}")
        print("Run voice_mixer.py first to generate mixed voices.")
        return 1

    if not args.audio_dir.exists():
        print(f"WARNING: Audio directory not found: {args.audio_dir}")
        print("Run generate_audio.py first to create audio samples.")
        # Continue anyway to create the task file

    # Create tasks
    print(f"Loading metadata from: {args.metadata}")
    num_tasks = create_label_studio_tasks(
        args.metadata,
        args.audio_dir,
        args.output,
        args.url_prefix,
    )

    print(f"Created {num_tasks} Label Studio tasks")
    print(f"Output: {args.output}")

    # Create browser page
    browser_path = args.audio_dir.parent / "audio_browser.html"
    create_local_serving_config(args.audio_dir, browser_path)
    print(f"Created audio browser: {browser_path}")

    # Instructions
    print("\n" + "=" * 50)
    print("Label Studio Setup Instructions:")
    print("=" * 50)
    print("""
1. Install Label Studio:
   pip install label-studio

2. Start Label Studio with local file serving:
   LABEL_STUDIO_LOCAL_FILES_SERVING_ENABLED=true label-studio start

3. Create a new project and import:
   - Labeling config: config/label_studio_config.xml
   - Tasks: data/label_studio_tasks.json

4. Configure storage (Settings > Cloud Storage):
   - Add Local Files storage
   - Set root directory to your audio folder

5. Start annotating!
""")

    return 0


if __name__ == "__main__":
    exit(main())
