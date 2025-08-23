/// Audio trimming functionality to remove leading and trailing silence
/// Based on the librosa trim implementation from the original kokoro-onnx

pub fn trim_audio(audio: &[f32], top_db: f32) -> Vec<f32> {
    if audio.is_empty() {
        return audio.to_vec();
    }
    
    let frame_length = 2048;
    let hop_length = 512;
    
    // Compute RMS for each frame
    let rms_values = compute_rms(audio, frame_length, hop_length);
    
    if rms_values.is_empty() {
        return audio.to_vec();
    }
    
    // Find the reference level (maximum RMS)
    let max_rms = rms_values.iter().fold(0.0f32, |max, &val| max.max(val));
    
    if max_rms == 0.0 {
        return audio.to_vec();
    }
    
    // Convert to dB and find non-silent frames
    let threshold_linear = max_rms * 10.0f32.powf(-top_db / 20.0);
    
    let mut non_silent_frames = Vec::new();
    for (i, &rms) in rms_values.iter().enumerate() {
        if rms > threshold_linear {
            non_silent_frames.push(i);
        }
    }
    
    if non_silent_frames.is_empty() {
        return audio.to_vec();
    }
    
    // Convert frame indices back to sample indices
    let start_sample = non_silent_frames[0] * hop_length;
    let end_sample = ((non_silent_frames[non_silent_frames.len() - 1] + 1) * hop_length)
        .min(audio.len());
    
    audio[start_sample..end_sample].to_vec()
}

fn compute_rms(audio: &[f32], frame_length: usize, hop_length: usize) -> Vec<f32> {
    let mut rms_values = Vec::new();
    
    let mut frame_start = 0;
    while frame_start + frame_length <= audio.len() {
        let frame_end = frame_start + frame_length;
        let frame = &audio[frame_start..frame_end];
        
        // Compute RMS for this frame
        let sum_squares: f32 = frame.iter().map(|&x| x * x).sum();
        let rms = (sum_squares / frame_length as f32).sqrt();
        
        rms_values.push(rms);
        frame_start += hop_length;
    }
    
    rms_values
}