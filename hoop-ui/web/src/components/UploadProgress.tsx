/**
 * UploadProgress - React component for displaying upload progress
 *
 * Shows progress bar, uploaded/total size, and status for active uploads.
 */

import React, { useState, useRef } from 'react';
import { UploadManager, UploadProgress, formatBytes, formatProgress } from './UploadManager';

interface UploadProgressItemProps {
  progress: UploadProgress;
  onCancel?: (uploadId: string) => void;
}

export const UploadProgressItem: React.FC<UploadProgressItemProps> = ({ progress, onCancel }) => {
  const isComplete = progress.status === 'complete';
  const isError = progress.status === 'error';

  return (
    <div className={`upload-item upload-item-${progress.status}`}>
      <div className="upload-header">
        <span className="upload-filename">{progress.filename}</span>
        <span className="upload-status">
          {isComplete ? '✓ Complete' : isError ? '✗ Failed' : `${formatProgress(progress.progress)}`}
        </span>
      </div>

      {!isComplete && !isError && (
        <div className="upload-progress-bar">
          <div
            className="upload-progress-fill"
            style={{ width: `${progress.progress}%` }}
          />
        </div>
      )}

      <div className="upload-details">
        <span>{formatBytes(progress.uploadedSize)} / {formatBytes(progress.totalSize)}</span>
        {isError && progress.error && (
          <span className="upload-error">{progress.error}</span>
        )}
      </div>

      {!isComplete && onCancel && (
        <button
          className="upload-cancel-btn"
          onClick={() => onCancel(progress.uploadId)}
          disabled={isError}
        >
          Cancel
        </button>
      )}
    </div>
  );
};

interface UploadProgressPanelProps {
  uploads: UploadProgress[];
  onCancel?: (uploadId: string) => void;
}

export const UploadProgressPanel: React.FC<UploadProgressPanelProps> = ({ uploads, onCancel }) => {
  if (uploads.length === 0) return null;

  return (
    <div className="upload-progress-panel">
      <h3>Uploads ({uploads.length})</h3>
      <div className="upload-list">
        {uploads.map((upload) => (
          <UploadProgressItem
            key={upload.uploadId}
            progress={upload}
            onCancel={onCancel}
          />
        ))}
      </div>
    </div>
  );
};

interface UseUploadsResult {
  uploads: UploadProgress[];
  startUpload: (file: File, options: UploadManager['options']) => Promise<string>;
  cancelUpload: (uploadId: string) => Promise<void>;
  clearCompleted: () => void;
}

/**
 * React hook for managing uploads
 */
export function useUploads(): UseUploadsResult {
  const [uploads, setUploads] = useState<UploadProgress[]>([]);
  const managersRef = useRef<Map<string, UploadManager>>(new Map());

  const startUpload = async (file: File, options: UploadManager['options']): Promise<string> => {
    const manager = new UploadManager(file, {
      ...options,
      onProgress: (progress) => {
        setUploads((prev) => {
          const existing = prev.findIndex((u) => u.uploadId === progress.uploadId);
          if (existing >= 0) {
            const updated = [...prev];
            updated[existing] = progress;
            return updated;
          }
          return [...prev, progress];
        });
      },
      onComplete: (uploadId) => {
        setUploads((prev) =>
          prev.map((u) =>
            u.uploadId === uploadId ? { ...u, status: 'complete' as const } : u
          )
        );
        managersRef.current.delete(uploadId);
      },
      onError: (uploadId, error) => {
        setUploads((prev) =>
          prev.map((u) =>
            u.uploadId === uploadId
              ? { ...u, status: 'error' as const, error: error.message }
              : u
          )
        );
        managersRef.current.delete(uploadId);
      },
    });

    const uploadId = await manager.start();
    managersRef.current.set(uploadId, manager);
    return uploadId;
  };

  const cancelUpload = async (uploadId: string): Promise<void> => {
    const manager = managersRef.current.get(uploadId);
    if (manager) {
      await manager.cancel();
      managersRef.current.delete(uploadId);
      setUploads((prev) => prev.filter((u) => u.uploadId !== uploadId));
    }
  };

  const clearCompleted = () => {
    setUploads((prev) => prev.filter((u) => u.status !== 'complete'));
  };

  return {
    uploads,
    startUpload,
    cancelUpload,
    clearCompleted,
  };
}
