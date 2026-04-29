# Phase 3 Implementation Plan: File Browser + Artifact Preview + Multimodal (v0.3)

## Overview

This plan implements Phase 3 of HOOP: enhanced file browser, multimodal artifact preview, and voice capture capabilities for dictation and screen recording.

## Current State Analysis

### Already Implemented
- **File Browser** (`FilesTab.tsx`): Tree navigation, git status, search, syntax highlighting
- **Content Blocks API** (`api_content_blocks.rs`): CRUD for stitch-associated content
- **Upload System** (`api_uploads.rs`): Resumable chunked uploads with progress tracking
- **WebSocket** (`ws.rs`): Topic-based pub/sub with 23+ broadcast channels
- **State Management** (`atoms.ts`): Jotai atoms with streaming isolation

### What Needs to Be Built

## Implementation Tasks

### 1. Audio/Video Preview Components

**Backend**: Add serving endpoint for audio/video files
- Extend `api_attachments.rs` to serve audio/video with proper MIME types
- Add range request support for large media files

**Frontend**: Create `AudioViewer.tsx` and `VideoViewer.tsx`
- HTML5 `<audio>` and `<video>` elements with controls
- Waveform visualization for audio (optional)
- Thumbnail preview for video

**Files**:
- `hoop-ui/web/src/AudioViewer.tsx` (new)
- `hoop-ui/web/src/VideoViewer.tsx` (new)

### 2. File Drag-Drop into Stitch Drafts

**Backend**: Add attachment link API
- `POST /api/stitches/:stitch_id/attachments` - link uploaded file to stitch
- Update content block metadata with file reference

**Frontend**: Enhance draft forms
- Handle drop events on StitchForm and BeadForm
- Use existing `fileAttachContextAtom` for file → draft piping
- Show attachment preview with file type icon

**Files**:
- `hoop-daemon/src/api_attachments.rs` (extend)
- `hoop-ui/web/src/StitchForm.tsx` (modify)
- `hoop-ui/web/src/BeadForm.tsx` (modify)

### 3. Dictated Notes (Marquee #5)

**Backend**: Dictated notes API
- `POST /api/projects/:project/notes` - create note from audio upload
- `GET /api/projects/:project/notes` - list notes
- `GET /api/projects/:project/notes/:stitch_id` - get note details
- Background Whisper transcription via tokio task
- Store audio in `~/.hoop/notes/{project}/{stitch_id}/audio.{ext}`
- Store transcript in content_blocks table

**Frontend**: DictationWidget enhancement
- Audio recording with MediaRecorder API
- Real-time transcript display via WebSocket
- Note timeline view in project detail
- Mobile-optimized controls (ADB integration)

**WebSocket Events**:
- `dictated_note_created` - new note recorded
- `dictated_note_transcript` - incremental transcript update
- `dictated_note_completed` - transcription finished

**Files**:
- `hoop-daemon/src/api_notes.rs` (new)
- `hoop-daemon/src/whisper.rs` (new - transcription service)
- `hoop-ui/web/src/DictationWidget.tsx` (extend)
- `hoop-ui/web/src/NotesTimeline.tsx` (new)

### 4. Voice/Screen Work Capture (Marquee #6)

**Backend**: Screen capture API
- `POST /api/projects/:project/screen-captures` - initiate capture
- `GET /api/projects/:project/screen-captures` - list captures
- Integrate with ADB for mobile screen recording
- Frame sampling for chapter markers (UI-change detection)
- Stitch draft synthesis from capture

**Frontend**: Capture controls
- Desktop: browser-based capture (getDisplayMedia)
- Mobile: ADB `screenrecord` command
- Chapter marker timeline
- One-click "Create Stitch" from capture

**Files**:
- `hoop-daemon/src/api_screen_captures.rs` (new)
- `hoop-ui/web/src/ScreenCaptureWidget.tsx` (new)

### 5. Multimodal Input to Agent Chat

**Backend**: Content block attachment API
- `POST /api/conversations/:conversation_id/attachments` - attach file to chat
- Convert attachment to content block for context

**Frontend**: AgentChatPane enhancements
- Image paste in textarea
- File attachment button
- Attachment preview in message bubble

**Files**:
- `hoop-ui/web/src/AgentChatPane.tsx` (modify)

### 6. Performance Optimizations

**Backend**:
- Directory listing caching for large repos
- Parallel file system operations
- Git status caching per workspace

**Frontend**:
- Virtual scrolling for file tree (20k+ files)
- Debounced search input
- Lazy loading of viewer components

## Schema Changes

### New Schema: `dictated_note.json`
```json
{
  "type": "object",
  "properties": {
    "stitch_id": { "type": "string" },
    "project": { "type": "string" },
    "audio_filename": { "type": "string" },
    "audio_url": { "type": "string" },
    "transcript": { "type": "string" },
    "transcript_words": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "word": { "type": "string" },
          "start": { "type": "number" },
          "end": { "type": "number" }
        }
      }
    },
    "duration_secs": { "type": "number" },
    "language": { "type": "string" },
    "recorded_at": { "type": "string", "format": "date-time" },
    "transcription_status": { "type": "string", "enum": ["Pending", "Completed", "Failed"] }
  }
}
```

### New Schema: `screen_capture.json`
```json
{
  "type": "object",
  "properties": {
    "stitch_id": { "type": "string" },
    "project": { "type": "string" },
    "video_filename": { "type": "string" },
    "video_url": { "type": "string" },
    "duration_secs": { "type": "number" },
    "recorded_at": { "type": "string", "format": "date-time" },
    "chapters": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "timestamp_secs": { "type": "number" },
          "label": { "type": "string" }
        }
      }
    }
  }
}
```

## Closing Criteria Verification

1. **File browser <1s on 20k files**: Virtual scrolling + caching
2. **Syntax highlighting**: Already working for 10+ languages
3. **Image/audio/video preview**: Add AudioViewer/VideoViewer components
4. **10MB attachment in Stitch**: Test upload → attach → view flow
5. **Voice capture → Note <60s**: Test ADB dictation → transcription flow

## Implementation Order

1. Audio/Video preview components (quick win)
2. File drag-drop into drafts (uses existing upload system)
3. Dictated Notes API (new backend service)
4. DictationWidget UI enhancement
5. Multimodal agent chat input
6. Screen capture (requires ADB integration)
7. Performance optimization
8. Testing and verification

## Dependencies

- Whisper model for transcription (local or API)
- ADB for mobile screen recording
- MediaRecorder API (browser support)
- Range request handling for media serving

## Risk Mitigation

- **Whisper performance**: Use local model for privacy, fallback to API
- **Mobile ADB instability**: Add reconnection logic, status indicators
- **Large file uploads**: Existing resumable upload system handles this
- **Storage growth**: Add cleanup policy for old recordings
