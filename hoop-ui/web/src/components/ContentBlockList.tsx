import { useState, useEffect, useCallback } from 'react';
import ContentBlockRenderer, { ContentBlock, ContentBlockMetadata } from './ContentBlockRenderer';

interface ContentBlockListProps {
  stitchId: string;
  readOnly?: boolean;
  onBlocksChange?: (blocks: ContentBlock[]) => void;
}

export default function ContentBlockList({ stitchId, readOnly = false, onBlocksChange }: ContentBlockListProps) {
  const [blocks, setBlocks] = useState<ContentBlock[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchBlocks = useCallback(async () => {
    if (!stitchId) return;
    setIsLoading(true);
    setError(null);

    try {
      const resp = await fetch(`/api/stitches/${stitchId}/content-blocks`);
      if (!resp.ok) {
        if (resp.status === 404) {
          setBlocks([]);
        } else {
          throw new Error(`Failed to fetch content blocks: ${resp.statusText}`);
        }
      } else {
        const data = await resp.json();
        setBlocks(data);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
      console.error('Failed to fetch content blocks:', err);
    } finally {
      setIsLoading(false);
    }
  }, [stitchId]);

  useEffect(() => {
    fetchBlocks();
  }, [fetchBlocks]);

  const handleDelete = useCallback(async (blockId: string) => {
    if (readOnly) return;

    try {
      const resp = await fetch(`/api/stitches/${stitchId}/content-blocks/${blockId}`, {
        method: 'DELETE',
      });

      if (!resp.ok) {
        throw new Error(`Failed to delete content block: ${resp.statusText}`);
      }

      setBlocks(prev => prev.filter(b => b.id !== blockId));
      onBlocksChange?.(blocks.filter(b => b.id !== blockId));
    } catch (err) {
      console.error('Failed to delete content block:', err);
      setError(err instanceof Error ? err.message : 'Failed to delete block');
    }
  }, [stitchId, readOnly, blocks, onBlocksChange]);

  if (isLoading) {
    return (
      <div className="content-block-list content-block-list--loading">
        <span className="content-block-loading-spinner" />
        <span>Loading content blocks...</span>
      </div>
    );
  }

  if (error) {
    return (
      <div className="content-block-list content-block-list--error">
        <span className="content-block-error-icon">⚠️</span>
        <span>{error}</span>
        <button onClick={fetchBlocks} className="content-block-retry-btn">Retry</button>
      </div>
    );
  }

  if (blocks.length === 0) {
    return null;
  }

  return (
    <div className="content-block-list">
      <h4 className="content-block-list-title">Content</h4>
      <div className="content-block-list-items">
        {blocks.map(block => (
          <ContentBlockRenderer
            key={block.id}
            block={block}
            stitchId={stitchId}
            onDelete={readOnly ? undefined : handleDelete}
            readOnly={readOnly}
          />
        ))}
      </div>
    </div>
  );
}
