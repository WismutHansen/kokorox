"""
uv venv -p 3.12
uv pip install torch==2.5.1 numpy==2.0.2
uv run fetch_voices.py
"""

import io
import json

import numpy as np
import requests
import torch
import os

os.makedirs("data", exist_ok=True)

voices = [
    "af",
    "af_bella",
    "af_nicole",
    "af_sarah",
    "af_sky",
    "am_adam",
    "am_michael",
    "bf_emma",
    "bf_isabella",
    "bm_george",
    "bm_lewis",
]
voices_json = {}
# pattern = "https://huggingface.co/hexgrad/Kokoro-82M/resolve/main/voices/{voice}.pt"
# in case in China unable to access huggingface.co
pattern = "https://hf-mirror.com/hexgrad/Kokoro-82M/resolve/main/voices/{voice}.pt"
for voice in voices:
    url = pattern.format(voice=voice)
    print(f"Downloading {url}")
    r = requests.get(url)
    content = io.BytesIO(r.content)
    voice_data: np.ndarray = torch.load(content).numpy()
    print(f"voice data: {voice_data.shape}")
    # (511, 1, 256)
    voices_json[voice] = voice_data.tolist()

with open("data/voices.json", "w") as f:
    json.dump(voices_json, f, separators=(",", ":"))
