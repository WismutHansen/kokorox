# Kokoro Voice PCA Analysis

This project creates a dataset of mixed voice vectors for Principal Component Analysis (PCA) of the Kokoro TTS voice space. The goal is to understand the latent voice characteristics and their perceptual correlates.

## Overview

The Kokoro TTS system uses 256-dimensional style embeddings indexed by token length (shape: 510x1x256). This project:

1. Generates random voice mixtures (intra-language and inter-language)
2. Creates audio samples for perceptual evaluation
3. Provides a Label Studio annotation schema for human labeling
4. Enables PCA analysis to discover voice embedding structure

## Project Structure

```
voice-pca/
├── scripts/
│   ├── voice_mixer.py      # Generate mixed voice vectors
│   └── generate_audio.py   # Create audio samples using koko CLI
├── data/
│   ├── voices/             # (empty, uses parent data dir)
│   ├── mixed/              # Mixed voice NPZ files + metadata
│   └── audio/              # Generated audio samples
├── config/
│   └── label_studio_config.xml  # Annotation interface config
├── docs/
│   └── methodology.md      # Detailed methodology documentation
├── pyproject.toml          # Python project config
└── README.md               # This file
```

## Quick Start

### 1. Generate Mixed Voices

```bash
cd voice-pca
uv run python scripts/voice_mixer.py --intra-mixes 150 --inter-mixes 150
```

Options:
- `--list-voices`: Show all available source voices
- `--intra-mixes N`: Number of same-language mixes
- `--inter-mixes N`: Number of cross-language mixes
- `--min-voices N`: Minimum voices per mix (default: 2)
- `--max-voices N`: Maximum voices per mix (default: 4)
- `--seed N`: Random seed for reproducibility

### 2. Generate Audio Samples

```bash
uv run python scripts/generate_audio.py --workers 4
```

Options:
- `--koko PATH`: Path to koko binary (auto-detected)
- `--workers N`: Parallel generation workers
- `--sentence-index N`: Which test sentence (0-2)
- `--limit N`: Process only first N voices (for testing)
- `--voice-filter PREFIX`: Only process matching voices

### 3. Set Up Label Studio

1. Install Label Studio: `pip install label-studio`
2. Start server with local files: `LABEL_STUDIO_LOCAL_FILES_SERVING_ENABLED=true label-studio start`
3. Create new project
4. Import labeling config from `config/label_studio_config.xml`
5. Configure local file storage pointing to `data/audio/`
6. Run `uv run python scripts/prepare_label_studio.py` to generate import JSON
7. Import tasks from `data/label_studio_tasks.json`

Note: All audio samples use English test sentences for consistent cross-voice comparison,
regardless of the original voice language. This ensures fair perceptual evaluation.

## Voice Mixing Methodology

### Intra-Language Mixing

Voices within the same language are mixed to explore:
- Gender blending (male + female voices)
- Speaker characteristic interpolation
- Regional accent mixing (e.g., US + UK English)

### Inter-Language Mixing

Voices across different languages are mixed to:
- Explore the full voice space
- Discover language-independent features
- Create novel voice characteristics

### Weight Distribution

Weights are sampled from a Dirichlet distribution, ensuring:
- Natural-sounding weight distributions
- Avoiding extreme single-voice dominance
- Reproducible random mixing

## Annotation Classifiers

The Label Studio interface captures these perceptual dimensions:

| Classifier | Scale | Description |
|------------|-------|-------------|
| **Quality** | 1-5 | Overall quality (1=unusable, 5=production-ready) |
| **Bookmark** | 3-choice | favorite / promising / skip - earmark good voices |
| Gender | 5-point | strongly_masculine to strongly_feminine |
| Age | 5-class | child, teenager, young_adult, middle_aged, senior |
| Pitch | 1-7 | Very Low to Very High |
| Pace | 1-7 | Very Slow to Very Fast |
| Energy | 1-7 | Calm/Monotone to Very Energetic |
| Warmth | 1-7 | Cold/Distant to Very Warm |
| Clarity | 1-7 | Unclear to Crystal Clear |
| Accent | 1-7 | No Accent to Very Strong |
| Formality | 1-7 | Very Casual to Very Formal |
| Naturalness | 1-7 | Very Synthetic to Completely Natural |

Additional annotations:
- Emotional tone (multi-select)
- Use case fit (multi-select)
- Free-form notes

## PCA Analysis Workflow

After annotation, run PCA analysis to discover:

1. **Primary Components**: Which dimensions explain most variance?
2. **Perceptual Correlates**: How do components relate to annotations?
3. **Voice Clusters**: Are there natural groupings in voice space?
4. **Mixing Effects**: How does inter-language mixing affect perception?

## Available Voices (54 total)

| Language | Voices |
|----------|--------|
| American English | af_alloy, af_aoede, af_bella, af_heart, af_jessica, af_kore, af_nicole, af_nova, af_river, af_sarah, af_sky, am_adam, am_echo, am_eric, am_fenrir, am_liam, am_michael, am_onyx, am_puck, am_santa |
| British English | bf_alice, bf_emma, bf_isabella, bf_lily, bm_daniel, bm_fable, bm_george, bm_lewis |
| Spanish | ef_dora, em_alex, em_santa |
| Portuguese | pf_dora, pm_alex, pm_santa |
| French | ff_siwis |
| Italian | if_sara, im_nicola |
| Japanese | jf_alpha, jf_gongitsune, jf_nezumi, jf_tebukuro, jm_kumo |
| Hindi | hf_alpha, hf_beta, hm_omega, hm_psi |
| Chinese | zf_xiaobei, zf_xiaoni, zf_xiaoxiao, zf_xiaoyi, zm_yunjian, zm_yunxi, zm_yunxia, zm_yunyang |

## File Formats

### Mixed Voices NPZ

```python
import numpy as np
npz = np.load('data/mixed/all_mixes.npz')
voice_vector = npz['mix_intra_en-us_0001']  # Shape: (510, 1, 256)
```

### Metadata JSON

```json
{
  "generated_at": "2025-01-15T10:30:00",
  "total_mixes": 300,
  "mixes": [
    {
      "id": "mix_intra_en-us_0001",
      "type": "intra_language",
      "language": "en-us",
      "components": [
        {"voice": "af_heart", "weight": 0.4},
        {"voice": "af_sarah", "weight": 0.6}
      ],
      "genders": ["female"]
    }
  ]
}
```

## Requirements

- Python 3.11+
- numpy
- Rust toolchain (for building koko)
- Label Studio (for annotation)

## License

Part of the kokorox project. See parent LICENSE file.
