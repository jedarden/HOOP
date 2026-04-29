import { useMemo } from 'react';
import AudioPlayer from './AudioPlayer';
import VideoPlayer from './VideoPlayer';

export interface ContentBlock {
  id: string;
  stitch_id: string;
  block_type: 'text' | 'image' | 'audio' | 'video' | 'file';
  content: string | null;
  metadata: ContentBlockMetadata | null;
  block_order: number;
  created_at: string;
}

export interface ContentBlockMetadata {
  filename?: string;
  content_type?: string;
  size_bytes?: number;
  duration_secs?: number;
  width?: number;
  height?: number;
  transcript_words?: Array<{ word: string; start: number; end: number }>;
  frame_samples?: Array<{ timestamp_secs: number; label: string; thumbnail?: string }>;
}

interface ContentBlockRendererProps {
  block: ContentBlock;
  stitchId: string;
  onEdit?: (block: ContentBlock) => void;
  onDelete?: (blockId: string) => void;
  readOnly?: boolean;
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}

function getAttachmentUrl(block: ContentBlock): string {
  return `/api/attachments/stitch/${block.stitch_id}/${block.metadata?.filename || ''}`;
}

export default function ContentBlockRenderer({
  block,
  stitchId,
  onEdit,
  onDelete,
  readOnly = false,
}: ContentBlockRendererProps) {
  const blockActions = useMemo(() => {
    if (readOnly) return null;
    return (
      <div className="content-block-actions">
        {onEdit && (
          <button
            className="content-block-btn content-block-btn-edit"
            onClick={() => onEdit(block)}
            aria-label="Edit block"
            title="Edit"
          >
            ✏️
          </button>
        )}
        {onDelete && (
          <button
            className="content-block-btn content-block-btn-delete"
            onClick={() => onDelete(block.id)}
            aria-label="Delete block"
            title="Delete"
          >
            🗑️
          </button>
        )}
      </div>
    );
  }, [block, onEdit, onDelete, readOnly]);

  const renderContent = () => {
    switch (block.block_type) {
      case 'text':
        return (
          <div className="content-block-text">
            <pre className="content-block-text-content">{block.content || ''}</pre>
          </div>
        );

      case 'image': {
        const imageUrl = getAttachmentUrl(block);
        const metadata = block.metadata;
        return (
          <div className="content-block-image">
            <img
              src={imageUrl}
              alt={metadata?.filename || 'Image attachment'}
              className="content-block-image-element"
              loading="lazy"
            />
            {metadata && (metadata.width || metadata.height) && (
              <div className="content-block-meta">
                {metadata.width && <span>{metadata.width}px</span>}
                {metadata.width && metadata.height && <span> × </span>}
                {metadata.height && <span>{metadata.height}px</span>}
                {metadata.size_bytes && <span> · {formatBytes(metadata.size_bytes)}</span>}
              </div>
            )}
          </div>
        );
      }

      case 'audio': {
        const audioUrl = getAttachmentUrl(block);
        const metadata = block.metadata;
        const transcript = metadata?.transcript_words && metadata.transcript_words.length > 0
          ? {
              text: block.content || '',
              words: metadata.transcript_words,
            }
          : undefined;
        return (
          <div className="content-block-audio">
            <AudioPlayer audioUrl={audioUrl} transcript={transcript} />
            {metadata && metadata.duration_secs && (
              <div className="content-block-meta">
                <span>Duration: {Math.floor(metadata.duration_secs / 60)}:{(metadata.duration_secs % 60).toString().padStart(2, '0')}</span>
                {metadata.size_bytes && <span> · {formatBytes(metadata.size_bytes)}</span>}
              </div>
            )}
          </div>
        );
      }

      case 'video': {
        const videoUrl = getAttachmentUrl(block);
        const metadata = block.metadata;
        const transcript = metadata?.transcript_words && metadata.transcript_words.length > 0
          ? {
              text: block.content || '',
              words: metadata.transcript_words,
            }
          : undefined;
        return (
          <div className="content-block-video">
            <VideoPlayer
              videoUrl={videoUrl}
              chapters={metadata?.frame_samples}
              transcript={transcript}
            />
            {metadata && metadata.duration_secs && (
              <div className="content-block-meta">
                <span>Duration: {Math.floor(metadata.duration_secs / 60)}:{(Math.floor(metadata.duration_secs) % 60).toString().padStart(2, '0')}</span>
                {metadata.size_bytes && <span> · {formatBytes(metadata.size_bytes)}</span>}
              </div>
            )}
          </div>
        );
      }

      case 'file': {
        const fileUrl = getAttachmentUrl(block);
        const metadata = block.metadata;
        return (
          <div className="content-block-file">
            <a
              href={fileUrl}
              download={metadata?.filename || 'download'}
              className="content-block-file-link"
            >
              <span className="content-block-file-icon">📄</span>
              <span className="content-block-file-name">{metadata?.filename || 'Unnamed file'}</span>
              {metadata?.size_bytes && (
                <span className="content-block-file-size">({formatBytes(metadata.size_bytes)})</span>
              )}
            </a>
            {metadata?.content_type && (
              <div className="content-block-meta">
                <span className="content-block-mime-type">{metadata.content_type}</span>
              </div>
            )}
          </div>
        );
      }

      default:
        return (
          <div className="content-block-unknown">
            <em>Unknown block type: {block.block_type}</em>
          </div>
        );
    }
  };

  return (
    <div className={`content-block content-block-${block.block_type}`}>
      {blockActions}
      {renderContent()}
    </div>
  );
}
