import { useRef, useEffect, useState, useCallback, useMemo } from 'react';
import { BeadData } from './atoms';

interface BeadNode {
  id: string;
  x: number;
  y: number;
  radius: number;
  bead: BeadData;
  dependencies: string[];
  dependents: string[];
}

interface BeadGraphProps {
  beads: BeadData[];
}

const NODE_RADIUS = 8;
const HOVER_RADIUS = 12;
const LEVEL_HEIGHT = 60;
const NODE_SPACING = 40;

export default function BeadGraph({ beads }: BeadGraphProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const [hoveredNode, setHoveredNode] = useState<BeadNode | null>(null);
  const [selectedNode, setSelectedNode] = useState<BeadNode | null>(null);
  const [offset, setOffset] = useState({ x: 0, y: 0 });
  const [scale, setScale] = useState(1);
  const [isDragging, setIsDragging] = useState(false);
  const [dragStart, setDragStart] = useState({ x: 0, y: 0 });

  // Build dependency graph and layout
  const { nodes, edges } = useMemo(() => {
    // Create nodes map
    const nodesMap = new Map<string, BeadNode>();
    const beadMap = new Map(beads.map(b => [b.id, b]));

    // Calculate level for each bead based on dependencies
    const levels = new Map<string, number>();
    const visited = new Set<string>();

    function getLevel(beadId: string): number {
      if (levels.has(beadId)) return levels.get(beadId)!;
      if (visited.has(beadId)) {
        // Circular dependency, assign current level
        return 0;
      }
      visited.add(beadId);

      const bead = beadMap.get(beadId);
      if (!bead || bead.dependencies.length === 0) {
        levels.set(beadId, 0);
        return 0;
      }

      const maxDepLevel = Math.max(...bead.dependencies.map(dep => getLevel(dep)));
      const level = maxDepLevel + 1;
      levels.set(beadId, level);
      return level;
    }

    // Calculate levels for all beads
    beads.forEach(bead => getLevel(bead.id));

    // Group beads by level
    const levelGroups = new Map<number, BeadData[]>();
    beads.forEach(bead => {
      const level = levels.get(bead.id) ?? 0;
      if (!levelGroups.has(level)) {
        levelGroups.set(level, []);
      }
      levelGroups.get(level)!.push(bead);
    });

    // Position nodes
    const maxWidth = Math.max(...Array.from(levelGroups.values()).map(g => g.length));
    const graphWidth = maxWidth * (NODE_SPACING + NODE_RADIUS * 2);
    const graphHeight = levelGroups.size * LEVEL_HEIGHT;

    levelGroups.forEach((beadsAtLevel, level) => {
      const levelY = level * LEVEL_HEIGHT + 50;
      const levelWidth = beadsAtLevel.length * (NODE_SPACING + NODE_RADIUS * 2);
      const startX = (graphWidth - levelWidth) / 2;

      beadsAtLevel.forEach((bead, index) => {
        const x = startX + index * (NODE_SPACING + NODE_RADIUS * 2) + NODE_RADIUS;
        const y = levelY;
        nodesMap.set(bead.id, {
          id: bead.id,
          x,
          y,
          radius: NODE_RADIUS,
          bead,
          dependencies: bead.dependencies,
          dependents: [],
        });
      });
    });

    // Build dependents lists
    nodesMap.forEach(node => {
      node.dependencies.forEach(depId => {
        const depNode = nodesMap.get(depId);
        if (depNode) {
          depNode.dependents.push(node.id);
        }
      });
    });

    // Build edges
    const edges: [string, string][] = [];
    nodesMap.forEach(node => {
      node.dependencies.forEach(depId => {
        if (nodesMap.has(depId)) {
          edges.push([depId, node.id]);
        }
      });
    });

    return {
      nodes: Array.from(nodesMap.values()),
      edges,
      graphWidth,
      graphHeight,
    };
  }, [beads]);

  // Handle canvas resize
  useEffect(() => {
    const canvas = canvasRef.current;
    const container = containerRef.current;
    if (!canvas || !container) return;

    const resizeCanvas = () => {
      const dpr = window.devicePixelRatio || 1;
      const rect = container.getBoundingClientRect();
      canvas.width = rect.width * dpr;
      canvas.height = rect.height * dpr;
      canvas.style.width = `${rect.width}px`;
      canvas.style.height = `${rect.height}px`;
    };

    resizeCanvas();
    window.addEventListener('resize', resizeCanvas);
    return () => window.removeEventListener('resize', resizeCanvas);
  }, []);

  // Draw the graph
  const draw = useCallback(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const dpr = window.devicePixelRatio || 1;
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    ctx.scale(dpr, dpr);

    const width = canvas.width / dpr;
    const height = canvas.height / dpr;

    // Apply transform
    ctx.save();
    ctx.translate(width / 2, height / 2);
    ctx.scale(scale, scale);
    ctx.translate(-width / 2 + offset.x, -height / 2 + offset.y);

    // Draw edges
    ctx.strokeStyle = '#d0d0d0';
    ctx.lineWidth = 1.5;
    edges.forEach(([from, to]) => {
      const fromNode = nodes.find(n => n.id === from);
      const toNode = nodes.find(n => n.id === to);
      if (fromNode && toNode) {
        ctx.beginPath();
        ctx.moveTo(fromNode.x, fromNode.y);
        ctx.lineTo(toNode.x, toNode.y);
        ctx.stroke();

        // Draw arrow
        const angle = Math.atan2(toNode.y - fromNode.y, toNode.x - fromNode.x);
        const arrowLength = 8;
        const arrowX = toNode.x - Math.cos(angle) * (toNode.radius + 2);
        const arrowY = toNode.y - Math.sin(angle) * (toNode.radius + 2);

        ctx.beginPath();
        ctx.moveTo(arrowX, arrowY);
        ctx.lineTo(
          arrowX - arrowLength * Math.cos(angle - Math.PI / 6),
          arrowY - arrowLength * Math.sin(angle - Math.PI / 6)
        );
        ctx.moveTo(arrowX, arrowY);
        ctx.lineTo(
          arrowX - arrowLength * Math.cos(angle + Math.PI / 6),
          arrowY - arrowLength * Math.sin(angle + Math.PI / 6)
        );
        ctx.stroke();
      }
    });

    // Draw nodes
    nodes.forEach(node => {
      const isHovered = hoveredNode?.id === node.id;
      const isSelected = selectedNode?.id === node.id;
      const radius = isHovered ? HOVER_RADIUS : node.radius;

      // Node color based on status
      let fillColor = '#e0e0e0';
      if (node.bead.status === 'open') {
        fillColor = '#e6f4ea';
      } else if (node.bead.status === 'closed') {
        fillColor = '#f5f5f5';
      }

      // Highlight based on priority
      if (node.bead.priority === 0) {
        fillColor = '#fce8e6';
      } else if (node.bead.priority === 1) {
        fillColor = '#fff8e1';
      }

      // Draw node
      ctx.beginPath();
      ctx.arc(node.x, node.y, radius, 0, Math.PI * 2);
      ctx.fillStyle = fillColor;
      ctx.fill();

      // Border
      ctx.strokeStyle = isSelected ? '#1976d2' : isHovered ? '#64b5f6' : '#999';
      ctx.lineWidth = isSelected ? 2 : 1;
      ctx.stroke();

      // Draw ID text for hovered/selected nodes
      if (isHovered || isSelected) {
        ctx.fillStyle = '#333';
        ctx.font = '10px Inter, sans-serif';
        ctx.textAlign = 'center';
        ctx.textBaseline = 'middle';
        ctx.fillText(node.bead.id, node.x, node.y - radius - 8);
      }
    });

    ctx.restore();
  }, [nodes, edges, offset, scale, hoveredNode, selectedNode]);

  useEffect(() => {
    draw();
  }, [draw]);

  // Handle mouse events
  const handleMouseMove = useCallback((e: React.MouseEvent<HTMLCanvasElement>) => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const rect = canvas.getBoundingClientRect();
    const mouseX = e.clientX - rect.left;
    const mouseY = e.clientY - rect.top;

    // Handle dragging
    if (isDragging) {
      setOffset(prev => ({
        x: prev.x + (mouseX - dragStart.x) / scale,
        y: prev.y + (mouseY - dragStart.y) / scale,
      }));
      setDragStart({ x: mouseX, y: mouseY });
      return;
    }

    // Check for node hover
    const dpr = window.devicePixelRatio || 1;
    const width = canvas.width / dpr;
    const height = canvas.height / dpr;

    // Transform mouse position to graph coordinates
    const graphX = (mouseX - width / 2) / scale + width / 2 - offset.x;
    const graphY = (mouseY - height / 2) / scale + height / 2 - offset.y;

    let found: BeadNode | null = null;
    for (const node of nodes) {
      const dx = graphX - node.x;
      const dy = graphY - node.y;
      if (dx * dx + dy * dy < HOVER_RADIUS * HOVER_RADIUS) {
        found = node;
        break;
      }
    }

    setHoveredNode(found);
    canvas.style.cursor = found ? 'pointer' : 'grab';
  }, [nodes, scale, offset, isDragging, dragStart]);

  const handleMouseDown = useCallback((e: React.MouseEvent<HTMLCanvasElement>) => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const rect = canvas.getBoundingClientRect();
    const mouseX = e.clientX - rect.left;
    const mouseY = e.clientY - rect.top;

    if (hoveredNode) {
      setSelectedNode(hoveredNode);
    } else {
      setIsDragging(true);
      setDragStart({ x: mouseX, y: mouseY });
      canvas.style.cursor = 'grabbing';
    }
  }, [hoveredNode]);

  const handleMouseUp = useCallback(() => {
    setIsDragging(false);
    if (canvasRef.current) {
      canvasRef.current.style.cursor = hoveredNode ? 'pointer' : 'grab';
    }
  }, [hoveredNode]);

  const handleWheel = useCallback((e: React.WheelEvent<HTMLCanvasElement>) => {
    e.preventDefault();
    const delta = e.deltaY > 0 ? 0.9 : 1.1;
    setScale(prev => Math.max(0.1, Math.min(5, prev * delta)));
  }, []);

  const handleDoubleClick = useCallback(() => {
    // Reset view
    setOffset({ x: 0, y: 0 });
    setScale(1);
  }, []);

  return (
    <div className="bead-graph-container" ref={containerRef}>
      <div className="bead-graph-toolbar">
        <div className="graph-stats">
          <span className="graph-stat">
            <strong>{beads.length}</strong> beads
          </span>
          <span className="graph-stat">
            <strong>{edges.length}</strong> dependencies
          </span>
        </div>
        <div className="graph-controls">
          <button
            className="graph-control-btn"
            onClick={() => setScale(s => Math.min(5, s * 1.2))}
            title="Zoom in"
          >
            +
          </button>
          <button
            className="graph-control-btn"
            onClick={() => setScale(s => Math.max(0.1, s / 1.2))}
            title="Zoom out"
          >
            −
          </button>
          <button
            className="graph-control-btn"
            onClick={() => { setOffset({ x: 0, y: 0 }); setScale(1); }}
            title="Reset view"
          >
            ↺
          </button>
        </div>
      </div>

      <canvas
        ref={canvasRef}
        className="bead-graph-canvas"
        onMouseMove={handleMouseMove}
        onMouseDown={handleMouseDown}
        onMouseUp={handleMouseUp}
        onMouseLeave={handleMouseUp}
        onWheel={handleWheel}
        onDoubleClick={handleDoubleClick}
      />

      {selectedNode && (
        <div className="bead-node-tooltip">
          <div className="tooltip-header">
            <strong>{selectedNode.bead.id}</strong>
            <button
              className="tooltip-close"
              onClick={() => setSelectedNode(null)}
            >
              ×
            </button>
          </div>
          <div className="tooltip-body">
            <p><strong>Title:</strong> {selectedNode.bead.title}</p>
            <p><strong>Status:</strong> {selectedNode.bead.status}</p>
            <p><strong>Type:</strong> {selectedNode.bead.issue_type}</p>
            <p><strong>Priority:</strong> P{selectedNode.bead.priority}</p>
            {selectedNode.dependencies.length > 0 && (
              <p><strong>Dependencies:</strong> {selectedNode.dependencies.join(', ')}</p>
            )}
            {selectedNode.dependents.length > 0 && (
              <p><strong>Dependents:</strong> {selectedNode.dependents.join(', ')}</p>
            )}
          </div>
        </div>
      )}

      <div className="bead-graph-hints">
        <span>Scroll to zoom • Drag to pan • Double-click to reset • Click node for details</span>
      </div>
    </div>
  );
}
