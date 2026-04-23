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

const NODE_RADIUS = 6;
const HOVER_RADIUS = 10;
const LEVEL_HEIGHT = 50;
const NODE_SPACING = 30;
const GRID_CELL = 20; // spatial hash cell size for hit-testing

export default function BeadGraph({ beads }: BeadGraphProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const [selectedNode, setSelectedNode] = useState<BeadNode | null>(null);
  const [offset, setOffset] = useState({ x: 0, y: 0 });
  const [scale, setScale] = useState(1);
  const [isDragging, setIsDragging] = useState(false);
  const dragStartRef = useRef({ x: 0, y: 0 });
  const rafRef = useRef<number>(0);
  const dirtyRef = useRef(true);

  // Use refs for hover/select state inside draw() to avoid recreating the callback
  const hoveredNodeRef = useRef<BeadNode | null>(null);
  const selectedNodeRef = useRef<BeadNode | null>(null);

  // Keep selectedNode ref in sync with state
  useEffect(() => { selectedNodeRef.current = selectedNode; }, [selectedNode]);

  // Build dependency graph and layout — iterative topological sort for 500+ beads
  const { nodes, edges, spatialGrid, nodesMap } = useMemo(() => {
    const nodesMap = new Map<string, BeadNode>();
    const beadMap = new Map(beads.map(b => [b.id, b]));

    // Iterative topological sort using Kahn's algorithm
    const levels = new Map<string, number>();
    const inDegree = new Map<string, number>();
    const dependentsMap = new Map<string, string[]>();

    beads.forEach(b => {
      inDegree.set(b.id, 0);
      dependentsMap.set(b.id, []);
    });

    beads.forEach(b => {
      for (const dep of b.dependencies) {
        if (beadMap.has(dep)) {
          inDegree.set(b.id, (inDegree.get(b.id) ?? 0) + 1);
          dependentsMap.get(dep)?.push(b.id);
        }
      }
    });

    // BFS level assignment
    const queue: string[] = [];
    beads.forEach(b => {
      if ((inDegree.get(b.id) ?? 0) === 0) queue.push(b.id);
    });

    let processed = 0;
    while (queue.length > 0) {
      const beadId = queue.shift()!;
      const bead = beadMap.get(beadId);
      let level = 0;
      if (bead) {
        for (const dep of bead.dependencies) {
          const depLevel = levels.get(dep);
          if (depLevel !== undefined && depLevel + 1 > level) {
            level = depLevel + 1;
          }
        }
      }
      levels.set(beadId, level);
      processed++;

      for (const depId of (dependentsMap.get(beadId) ?? [])) {
        const deg = (inDegree.get(depId) ?? 1) - 1;
        inDegree.set(depId, deg);
        if (deg === 0) queue.push(depId);
      }
    }

    // Handle cycles: assign level 0 to any unprocessed beads
    if (processed < beads.length) {
      beads.forEach(b => {
        if (!levels.has(b.id)) levels.set(b.id, 0);
      });
    }

    const levelGroups = new Map<number, BeadData[]>();
    beads.forEach(bead => {
      const level = levels.get(bead.id) ?? 0;
      if (!levelGroups.has(level)) levelGroups.set(level, []);
      levelGroups.get(level)!.push(bead);
    });

    let maxWidth = 0;
    for (const g of levelGroups.values()) {
      if (g.length > maxWidth) maxWidth = g.length;
    }
    const gw = maxWidth * (NODE_SPACING + NODE_RADIUS * 2);

    levelGroups.forEach((beadsAtLevel, level) => {
      const levelY = level * LEVEL_HEIGHT + 40;
      const levelWidth = beadsAtLevel.length * (NODE_SPACING + NODE_RADIUS * 2);
      const startX = (gw - levelWidth) / 2;

      beadsAtLevel.forEach((bead, index) => {
        const x = startX + index * (NODE_SPACING + NODE_RADIUS * 2) + NODE_RADIUS;
        nodesMap.set(bead.id, {
          id: bead.id, x, y: levelY, radius: NODE_RADIUS, bead,
          dependencies: bead.dependencies, dependents: [],
        });
      });
    });

    // Build dependents
    nodesMap.forEach(node => {
      node.dependencies.forEach(depId => {
        const depNode = nodesMap.get(depId);
        if (depNode) depNode.dependents.push(node.id);
      });
    });

    // Build edges with direct node references
    const edgePairsList: [BeadNode, BeadNode][] = [];
    nodesMap.forEach(node => {
      node.dependencies.forEach(depId => {
        const depNode = nodesMap.get(depId);
        if (depNode) edgePairsList.push([depNode, node]);
      });
    });

    // Spatial hash grid for O(1) hit testing
    const grid = new Map<string, BeadNode[]>();
    nodesMap.forEach(node => {
      const cx = Math.floor(node.x / GRID_CELL);
      const cy = Math.floor(node.y / GRID_CELL);
      for (let dx = -1; dx <= 1; dx++) {
        for (let dy = -1; dy <= 1; dy++) {
          const key = `${cx + dx},${cy + dy}`;
          let cell = grid.get(key);
          if (!cell) { cell = []; grid.set(key, cell); }
          cell.push(node);
        }
      }
    });

    const nodesArray = Array.from(nodesMap.values());
    return { nodes: nodesArray, edges: edgePairsList, spatialGrid: grid, nodesMap };
  }, [beads]);

  // Stable refs for layout data (avoid restarting RAF on offset/scale changes)
  const nodesRef = useRef(nodes);
  const edgesRef = useRef(edges);
  useEffect(() => { nodesRef.current = nodes; }, [nodes]);
  useEffect(() => { edgesRef.current = edges; }, [edges]);

  const offsetRef = useRef(offset);
  const scaleRef = useRef(scale);
  useEffect(() => { offsetRef.current = offset; }, [offset]);
  useEffect(() => { scaleRef.current = scale; }, [scale]);

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
      dirtyRef.current = true;
    };

    resizeCanvas();
    window.addEventListener('resize', resizeCanvas);
    return () => window.removeEventListener('resize', resizeCanvas);
  }, []);

  // Stable draw function — reads from refs, not state
  const draw = useCallback(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const dpr = window.devicePixelRatio || 1;
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);

    const currentNodes = nodesRef.current;
    const currentEdges = edgesRef.current;
    const s = scaleRef.current;
    const o = offsetRef.current;
    const hovered = hoveredNodeRef.current;
    const selected = selectedNodeRef.current;

    const width = canvas.width / dpr;
    const height = canvas.height / dpr;

    ctx.save();
    ctx.translate(width / 2, height / 2);
    ctx.scale(s, s);
    ctx.translate(-width / 2 + o.x, -height / 2 + o.y);

    // Compute visible bounds in graph coordinates for culling
    const margin = 20;
    const visLeft = (0 - width / 2) / s + width / 2 - o.x - margin;
    const visRight = (width - width / 2) / s + width / 2 - o.x + margin;
    const visTop = (0 - height / 2) / s + height / 2 - o.y - margin;
    const visBottom = (height - height / 2) / s + height / 2 - o.y + margin;

    // Draw edges — single batched path for performance with 500+ beads
    ctx.strokeStyle = '#d0d0d0';
    ctx.lineWidth = 1;
    ctx.beginPath();
    for (const [fromNode, toNode] of currentEdges) {
      if (fromNode.x < visLeft && toNode.x < visLeft) continue;
      if (fromNode.x > visRight && toNode.x > visRight) continue;
      if (fromNode.y < visTop && toNode.y < visTop) continue;
      if (fromNode.y > visBottom && toNode.y > visBottom) continue;

      ctx.moveTo(fromNode.x, fromNode.y);
      ctx.lineTo(toNode.x, toNode.y);
    }
    ctx.stroke();

    // Draw arrow heads — skip when zoomed out (expensive and invisible at low zoom)
    if (s >= 0.4) {
      ctx.strokeStyle = '#b0b0b0';
      ctx.lineWidth = 1;
      const arrowLen = 5;
      for (const [fromNode, toNode] of currentEdges) {
        if (fromNode.x < visLeft && toNode.x < visLeft) continue;
        if (fromNode.x > visRight && toNode.x > visRight) continue;
        if (fromNode.y < visTop && toNode.y < visTop) continue;
        if (fromNode.y > visBottom && toNode.y > visBottom) continue;

        const angle = Math.atan2(toNode.y - fromNode.y, toNode.x - fromNode.x);
        const ax = toNode.x - Math.cos(angle) * (toNode.radius + 2);
        const ay = toNode.y - Math.sin(angle) * (toNode.radius + 2);

        ctx.beginPath();
        ctx.moveTo(ax, ay);
        ctx.lineTo(ax - arrowLen * Math.cos(angle - 0.5), ay - arrowLen * Math.sin(angle - 0.5));
        ctx.moveTo(ax, ay);
        ctx.lineTo(ax - arrowLen * Math.cos(angle + 0.5), ay - arrowLen * Math.sin(angle + 0.5));
        ctx.stroke();
      }
    }

    // Draw nodes — batch by fill color to reduce state changes for 500+ beads
    const colorBuckets = new Map<string, { node: typeof currentNodes[0]; radius: number }[]>();

    for (const node of currentNodes) {
      if (node.x + NODE_RADIUS < visLeft || node.x - NODE_RADIUS > visRight) continue;
      if (node.y + NODE_RADIUS < visTop || node.y - NODE_RADIUS > visBottom) continue;

      const isHovered = hovered?.id === node.id;
      const isSelected = selected?.id === node.id;
      const radius = isHovered ? HOVER_RADIUS : node.radius;

      let fillColor = '#e0e0e0';
      if (node.bead.status === 'open') fillColor = '#e6f4ea';
      else if (node.bead.status === 'closed') fillColor = '#f5f5f5';
      if (node.bead.priority === 0) fillColor = '#fce8e6';
      else if (node.bead.priority === 1) fillColor = '#fff8e1';

      const strokeKey = isSelected ? 'sel' : isHovered ? 'hov' : 'def';
      const key = `${fillColor}|${strokeKey}`;

      let bucket = colorBuckets.get(key);
      if (!bucket) { bucket = []; colorBuckets.set(key, bucket); }
      bucket.push({ node, radius });
    }

    const strokeWidths: Record<string, number> = { sel: 2, hov: 1, def: 1 };
    const strokeColors: Record<string, string> = { sel: '#1976d2', hov: '#64b5f6', def: '#aaa' };

    for (const [key, bucket] of colorBuckets) {
      const [fill, strokeKey] = key.split('|');
      ctx.fillStyle = fill;
      ctx.strokeStyle = strokeColors[strokeKey];
      ctx.lineWidth = strokeWidths[strokeKey];

      ctx.beginPath();
      for (const { node, radius } of bucket) {
        ctx.moveTo(node.x + radius, node.y);
        ctx.arc(node.x, node.y, radius, 0, Math.PI * 2);
      }
      ctx.fill();
      ctx.stroke();

      // Labels for hovered/selected only
      if (strokeKey === 'sel' || strokeKey === 'hov') {
        ctx.fillStyle = '#333';
        ctx.font = '9px Inter, system-ui, sans-serif';
        ctx.textAlign = 'center';
        ctx.textBaseline = 'bottom';
        for (const { node, radius } of bucket) {
          ctx.fillText(node.bead.id, node.x, node.y - radius - 3);
        }
      }
    }

    ctx.restore();
  }, []); // Stable — reads from refs

  // RAF loop — single long-lived loop, only redraws when dirty
  useEffect(() => {
    let running = true;
    function loop() {
      if (!running) return;
      if (dirtyRef.current) {
        draw();
        dirtyRef.current = false;
      }
      rafRef.current = requestAnimationFrame(loop);
    }
    rafRef.current = requestAnimationFrame(loop);
    return () => { running = false; cancelAnimationFrame(rafRef.current); };
  }, [draw]);

  // Spatial hash hit test
  const hitTest = useCallback((graphX: number, graphY: number): BeadNode | null => {
    const cx = Math.floor(graphX / GRID_CELL);
    const cy = Math.floor(graphY / GRID_CELL);
    const cell = spatialGrid.get(`${cx},${cy}`);
    if (!cell) return null;
    for (const node of cell) {
      const dx = graphX - node.x;
      const dy = graphY - node.y;
      if (dx * dx + dy * dy < HOVER_RADIUS * HOVER_RADIUS) return node;
    }
    return null;
  }, [spatialGrid]);

  // Mouse position to graph coordinates
  const toGraphCoords = useCallback((clientX: number, clientY: number) => {
    const canvas = canvasRef.current;
    if (!canvas) return { x: 0, y: 0 };
    const rect = canvas.getBoundingClientRect();
    const mouseX = clientX - rect.left;
    const mouseY = clientY - rect.top;
    const dpr = window.devicePixelRatio || 1;
    const w = canvas.width / dpr;
    const h = canvas.height / dpr;
    return {
      x: (mouseX - w / 2) / scaleRef.current + w / 2 - offsetRef.current.x,
      y: (mouseY - h / 2) / scaleRef.current + h / 2 - offsetRef.current.y,
    };
  }, []);

  // Throttled mouse move — ~60fps cap for 500+ bead responsiveness
  const lastMoveRef = useRef(0);
  const handleMouseMove = useCallback((e: React.MouseEvent<HTMLCanvasElement>) => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const now = performance.now();
    if (isDragging) {
      const rect = canvas.getBoundingClientRect();
      const mx = e.clientX - rect.left;
      const my = e.clientY - rect.top;
      setOffset(prev => {
        const next = {
          x: prev.x + (mx - dragStartRef.current.x) / scaleRef.current,
          y: prev.y + (my - dragStartRef.current.y) / scaleRef.current,
        };
        offsetRef.current = next;
        return next;
      });
      dragStartRef.current = { x: mx, y: my };
      dirtyRef.current = true;
      return;
    }

    // Throttle hover detection to ~60fps
    if (now - lastMoveRef.current < 16) return;
    lastMoveRef.current = now;

    const { x, y } = toGraphCoords(e.clientX, e.clientY);
    const found = hitTest(x, y);
    const prevHovered = hoveredNodeRef.current;
    if (prevHovered !== found && (prevHovered?.id !== found?.id)) {
      hoveredNodeRef.current = found;
      canvas.style.cursor = found ? 'pointer' : 'grab';
      dirtyRef.current = true;
    }
  }, [isDragging, toGraphCoords, hitTest]);

  const handleMouseDown = useCallback((e: React.MouseEvent<HTMLCanvasElement>) => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const rect = canvas.getBoundingClientRect();

    if (hoveredNodeRef.current) {
      setSelectedNode(hoveredNodeRef.current);
      selectedNodeRef.current = hoveredNodeRef.current;
      dirtyRef.current = true;
    } else {
      setIsDragging(true);
      dragStartRef.current = { x: e.clientX - rect.left, y: e.clientY - rect.top };
      canvas.style.cursor = 'grabbing';
    }
  }, []);

  const handleMouseUp = useCallback(() => {
    setIsDragging(false);
    if (canvasRef.current) {
      canvasRef.current.style.cursor = hoveredNodeRef.current ? 'pointer' : 'grab';
    }
  }, []);

  const handleWheel = useCallback((e: React.WheelEvent<HTMLCanvasElement>) => {
    e.preventDefault();
    const delta = e.deltaY > 0 ? 0.9 : 1.1;
    setScale(prev => {
      const next = Math.max(0.1, Math.min(5, prev * delta));
      scaleRef.current = next;
      dirtyRef.current = true;
      return next;
    });
  }, []);

  const handleDoubleClick = useCallback(() => {
    const resetOffset = { x: 0, y: 0 };
    setOffset(resetOffset);
    offsetRef.current = resetOffset;
    setScale(1);
    scaleRef.current = 1;
    dirtyRef.current = true;
  }, []);

  // Keyboard navigation within the graph
  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (e.key === 'Escape') {
      setSelectedNode(null);
      selectedNodeRef.current = null;
      dirtyRef.current = true;
      return;
    }
    const sel = selectedNodeRef.current;
    if (!sel) return;

    // Navigate to dependency/dependent with arrow keys
    if (e.key === 'ArrowUp' && sel.dependencies.length > 0) {
      const dep = nodesMap.get(sel.dependencies[0]);
      if (dep) {
        setSelectedNode(dep);
        selectedNodeRef.current = dep;
        dirtyRef.current = true;
      }
    } else if (e.key === 'ArrowDown' && sel.dependents.length > 0) {
      const dep = nodesMap.get(sel.dependents[0]);
      if (dep) {
        setSelectedNode(dep);
        selectedNodeRef.current = dep;
        dirtyRef.current = true;
      }
    }
  }, [nodesMap]);

  const openCount = beads.filter(b => b.status === 'open').length;

  return (
    <div className="bead-graph-container" ref={containerRef} onKeyDown={handleKeyDown} tabIndex={0}>
      <div className="bead-graph-toolbar">
        <div className="graph-stats">
          <span className="graph-stat"><strong>{beads.length}</strong> beads</span>
          <span className="graph-stat"><strong>{openCount}</strong> open</span>
          <span className="graph-stat"><strong>{beads.length - openCount}</strong> closed</span>
          <span className="graph-stat"><strong>{edges.length}</strong> deps</span>
        </div>
        <div className="graph-controls">
          <button className="graph-control-btn" onClick={() => {
            setScale(s => { const n = Math.min(5, s * 1.2); scaleRef.current = n; return n; });
            dirtyRef.current = true;
          }} title="Zoom in">+</button>
          <button className="graph-control-btn" onClick={() => {
            setScale(s => { const n = Math.max(0.1, s / 1.2); scaleRef.current = n; return n; });
            dirtyRef.current = true;
          }} title="Zoom out">−</button>
          <button className="graph-control-btn" onClick={handleDoubleClick} title="Reset view">↺</button>
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
        <div className="bead-node-tooltip" role="dialog" aria-label={`Details for ${selectedNode.bead.id}`}>
          <div className="tooltip-header">
            <strong>{selectedNode.bead.id}</strong>
            <button className="tooltip-close" onClick={() => {
              setSelectedNode(null);
              selectedNodeRef.current = null;
              dirtyRef.current = true;
            }} aria-label="Close">×</button>
          </div>
          <div className="tooltip-body">
            <p><strong>Title:</strong> {selectedNode.bead.title}</p>
            <p><strong>Status:</strong> {selectedNode.bead.status}</p>
            <p><strong>Type:</strong> {selectedNode.bead.issue_type}</p>
            <p><strong>Priority:</strong> P{selectedNode.bead.priority}</p>
            {selectedNode.dependencies.length > 0 && (
              <p><strong>Deps:</strong> {selectedNode.dependencies.join(', ')}</p>
            )}
            {selectedNode.dependents.length > 0 && (
              <p><strong>Dependents:</strong> {selectedNode.dependents.join(', ')}</p>
            )}
          </div>
        </div>
      )}

      <div className="bead-graph-hints">
        <span>Scroll to zoom · Drag to pan · Double-click to reset · ↑↓ navigate deps · Esc deselect</span>
      </div>
    </div>
  );
}
