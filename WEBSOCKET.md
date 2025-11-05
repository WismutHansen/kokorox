# WebSocket API Documentation

The Kokorox WebSocket API provides real-time text-to-speech synthesis with streaming audio chunks. This enables low-latency integration with external clients and applications.

## Connection

Connect to the WebSocket server using a standard WebSocket client:

```
ws://localhost:8080
```

The server address and port can be configured when starting the WebSocket server.

## Message Format

All messages are JSON-formatted text frames. The client sends commands as JSON objects, and the server responds with JSON objects containing results or status updates.

### Client Commands

All client messages must have a `command` field specifying the operation:

```json
{
  "command": "command_name",
  "field1": "value1",
  "field2": "value2"
}
```

### Server Responses

All server messages have a `type` field indicating the message purpose:

```json
{
  "type": "message_type",
  "field1": "value1",
  "field2": "value2"
}
```

## Available Commands

### 1. List Voices

Lists all available voice styles.

**Request:**
```json
{
  "command": "list_voices"
}
```

**Response:**
```json
{
  "type": "voices",
  "voice": "af_heart",
  "voices": ["af_heart", "af_sky", "am_michael", "ef_dora", "..."]
}
```

- `voice`: Currently selected voice
- `voices`: Array of all available voice IDs

### 2. Set Voice

Changes the active voice for synthesis. Supports both single voices and voice mixing.

**Request (single voice):**
```json
{
  "command": "set_voice",
  "voice": "af_sky"
}
```

**Request (voice mixing):**
```json
{
  "command": "set_voice",
  "voice": "af_sarah.4+af_nicole.6"
}
```

Voice mixing format: `voice1.weight+voice2.weight` where weight is an integer 0-10 representing percentage (e.g., `.4` = 40%, `.6` = 60%).

**Examples:**
- `"af_sarah.5+af_nicole.5"` - 50% of each voice
- `"jf_alpha.3+am_echo.7"` - 30% jf_alpha + 70% am_echo
- `"af_heart.2+af_sky.8"` - 20% af_heart + 80% af_sky

**Success Response:**
```json
{
  "type": "voice_changed",
  "voice": "af_sky"
}
```

**Error Response (invalid voice):**
```json
{
  "type": "error"
}
```

### 3. Set Language

Changes the active language for synthesis. Default is "en-us".

**Request:**
```json
{
  "command": "set_language",
  "language": "en-gb"
}
```

**Success Response:**
```json
{
  "type": "language_changed"
}
```

### 4. Set Speed

Changes the active speech speed for synthesis. Default is 1.0.

**Request:**
```json
{
  "command": "set_speed",
  "speed": 1.2
}
```

- `speed`: Float value between 0.1 and 3.0
  - 0.5 = half speed (slower)
  - 1.0 = normal speed
  - 2.0 = double speed (faster)

**Success Response:**
```json
{
  "type": "speed_changed"
}
```

### 5. Synthesize Speech

Generates speech from text with streaming audio chunks. Optionally override the language or speed for this request.

**Request:**
```json
{
  "command": "synthesize",
  "text": "Hello world! This is a test.",
  "language": "en-gb",
  "speed": 1.5
}
```

- `text`: Required. The text to synthesize.
- `language`: Optional. Override the current language for this request.
- `speed`: Optional. Override the current speed for this request (0.1-3.0).

**Response Sequence:**

1. **Synthesis Started:**
```json
{
  "type": "synthesis_started"
}
```

2. **Audio Chunks (one per sentence):**
```json
{
  "type": "audio_chunk",
  "chunk": "UklGRiQAAABXQVZFZm10IBAAAAABAAEARKwAAIhY...",
  "index": 0,
  "total": 3,
  "sample_rate": 24000
}
```

- `chunk`: Base64-encoded WAV audio data (16-bit PCM, mono)
- `index`: Zero-based index of this chunk
- `total`: Total number of chunks for this synthesis
- `sample_rate`: Audio sample rate in Hz (always 24000)

3. **Synthesis Completed:**
```json
{
  "type": "synthesis_completed"
}
```

**Error Response:**
```json
{
  "type": "error"
}
```

## Audio Format

Audio chunks are delivered as complete WAV files encoded in Base64. Each chunk corresponds to one sentence from the input text.

**Format Specifications:**
- **Encoding:** 16-bit PCM
- **Sample Rate:** 24000 Hz
- **Channels:** Mono (1 channel)
- **Container:** WAV with standard RIFF headers
- **Transfer:** Base64-encoded string

To decode and play audio chunks:

1. Decode the Base64 string to binary data
2. The result is a complete WAV file ready for playback
3. Play chunks sequentially for natural speech flow

## Streaming Behavior

The server processes text in sentences and streams audio chunks as they are generated:

1. Text is split into sentences using natural language segmentation
2. Each sentence is synthesized independently
3. Audio chunks are sent immediately upon generation
4. Clients can begin playback before all chunks arrive
5. Sequential playback of chunks provides natural continuous speech

## Example Client Implementation

### JavaScript/Browser

```javascript
const socket = new WebSocket('ws://localhost:8080');

socket.onopen = () => {
  // List available voices
  socket.send(JSON.stringify({ command: 'list_voices' }));
  
  // Set voice
  socket.send(JSON.stringify({ 
    command: 'set_voice', 
    voice: 'af_sky' 
  }));
  
  // Set speed
  socket.send(JSON.stringify({ 
    command: 'set_speed', 
    speed: 1.2 
  }));
  
  // Synthesize speech (uses current speed)
  socket.send(JSON.stringify({ 
    command: 'synthesize', 
    text: 'Hello world! This is a streaming test.' 
  }));
  
  // Synthesize with custom speed for this request only
  socket.send(JSON.stringify({ 
    command: 'synthesize', 
    text: 'This sentence will be faster!',
    speed: 1.8
  }));
};

socket.onmessage = (event) => {
  const data = JSON.parse(event.data);
  
  switch (data.type) {
    case 'voices':
      console.log('Available voices:', data.voices);
      console.log('Current voice:', data.voice);
      break;
      
    case 'speed_changed':
      console.log('Speed changed successfully');
      break;
      
    case 'audio_chunk':
      // Decode Base64 to binary
      const audioData = atob(data.chunk);
      const audioArray = new Uint8Array(audioData.length);
      for (let i = 0; i < audioData.length; i++) {
        audioArray[i] = audioData.charCodeAt(i);
      }
      
      // Create audio blob and play
      const audioBlob = new Blob([audioArray], { type: 'audio/wav' });
      const audioUrl = URL.createObjectURL(audioBlob);
      const audio = new Audio(audioUrl);
      audio.play();
      
      console.log(`Chunk ${data.index + 1}/${data.total} received`);
      break;
      
    case 'synthesis_completed':
      console.log('Synthesis complete!');
      break;
      
    case 'error':
      console.error('Server error');
      break;
  }
};

socket.onerror = (error) => {
  console.error('WebSocket error:', error);
};

socket.onclose = () => {
  console.log('Connection closed');
};
```

### Python

```python
import asyncio
import websockets
import json
import base64
import wave

async def tts_client():
    uri = "ws://localhost:8080"
    
    async with websockets.connect(uri) as websocket:
        # List voices
        await websocket.send(json.dumps({"command": "list_voices"}))
        response = json.loads(await websocket.recv())
        print(f"Available voices: {response['voices']}")
        
        # Set voice
        await websocket.send(json.dumps({
            "command": "set_voice",
            "voice": "af_sky"
        }))
        
        # Set speed
        await websocket.send(json.dumps({
            "command": "set_speed",
            "speed": 1.2
        }))
        
        # Synthesize speech (uses current speed)
        await websocket.send(json.dumps({
            "command": "synthesize",
            "text": "Hello world! This is a test of the WebSocket API."
        }))
        
        # Collect audio chunks
        chunks = []
        while True:
            message = json.loads(await websocket.recv())
            
            if message['type'] == 'audio_chunk':
                audio_data = base64.b64decode(message['chunk'])
                chunks.append(audio_data)
                print(f"Received chunk {message['index'] + 1}/{message['total']}")
                
            elif message['type'] == 'synthesis_completed':
                print("Synthesis completed!")
                break
                
            elif message['type'] == 'error':
                print("Error occurred")
                break
        
        # Save combined audio (optional)
        # Each chunk is a complete WAV file, so you can save them individually
        # or extract PCM data and combine them
        for i, chunk_data in enumerate(chunks):
            with open(f'output_{i}.wav', 'wb') as f:
                f.write(chunk_data)

asyncio.run(tts_client())
```

## Testing

A complete test client is provided in `websocket_test.html`. Open this file in a web browser to:

- Connect to the WebSocket server
- List and select voices
- Synthesize and play speech in real-time
- View detailed connection logs

Start the server and open the HTML file in your browser to test the API interactively.

## Error Handling

The server sends `{"type": "error"}` for various error conditions:

- Invalid JSON in client message
- Unknown command
- Invalid voice ID in `set_voice`
- Synthesis failure in `synthesize`

Clients should handle these errors gracefully and may reconnect if necessary.

## Connection Lifecycle

1. Client connects via WebSocket
2. Server accepts connection and spawns handler
3. Client sends commands, server responds
4. Connection persists for multiple commands
5. Either party can close the connection
6. Server cleans up resources on disconnect

Multiple clients can connect simultaneously, each with independent voice, language, and speed settings and synthesis requests.

## Performance Considerations

- Audio chunks are sent as generated (streaming)
- First chunk typically arrives within 100-500ms
- Chunk size depends on sentence length
- No buffering required on server side
- Client should queue chunks for sequential playback
- Base64 encoding adds ~33% size overhead
- Speed changes do not affect audio quality, only playback rate

## Implementation Reference

The WebSocket implementation is located in:
- `kokorox-websocket/src/lib.rs` - Server implementation
- `websocket_test.html` - Reference client implementation
