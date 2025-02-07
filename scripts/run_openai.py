from openai import OpenAI

base_url = "http://localhost:3000/v1"

client = OpenAI(base_url=base_url, api_key="sfrhg453656")

speech_file_path = "tmp/speech.wav"
response = client.audio.speech.create(
    model="anything can go here",
    voice="am_michael", # or voice=NotGiven(), (`from openai import NotGiven`) to use the server default
    input="Today is a wonderful day to build something people love!",
)
response.write_to_file(speech_file_path)
