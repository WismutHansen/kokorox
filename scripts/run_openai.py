import requests

url = "http://localhost:3000/v1/audio/speech"
payload = {
    "model": "tts-1",
    "input": "Hello, this is a test of the Kokoro TTS system!",
    "voice": "af_sky"
}

response = requests.post(url, json=payload)
print(response.json())