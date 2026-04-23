/**
 * UploadManager - Handles resumable chunked uploads with progress tracking
 *
 * Uses a tus-like protocol for reliable uploads:
 * - Files are split into chunks
 * - Progress is tracked server-side
 * - Network failures can be resumed
 * - Checksum verification on completion
 */

export interface UploadProgress {
  uploadId: string;
  filename: string;
  totalSize: number;
  uploadedSize: number;
  progress: number; // 0-100
  status: 'pending' | 'uploading' | 'complete' | 'error';
  error?: string;
}

export interface UploadOptions {
  attachmentType: 'bead' | 'stitch';
  resourceId: string;
  chunkSize?: number; // bytes, default 5MB
  onProgress?: (progress: UploadProgress) => void;
  onComplete?: (uploadId: string, path: string) => void;
  onError?: (uploadId: string, error: Error) => void;
}

const DEFAULT_CHUNK_SIZE = 5 * 1024 * 1024; // 5MB

/**
 * Compute SHA-256 checksum of a file
 */
async function computeChecksum(file: File): Promise<string> {
  const buffer = await file.arrayBuffer();
  const hashBuffer = await crypto.subtle.digest('SHA-256', buffer);
  const hashArray = Array.from(new Uint8Array(hashBuffer));
  return hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
}

/**
 * Resume an existing upload from server state
 */
async function resumeUpload(uploadId: string): Promise<{ offset: number; totalSize: number } | null> {
  try {
    const response = await fetch(`/api/uploads/${uploadId}`, {
      method: 'HEAD',
    });
    if (!response.ok) return null;

    const offset = response.headers.get('Upload-Offset');
    const totalSize = response.headers.get('Upload-Length');

    if (offset && totalSize) {
      return {
        offset: parseInt(offset, 10),
        totalSize: parseInt(totalSize, 10),
      };
    }
    return null;
  } catch {
    return null;
  }
}

/**
 * UploadManager class - manages a single file upload
 */
export class UploadManager {
  private file: File;
  private options: UploadOptions;
  private uploadId: string | null = null;
  private abortController: AbortController | null = null;
  private chunkSize: number;

  // Resume state (loaded from localStorage)
  private static STORAGE_PREFIX = 'hoop_upload_';

  constructor(file: File, options: UploadOptions) {
    this.file = file;
    this.options = options;
    this.chunkSize = options.chunkSize || DEFAULT_CHUNK_SIZE;
  }

  /**
   * Get localStorage key for this upload
   */
  private getStorageKey(filename: string): string {
    return UploadManager.STORAGE_PREFIX + filename;
  }

  /**
   * Save upload state to localStorage for resume capability
   */
  private saveUploadState(state: { uploadId: string; filename: string; resourceId: string }): void {
    try {
      localStorage.setItem(this.getStorageKey(state.filename), JSON.stringify(state));
    } catch {
      // Ignore storage errors (e.g., quota exceeded)
    }
  }

  /**
   * Load and clear upload state from localStorage
   */
  private loadUploadState(filename: string): { uploadId: string; resourceId: string } | null {
    try {
      const key = this.getStorageKey(filename);
      const data = localStorage.getItem(key);
      if (data) {
        localStorage.removeItem(key);
        return JSON.parse(data);
      }
    } catch {
      // Ignore
    }
    return null;
  }

  /**
   * Start or resume the upload
   */
  async start(): Promise<string> {
    this.abortController = new AbortController();

    // Check for existing upload to resume
    const savedState = this.loadUploadState(this.file.name);
    let startOffset = 0;

    if (savedState && savedState.resourceId === this.options.resourceId) {
      const resumeState = await resumeUpload(savedState.uploadId);
      if (resumeState && resumeState.totalSize === this.file.size) {
        this.uploadId = savedState.uploadId;
        startOffset = resumeState.offset;
        console.log(`Resuming upload from byte ${startOffset}`);
      }
    }

    // If no existing upload, initiate new one
    if (!this.uploadId) {
      const checksum = await computeChecksum(this.file);

      const initResponse = await fetch('/api/uploads', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          filename: this.file.name,
          total_size: this.file.size,
          checksum: checksum,
          attachment_type: this.options.attachmentType,
          resource_id: this.options.resourceId,
        }),
        signal: this.abortController.signal,
      });

      if (!initResponse.ok) {
        throw new Error(`Failed to initiate upload: ${initResponse.statusText}`);
      }

      const initData = await initResponse.json();
      this.uploadId = initData.upload_id ?? null;
      if (this.uploadId !== null) {
        this.saveUploadState({
          uploadId: this.uploadId,
          filename: this.file.name,
          resourceId: this.options.resourceId ?? null,
        });
      }
    }

    // Start uploading chunks
    this.uploadChunks(startOffset);

    if (!this.uploadId) {
      throw new Error('Upload ID not available after initiation');
    }
    return this.uploadId;
  }

  /**
   * Upload file in chunks, starting from the given offset
   */
  private async uploadChunks(startOffset: number): Promise<void> {
    if (!this.uploadId) throw new Error('Upload not initiated');

    const totalSize = this.file.size;
    let offset = startOffset;

    this.reportProgress(offset, totalSize, 'uploading');

    try {
      while (offset < totalSize) {
        if (this.abortController?.signal.aborted) {
          throw new Error('Upload cancelled');
        }

        const chunk = this.file.slice(offset, Math.min(offset + this.chunkSize, totalSize));

        const response = await fetch(`/api/uploads/${this.uploadId}`, {
          method: 'PATCH',
          headers: {
            'Upload-Offset': offset.toString(),
            'Content-Type': 'application/octet-stream',
          },
          body: chunk,
          signal: this.abortController?.signal,
        });

        if (!response.ok) {
          throw new Error(`Chunk upload failed: ${response.statusText}`);
        }

        const progressData = await response.json();
        offset = progressData.offset;

        this.reportProgress(offset, totalSize, 'uploading');

        // Save state for resume
        this.saveUploadState({
          uploadId: this.uploadId,
          filename: this.file.name,
          resourceId: this.options.resourceId,
        });
      }

      // All chunks uploaded - complete the upload
      await this.completeUpload();

    } catch (error) {
      const err = error instanceof Error ? error : new Error('Upload failed');
      this.reportProgress(offset, totalSize, 'error', err.message);
      this.options.onError?.(this.uploadId, err);
      throw error;
    }
  }

  /**
   * Complete the upload and verify checksum
   */
  private async completeUpload(): Promise<void> {
    if (!this.uploadId) throw new Error('Upload not initiated');

    const response = await fetch(`/api/uploads/${this.uploadId}/complete`, {
      method: 'POST',
      signal: this.abortController?.signal,
    });

    if (!response.ok) {
      const errorText = await response.text();
      throw new Error(`Failed to complete upload: ${errorText}`);
    }

    this.reportProgress(this.file.size, this.file.size, 'complete');
    this.options.onComplete?.(this.uploadId, this.file.name);

    // Clear saved state
    try {
      localStorage.removeItem(this.getStorageKey(this.file.name));
    } catch {
      // Ignore
    }
  }

  /**
   * Report progress to callback
   */
  private reportProgress(uploadedSize: number, totalSize: number, status: UploadProgress['status'], error?: string): void {
    if (!this.uploadId) return;

    this.options.onProgress?.({
      uploadId: this.uploadId,
      filename: this.file.name,
      totalSize,
      uploadedSize,
      progress: (uploadedSize / totalSize) * 100,
      status,
      error,
    });
  }

  /**
   * Cancel the upload
   */
  async cancel(): Promise<void> {
    this.abortController?.abort();

    if (this.uploadId) {
      try {
        await fetch(`/api/uploads/${this.uploadId}`, {
          method: 'DELETE',
        });
      } catch {
        // Ignore cleanup errors
      }

      // Clear saved state
      try {
        localStorage.removeItem(this.getStorageKey(this.file.name));
      } catch {
        // Ignore
      }
    }
  }

  /**
   * Pause the upload (keeps server state for resume)
   */
  pause(): void {
    this.abortController?.abort();
  }
}

/**
 * Format bytes to human-readable size
 */
export function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
}

/**
 * Format progress percentage
 */
export function formatProgress(progress: number): string {
  return `${progress.toFixed(1)}%`;
}
