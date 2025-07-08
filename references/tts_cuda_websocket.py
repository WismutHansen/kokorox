# /// script
# requires-python = ">=3.12"
# dependencies = [
#     "moshi==0.2.8",
#     "torch",
#     "numpy",
#     "websockets",
#     "asyncio",
# ]
# ///

import asyncio
import json
import queue
import threading
import time
import base64
import wave
import io
from typing import Optional, Dict, Any

import numpy as np
import torch
import websockets
from moshi.models.loaders import CheckpointInfo
from moshi.models.tts import DEFAULT_DSM_TTS_REPO, DEFAULT_DSM_TTS_VOICE_REPO, TTSModel


class TTSCudaWebSocketServer:
    def __init__(self, 
                 hf_repo: str = DEFAULT_DSM_TTS_REPO, 
                 voice_repo: str = DEFAULT_DSM_TTS_VOICE_REPO, 
                 device: str = "cuda",
                 temp: float = 0.6,
                 cfg_coef: float = 2.0):
        self.hf_repo = hf_repo
        self.voice_repo = voice_repo
        self.device = device
        self.temp = temp
        self.cfg_coef = cfg_coef
        self.tts_model = None
        self.current_voice = "expresso/ex03-ex01_happy_001_channel1_334s.wav"
        self.available_voices = []
        self.sample_rate = 24000
        
    def log(self, level: str, msg: str):
        print(f"[{level.upper()}] {msg}")

    async def initialize(self):
        """Initialize the TTS model with CUDA support"""
        self.log("info", f"Initializing TTS model on {self.device}...")
        
        # Check if CUDA is available
        if self.device == "cuda" and not torch.cuda.is_available():
            self.log("warning", "CUDA not available, falling back to CPU")
            self.device = "cpu"
        
        self.log("info", f"Loading model from {self.hf_repo}")
        checkpoint_info = CheckpointInfo.from_hf_repo(self.hf_repo)
        self.tts_model = TTSModel.from_checkpoint_info(
            checkpoint_info, 
            n_q=32, 
            temp=self.temp, 
            device=self.device
        )
        
        self.sample_rate = self.tts_model.mimi.sample_rate
        self.log("info", f"TTS model initialized on {self.device} with sample rate {self.sample_rate}")
        
        # Get available voices
        try:
            self.available_voices = self.tts_model.list_voices()
            self.log("info", f"Available voices: {len(self.available_voices)}")
        except:
            self.available_voices = [self.current_voice]
            self.log("info", f"Using default voice: {self.current_voice}")

    def encode_audio_to_base64(self, pcm_data: np.ndarray) -> str:
        """Encode PCM audio data to base64 (raw PCM, not WAV)"""
        # Convert float32 to int16 and encode as base64
        audio_int16 = (np.clip(pcm_data, -1, 1) * 32767).astype(np.int16)
        return base64.b64encode(audio_int16.tobytes()).decode('utf-8')

    async def synthesize_text(self, text: str, voice: str = None) -> list:
        """Synthesize text to speech and return audio chunks"""
        if not self.tts_model:
            raise RuntimeError("TTS model not initialized")
        
        if voice and voice != self.current_voice:
            self.current_voice = voice
            self.log("info", f"Switching to voice: {voice}")
        
        audio_chunks = []
        pcm_queue = queue.Queue()
        
        def _on_frame(frame):
            if (frame != -1).all():
                with torch.no_grad():
                    pcm = self.tts_model.mimi.decode(frame[:, 1:, :]).cpu().numpy()
                    pcm_data = np.clip(pcm[0, 0], -1, 1)
                    pcm_queue.put_nowait(pcm_data)

        # Prepare the text for synthesis
        entries = self.tts_model.prepare_script([text], padding_between=1)
        voice_path = self.tts_model.get_voice_path(self.current_voice)
        condition_attributes = self.tts_model.make_condition_attributes(
            [voice_path], cfg_coef=self.cfg_coef
        )

        self.log("info", f"Synthesizing: {text[:50]}...")
        begin = time.time()
        
        # Run synthesis in a separate thread
        def run_synthesis():
            with self.tts_model.mimi.streaming(1):
                self.tts_model.generate(
                    [entries], 
                    [condition_attributes], 
                    on_frame=_on_frame
                )
        
        synthesis_thread = threading.Thread(target=run_synthesis)
        synthesis_thread.start()
        
        # Collect audio chunks as they're generated
        while synthesis_thread.is_alive() or not pcm_queue.empty():
            try:
                pcm_data = pcm_queue.get(timeout=0.1)
                audio_b64 = self.encode_audio_to_base64(pcm_data)
                audio_chunks.append(audio_b64)
            except queue.Empty:
                continue
        
        synthesis_thread.join()
        
        time_taken = time.time() - begin
        self.log("info", f"Synthesis took {time_taken:.2f}s, generated {len(audio_chunks)} chunks")
        
        return audio_chunks

    async def handle_client(self, websocket):
        """Handle WebSocket client connections"""
        self.log("info", f"Client connected: {websocket.remote_address}")
        
        try:
            async for message in websocket:
                try:
                    data = json.loads(message)
                    command = data.get('command')
                    
                    if command == 'synthesize':
                        text = data.get('text', '')
                        voice = data.get('voice', self.current_voice)
                        
                        if not text:
                            await websocket.send(json.dumps({
                                'type': 'error',
                                'message': 'No text provided'
                            }))
                            continue
                        
                        # Send synthesis started message
                        await websocket.send(json.dumps({
                            'type': 'synthesis_started',
                            'text': text,
                            'voice': voice
                        }))
                        
                        # Generate audio chunks
                        audio_chunks = await self.synthesize_text(text, voice)
                        
                        # Send audio chunks
                        for i, chunk in enumerate(audio_chunks):
                            await websocket.send(json.dumps({
                                'type': 'audio_chunk',
                                'chunk': chunk,
                                'index': i,
                                'total': len(audio_chunks),
                                'sample_rate': self.sample_rate
                            }))
                        
                        # Send synthesis completed message
                        await websocket.send(json.dumps({
                            'type': 'synthesis_completed',
                            'total_chunks': len(audio_chunks)
                        }))
                    
                    elif command == 'list_voices':
                        await websocket.send(json.dumps({
                            'type': 'voices',
                            'voices': self.available_voices,
                            'current': self.current_voice
                        }))
                    
                    elif command == 'set_voice':
                        voice = data.get('voice', self.current_voice)
                        if voice in self.available_voices or voice == self.current_voice:
                            self.current_voice = voice
                            await websocket.send(json.dumps({
                                'type': 'voice_changed',
                                'voice': voice
                            }))
                        else:
                            await websocket.send(json.dumps({
                                'type': 'error',
                                'message': f'Voice not found: {voice}'
                            }))
                    
                    elif command == 'set_temperature':
                        temp = data.get('temperature', self.temp)
                        if 0.1 <= temp <= 2.0:
                            self.temp = temp
                            if self.tts_model:
                                self.tts_model.temp = temp
                            await websocket.send(json.dumps({
                                'type': 'temperature_changed',
                                'temperature': temp
                            }))
                        else:
                            await websocket.send(json.dumps({
                                'type': 'error',
                                'message': 'Temperature must be between 0.1 and 2.0'
                            }))
                    
                    elif command == 'set_cfg_coef':
                        cfg_coef = data.get('cfg_coef', self.cfg_coef)
                        if 0.5 <= cfg_coef <= 5.0:
                            self.cfg_coef = cfg_coef
                            await websocket.send(json.dumps({
                                'type': 'cfg_coef_changed',
                                'cfg_coef': cfg_coef
                            }))
                        else:
                            await websocket.send(json.dumps({
                                'type': 'error',
                                'message': 'CFG coefficient must be between 0.5 and 5.0'
                            }))
                    
                    elif command == 'status':
                        gpu_info = {}
                        if torch.cuda.is_available():
                            gpu_info = {
                                'cuda_available': True,
                                'cuda_device_count': torch.cuda.device_count(),
                                'cuda_current_device': torch.cuda.current_device(),
                                'cuda_device_name': torch.cuda.get_device_name(),
                                'cuda_memory_allocated': torch.cuda.memory_allocated(),
                                'cuda_memory_reserved': torch.cuda.memory_reserved(),
                            }
                        else:
                            gpu_info = {'cuda_available': False}
                        
                        await websocket.send(json.dumps({
                            'type': 'status',
                            'initialized': self.tts_model is not None,
                            'device': self.device,
                            'current_voice': self.current_voice,
                            'sample_rate': self.sample_rate,
                            'temperature': self.temp,
                            'cfg_coef': self.cfg_coef,
                            'gpu_info': gpu_info
                        }))
                    
                    else:
                        await websocket.send(json.dumps({
                            'type': 'error',
                            'message': f'Unknown command: {command}'
                        }))
                        
                except json.JSONDecodeError:
                    await websocket.send(json.dumps({
                        'type': 'error',
                        'message': 'Invalid JSON message'
                    }))
                except Exception as e:
                    self.log("error", f"Error processing message: {e}")
                    await websocket.send(json.dumps({
                        'type': 'error',
                        'message': str(e)
                    }))
                    
        except websockets.exceptions.ConnectionClosed:
            self.log("info", f"Client disconnected: {websocket.remote_address}")
        except Exception as e:
            self.log("error", f"Connection error: {e}")

    async def start_server(self, host: str = "localhost", port: int = 8766):
        """Start the WebSocket server"""
        self.log("info", "Initializing CUDA TTS model...")
        await self.initialize()
        
        self.log("info", f"Starting CUDA WebSocket server on {host}:{port}")
        server = await websockets.serve(self.handle_client, host, port)
        self.log("info", "CUDA WebSocket server started. Ready for connections.")
        
        await server.wait_closed()


async def main():
    import argparse
    
    parser = argparse.ArgumentParser(description="TTS CUDA WebSocket Server")
    parser.add_argument("--host", default="localhost", help="Host to bind to")
    parser.add_argument("--port", type=int, default=8766, help="Port to bind to")
    parser.add_argument("--hf-repo", default=DEFAULT_DSM_TTS_REPO, help="HF repo for models")
    parser.add_argument("--voice-repo", default=DEFAULT_DSM_TTS_VOICE_REPO, help="HF repo for voices")
    parser.add_argument("--device", default="cuda", help="Device to use (cuda/cpu)")
    parser.add_argument("--temp", type=float, default=0.6, help="Temperature for generation")
    parser.add_argument("--cfg-coef", type=float, default=2.0, help="CFG coefficient")
    
    args = parser.parse_args()
    
    server = TTSCudaWebSocketServer(
        hf_repo=args.hf_repo,
        voice_repo=args.voice_repo,
        device=args.device,
        temp=args.temp,
        cfg_coef=args.cfg_coef
    )
    
    try:
        await server.start_server(args.host, args.port)
    except KeyboardInterrupt:
        print("\nServer stopped by user")


if __name__ == "__main__":
    asyncio.run(main())