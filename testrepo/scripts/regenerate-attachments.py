#!/usr/bin/env python3
"""Regenerate attachment fixtures for testrepo."""

import struct
import os
import json

TESTREPO_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

def create_minimal_png(path: str, width: int = 8, height: int = 8) -> None:
    """Create a minimal valid PNG file."""
    # PNG signature
    png_data = b'\x89PNG\r\n\x1a\n'
    
    # IHDR chunk (image header)
    ihdr = struct.pack('>IIBBBBB', width, height, 8, 2, 0, 0, 0)  # 8-bit RGB
    ihdr_chunk = b'IHDR' + ihdr
    ihdr_chunk += struct.pack('>I', 0x356c3a8f)  # CRC (simplified)
    png_data += struct.pack('>I', len(ihdr)) + ihdr_chunk
    
    # IDAT chunk (image data - minimal)
    idat_data = b'x\x9cc\xf8\xcf\xc0\x00\x00\x00\x03\x00\x01\x00\x00\x00\x00IEND\xaeB`\x82'
    idat_chunk = b'IDAT' + idat_data
    png_data += struct.pack('>I', len(idat_data)) + idat_chunk
    
    # IEND chunk
    iend = b'IEND\xaeB`\x82'
    png_data += struct.pack('>I', 0) + iend
    
    with open(path, 'wb') as f:
        f.write(png_data)

def create_minimal_wav(path: str, duration_sec: float = 1.0, sample_rate: int = 22050) -> None:
    """Create a minimal valid WAV file (PCM, mono, silence)."""
    num_samples = int(duration_sec * sample_rate)
    data_size = num_samples * 2  # 16-bit samples
    
    with open(path, 'wb') as f:
        # RIFF header
        f.write(b'RIFF')
        f.write(struct.pack('<I', 36 + data_size))
        f.write(b'WAVE')
        
        # fmt chunk
        f.write(b'fmt ')
        f.write(struct.pack('<I', 16))  # chunk size
        f.write(struct.pack('<H', 1))   # audio format (PCM)
        f.write(struct.pack('<H', 1))   # channels
        f.write(struct.pack('<I', sample_rate))
        f.write(struct.pack('<I', sample_rate * 2))  # byte rate
        f.write(struct.pack('<H', 2))   # block align
        f.write(struct.pack('<H', 16))  # bits per sample
        
        # data chunk
        f.write(b'data')
        f.write(struct.pack('<I', data_size))
        f.write(b'\x00\x00' * num_samples)

def create_minimal_mp4(path: str) -> None:
    """Create a minimal valid MP4 container for testing."""
    # Minimal ISO BMFF structure
    mp4_data = (
        b'\x00\x00\x00\x20\x66\x74\x79\x70\x69\x73\x6f\x6d'  # ftyp iso
        b'\x00\x00\x00\x01\x00\x00\x00\x01\x00\x00\x00\x00'
        b'\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00'
        b'\x00\x00\x00\x18\x6d\x6f\x6f\x76\x00\x00\x00\x6c'  # moov
        b'\x6d\x76\x68\x64\x00\x00\x00\x00\x00\x00\x00\x00'
        b'\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00'
        b'\x00\x00\x03\xe8\x00\x00\x00\x00\x00\x01\x00\x00'
        b'\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00'
        b'\x00\x00\x00\x00\x00\x00\x00\x01\x00\x00\x00\x00'
    )
    with open(path, 'wb') as f:
        f.write(mp4_data)

def create_attachment_metadata(bead_id: str, filename: str, content_type: str, 
                               size_bytes: int, purpose: str, **extra) -> dict:
    """Create metadata for an attachment."""
    metadata = {
        "filename": filename,
        "content_type": content_type,
        "size_bytes": size_bytes,
        "uploaded_at": "2026-04-21T18:45:00Z",
        "uploaded_by": "alpha",
        "purpose": purpose,
        **extra
    }
    return metadata

def main():
    # Create attachment directories
    attachments_dir = os.path.join(TESTREPO_ROOT, '.beads', 'attachments')
    
    # Bead tr-open-001 attachments (image, audio, video)
    open_dir = os.path.join(attachments_dir, 'tr-open-001')
    os.makedirs(open_dir, exist_ok=True)
    
    # Screenshot PNG
    screenshot_path = os.path.join(open_dir, 'screenshot.png')
    create_minimal_png(screenshot_path, 8, 8)
    with open(screenshot_path + '.meta.json', 'w') as f:
        json.dump(create_attachment_metadata(
            'tr-open-001', 'screenshot.png', 'image/png', 69,
            'Screenshot showing memory profiler output',
            resolution='8x8',
            description='Memory leak visible in parser tokenization'
        ), f, indent=2)
    
    # Audio WAV
    audio_path = os.path.join(open_dir, 'audio_message.wav')
    create_minimal_wav(audio_path, 1.0, 22050)
    with open(audio_path + '.meta.json', 'w') as f:
        json.dump(create_attachment_metadata(
            'tr-open-001', 'audio_message.wav', 'audio/wav', 44124,
            'Voice message from agent about parser memory leak analysis',
            duration_seconds=1.0,
            transcription='I\'ve identified the leak in the tokenizer.'
        ), f, indent=2)
    
    # Video MP4
    video_path = os.path.join(open_dir, 'demo_video.mp4')
    create_minimal_mp4(video_path)
    with open(video_path + '.meta.json', 'w') as f:
        json.dump(create_attachment_metadata(
            'tr-open-001', 'demo_video.mp4', 'video/mp4', 120,
            'Screen recording showing the leak in action',
            duration_seconds=0.033,
            resolution='1x1',
            description='Minimal demo video for testing'
        ), f, indent=2)
    
    # Bead tr-closed-002 attachments (error log)
    closed_dir = os.path.join(attachments_dir, 'tr-closed-002')
    os.makedirs(closed_dir, exist_ok=True)
    
    error_log_path = os.path.join(closed_dir, 'error_log.txt')
    with open(error_log_path, 'w') as f:
        f.write('[ERROR] 2026-04-21T18:30:00Z - Test suite failed\n')
        f.write('  at tests/integration/test_004.rs:42\n')
        f.write('  assertion failed: `expected == actual`\n')
    with open(error_log_path + '.meta.json', 'w') as f:
        json.dump(create_attachment_metadata(
            'tr-closed-002', 'error_log.txt', 'text/plain', 151,
            'Error log from failed test run',
            line_count=3
        ), f, indent=2)
    
    # Bead tr-failed-001 attachments (metrics)
    failed_dir = os.path.join(attachments_dir, 'tr-failed-001')
    os.makedirs(failed_dir, exist_ok=True)
    
    metrics_path = os.path.join(failed_dir, 'metrics.json')
    with open(metrics_path, 'w') as f:
        json.dump({
            "memory_mb": 2048,
            "cpu_percent": 95,
            "duration_ms": 120000,
            "tokens_used": 180000,
            "error": "context limit exceeded"
        }, f)
    with open(metrics_path + '.meta.json', 'w') as f:
        json.dump(create_attachment_metadata(
            'tr-failed-001', 'metrics.json', 'application/json', 85,
            'Performance metrics from failed task'
        ), f, indent=2)
    
    print("Created attachment fixtures:")
    print(f"  - {screenshot_path}")
    print(f"  - {audio_path}")
    print(f"  - {video_path}")
    print(f"  - {error_log_path}")
    print(f"  - {metrics_path}")

if __name__ == '__main__':
    main()
