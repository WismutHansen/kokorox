from pathlib import Path
from openai import OpenAI

url = "http://localhost:3000/v1/audio/speech"


client = OpenAI(base_url=url, api_key="sfrhg453656")
speech_file_path = "tmp/speech.mp3"
response = client.audio.speech.create(
    model="tts-1",
    voice="alloy",
    input="Today is a wonderful day to build something people love!",
)
response.write_to_file(speech_file_path)
