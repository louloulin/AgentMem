/**
 * Enhanced Memory Knowledge Graph Page
 * 
 * Advanced knowledge graph visualization with:
 * - Force-directed layout algorithm
 * - Interactive node dragging
 * - Node search and filtering
 * - Relationship highlighting
 * - Graph analytics
 * - Mini-map navigation
 */

'use client';

import React, { useState, useEffect, useRef, useCallback } from 'react';
import { 
  Network, 
  Brain, 
  ZoomIn, 
  ZoomOut, 
  Maximize2,
  Search,
  Layers,
  BarChart3,
  Share2,
  RefreshCw
} from 'lucide-react';
import { Card } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Input } from '@/components/ui/input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { apiClient, Memory } from '@/lib/api-client';
import { cn } from '@/lib/utils';

interface GraphNode {
  id: string;
  label: string;
  type: string;
  importance: number;
  x: number;
  y: number;
  vx: number;  // velocity x
  vy: number;  // velocity y
  fx?: number; // fixed x (for dragging)
  fy?: number; // fixed y (for dragging)
  radius: number;
  content: string;
}

interface GraphEdge {
  source: string;
  target: string;
  type: string;
  strength: number;
}

interface GraphStats {
  totalNodes: number;
  totalEdges: number;
  avgDegree: number;
  maxDegree: number;
  clusters: number;
  density: number;
}

export default function GraphPage() {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const animationRef = useRef<number | undefined>(undefined);
  const [memories, setMemories] = useState<Memory[]>([]);
  const [nodes, setNodes] = useState<GraphNode[]>([]);
  const [edges, setEdges] = useState<GraphEdge[]>([]);
  const [selectedNode, setSelectedNode] = useState<GraphNode | null>(null);
  const [hoveredNode, setHoveredNode] = useState<GraphNode | null>(null);
  const [filterType, setFilterType] = useState<string>('all');
  const [loading, setLoading] = useState(true);
  const [zoom, setZoom] = useState(1);
  const [pan, setPan] = useState({ x: 0, y: 0 });
  const [isDragging, setIsDragging] = useState(false);
  const [draggedNode, setDraggedNode] = useState<GraphNode | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [showStats, setShowStats] = useState(false);
  const [graphStats, setGraphStats] = useState<GraphStats | null>(null);
  const [showLabels, setShowLabels] = useState<'none' | 'selected' | 'important' | 'all'>('important');
  
  // Physics simulation parameters - Optimized for better spacing
  const FORCE_STRENGTH = 0.15;
  const DAMPING = 0.85;
  const LINK_DISTANCE = 150;
  const REPULSION_STRENGTH = 15000; // Increased for better node separation

  const loadMemoriesWrapper = useCallback(async () => {
    try {
      setLoading(true);
      const response = await apiClient.getAllMemories(0, 100);
      setMemories(response?.memories || []);
    } catch (error) {
      setMemories([]);
      console.error('Failed to load memories:', error);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadMemoriesWrapper();
    return () => {
      if (animationRef.current) {
        cancelAnimationFrame(animationRef.current);
      }
    };
  }, [loadMemoriesWrapper]);

  useEffect(() => {
    if (memories && memories.length > 0) {
      buildGraph();
    }
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [memories, filterType]);

  useEffect(() => {
    if (nodes && nodes.length > 0) {
      startSimulation();
    }
    return () => {
      if (animationRef.current) {
        cancelAnimationFrame(animationRef.current);
      }
    };
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [nodes.length]);


  const buildGraph = () => {
    const filteredMemories = filterType === 'all'
      ? (memories || [])
      : (memories || []).filter(m => m.memory_type === filterType);

    // Create nodes with initial random positions - spread out more
    const canvasWidth = 1200;
    const canvasHeight = 800;
    const graphNodes: GraphNode[] = filteredMemories.map((memory) => ({
      id: memory.id,
      label: memory.content.substring(0, 20) + '...', // Shorter labels
      type: memory.memory_type,
      importance: memory.importance,
      x: canvasWidth / 2 + (Math.random() - 0.5) * 400, // Wider initial spread
      y: canvasHeight / 2 + (Math.random() - 0.5) * 400,
      vx: 0,
      vy: 0,
      radius: 10 + memory.importance * 15, // Larger nodes
      content: memory.content,
    }));

    // Create edges based on content similarity
    const graphEdges: GraphEdge[] = [];
    for (let i = 0; i < graphNodes.length; i++) {
      for (let j = i + 1; j < graphNodes.length; j++) {
        const memory1 = filteredMemories[i];
        const memory2 = filteredMemories[j];
        
        const words1 = memory1.content.toLowerCase().split(' ').filter(w => w.length > 3);
        const words2 = memory2.content.toLowerCase().split(' ').filter(w => w.length > 3);
        const commonWords = words1.filter(w => words2.includes(w));
        
        if (commonWords.length > 2) {
          graphEdges.push({
            source: graphNodes[i].id,
            target: graphNodes[j].id,
            type: 'related',
            strength: commonWords.length / Math.max(words1.length, words2.length),
          });
        }
      }
    }

    setNodes(graphNodes);
    setEdges(graphEdges);
    calculateStats(graphNodes, graphEdges);
  };

  const calculateStats = (graphNodes: GraphNode[], graphEdges: GraphEdge[]) => {
    const degrees = new Map<string, number>();
    graphNodes.forEach(node => degrees.set(node.id, 0));
    graphEdges.forEach(edge => {
      degrees.set(edge.source, (degrees.get(edge.source) || 0) + 1);
      degrees.set(edge.target, (degrees.get(edge.target) || 0) + 1);
    });

    const degreeValues = Array.from(degrees.values());
    const avgDegree = degreeValues.reduce((a, b) => a + b, 0) / degreeValues.length || 0;
    const maxDegree = Math.max(...degreeValues, 0);
    const density = graphNodes.length > 1 
      ? (2 * graphEdges.length) / (graphNodes.length * (graphNodes.length - 1))
      : 0;

    setGraphStats({
      totalNodes: graphNodes.length,
      totalEdges: graphEdges.length,
      avgDegree: Number(avgDegree.toFixed(2)),
      maxDegree,
      clusters: estimateClusters(graphNodes, graphEdges),
      density: Number(density.toFixed(4)),
    });
  };

  const estimateClusters = (graphNodes: GraphNode[], graphEdges: GraphEdge[]): number => {
    // Simple clustering estimation based on connected components
    const visited = new Set<string>();
    let clusters = 0;

    const dfs = (nodeId: string) => {
      visited.add(nodeId);
      const connectedEdges = graphEdges.filter(e => e.source === nodeId || e.target === nodeId);
      connectedEdges.forEach(edge => {
        const nextNode = edge.source === nodeId ? edge.target : edge.source;
        if (!visited.has(nextNode)) {
          dfs(nextNode);
        }
      });
    };

    graphNodes.forEach(node => {
      if (!visited.has(node.id)) {
        clusters++;
        dfs(node.id);
      }
    });

    return clusters;
  };

  const startSimulation = () => {
    const simulate = () => {
      setNodes(prevNodes => {
        const newNodes = prevNodes.map(node => ({ ...node }));

        // Apply forces
        newNodes.forEach((node, i) => {
          if (node.fx !== undefined && node.fy !== undefined) {
            node.x = node.fx;
            node.y = node.fy;
            return;
          }

          let fx = 0, fy = 0;

          // Repulsion between nodes
          newNodes.forEach((other, j) => {
            if (i !== j) {
              const dx = node.x - other.x;
              const dy = node.y - other.y;
              const dist = Math.sqrt(dx * dx + dy * dy) || 1;
              const force = REPULSION_STRENGTH / (dist * dist);
              fx += (dx / dist) * force;
              fy += (dy / dist) * force;
            }
          });

          // Attraction along edges
          edges.forEach(edge => {
            const isSource = edge.source === node.id;
            const isTarget = edge.target === node.id;
            if (isSource || isTarget) {
              const otherId = isSource ? edge.target : edge.source;
              const other = newNodes.find(n => n.id === otherId);
              if (other) {
                const dx = other.x - node.x;
                const dy = other.y - node.y;
                const dist = Math.sqrt(dx * dx + dy * dy) || 1;
                const force = (dist - LINK_DISTANCE) * FORCE_STRENGTH * edge.strength;
                fx += (dx / dist) * force;
                fy += (dy / dist) * force;
              }
            }
          });

      // Center gravity - weaker to allow more spread
      const centerX = 600;
      const centerY = 400;
      fx += (centerX - node.x) * 0.005; // Reduced gravity
      fy += (centerY - node.y) * 0.005;

          // Update velocity
          node.vx = (node.vx + fx) * DAMPING;
          node.vy = (node.vy + fy) * DAMPING;

          // Update position
          node.x += node.vx;
          node.y += node.vy;

      // Boundary constraints - larger canvas
      node.x = Math.max(node.radius, Math.min(1200 - node.radius, node.x));
      node.y = Math.max(node.radius, Math.min(800 - node.radius, node.y));
        });

        return newNodes;
      });

      drawGraph();
      animationRef.current = requestAnimationFrame(simulate);
    };

    if (animationRef.current) {
      cancelAnimationFrame(animationRef.current);
    }
    animationRef.current = requestAnimationFrame(simulate);
  };

  const drawGraph = useCallback(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    ctx.clearRect(0, 0, canvas.width, canvas.height);
    ctx.save();
    ctx.translate(pan.x, pan.y);
    ctx.scale(zoom, zoom);

    // Draw edges
    edges.forEach(edge => {
      const sourceNode = nodes.find(n => n.id === edge.source);
      const targetNode = nodes.find(n => n.id === edge.target);
      if (sourceNode && targetNode) {
        ctx.beginPath();
        ctx.moveTo(sourceNode.x, sourceNode.y);
        ctx.lineTo(targetNode.x, targetNode.y);
        
        const isHighlighted = 
          (selectedNode && (edge.source === selectedNode.id || edge.target === selectedNode.id)) ||
          (hoveredNode && (edge.source === hoveredNode.id || edge.target === hoveredNode.id));
        
        ctx.strokeStyle = isHighlighted ? '#a78bfa' : '#475569';
        ctx.lineWidth = isHighlighted ? 2 : 1;
        ctx.globalAlpha = isHighlighted ? 0.8 : 0.3;
        ctx.stroke();
        ctx.globalAlpha = 1;
      }
    });

    // Draw nodes
    const filteredNodes = searchQuery
      ? nodes.filter(n => n.label.toLowerCase().includes(searchQuery.toLowerCase()) || 
                         n.content.toLowerCase().includes(searchQuery.toLowerCase()))
      : nodes;

    nodes.forEach(node => {
      const isSelected = selectedNode?.id === node.id;
      const isHovered = hoveredNode?.id === node.id;
      const isFiltered = searchQuery && !filteredNodes.includes(node);
      const isImportant = node.importance > 0.5;

      // Node circle
      ctx.beginPath();
      ctx.arc(node.x, node.y, node.radius, 0, 2 * Math.PI);
      
      const colors: Record<string, string> = {
        episodic: '#3b82f6',
        semantic: '#10b981',
        procedural: '#f59e0b',
        working: '#8b5cf6',
        core: '#ef4444',
      };
      ctx.fillStyle = colors[node.type] || '#6b7280';
      ctx.globalAlpha = isFiltered ? 0.2 : 1;
      ctx.fill();

      // Glow effect for selected/hovered nodes
      if (isSelected || isHovered) {
        ctx.shadowColor = colors[node.type] || '#6b7280';
        ctx.shadowBlur = 15;
        ctx.strokeStyle = '#ffffff';
        ctx.lineWidth = 3;
        ctx.globalAlpha = 1;
        ctx.stroke();
        ctx.shadowBlur = 0;
      }

      // Smart label rendering based on showLabels setting and zoom
      const shouldShowLabel = !isFiltered && (
        showLabels === 'all' ||
        (showLabels === 'selected' && (isSelected || isHovered)) ||
        (showLabels === 'important' && (isSelected || isHovered || (isImportant && zoom > 0.8)))
      );

      if (shouldShowLabel) {
        // Background for better readability
        ctx.font = `${Math.max(10, 11 * zoom)}px sans-serif`;
        ctx.textAlign = 'center';
        const displayLabel = node.label.length > 15 ? node.label.substring(0, 15) + '...' : node.label;
        const textMetrics = ctx.measureText(displayLabel);
        const padding = 4;
        
        // Semi-transparent background
        ctx.fillStyle = 'rgba(0, 0, 0, 0.7)';
        ctx.fillRect(
          node.x - textMetrics.width / 2 - padding,
          node.y + node.radius + 8 - padding,
          textMetrics.width + padding * 2,
          12 * zoom + padding * 2
        );
        
        // Text
        ctx.fillStyle = '#ffffff';
        ctx.globalAlpha = 0.95;
        ctx.fillText(displayLabel, node.x, node.y + node.radius + 16);
      }
      
      ctx.globalAlpha = 1;
    });

    ctx.restore();
  }, [nodes, edges, zoom, pan, selectedNode, hoveredNode, searchQuery, showLabels]);

  const handleCanvasMouseDown = (event: React.MouseEvent<HTMLCanvasElement>) => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const rect = canvas.getBoundingClientRect();
    const x = (event.clientX - rect.left - pan.x) / zoom;
    const y = (event.clientY - rect.top - pan.y) / zoom;

    const clickedNode = nodes.find(node => {
      const distance = Math.sqrt((x - node.x) ** 2 + (y - node.y) ** 2);
      return distance <= node.radius;
    });

    if (clickedNode) {
      setDraggedNode(clickedNode);
      setSelectedNode(clickedNode);
      setNodes(prevNodes => 
        prevNodes.map(n => 
          n.id === clickedNode.id 
            ? { ...n, fx: n.x, fy: n.y }
            : n
        )
      );
    } else {
      setIsDragging(true);
    }
  };

  const handleCanvasMouseMove = (event: React.MouseEvent<HTMLCanvasElement>) => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const rect = canvas.getBoundingClientRect();
    const x = (event.clientX - rect.left - pan.x) / zoom;
    const y = (event.clientY - rect.top - pan.y) / zoom;

    if (draggedNode) {
      setNodes(prevNodes =>
        prevNodes.map(n =>
          n.id === draggedNode.id
            ? { ...n, x, y, fx: x, fy: y, vx: 0, vy: 0 }
            : n
        )
      );
    } else if (isDragging) {
      setPan(prev => ({
        x: prev.x + event.movementX,
        y: prev.y + event.movementY,
      }));
    } else {
      const hoveredNode = nodes.find(node => {
        const distance = Math.sqrt((x - node.x) ** 2 + (y - node.y) ** 2);
        return distance <= node.radius;
      });
      setHoveredNode(hoveredNode || null);
    }
  };

  const handleCanvasMouseUp = () => {
    if (draggedNode) {
      setNodes(prevNodes =>
        prevNodes.map(n =>
          n.id === draggedNode.id
            ? { ...n, fx: undefined, fy: undefined }
            : n
        )
      );
      setDraggedNode(null);
    }
    setIsDragging(false);
  };

  const resetView = () => {
    setZoom(1);
    setPan({ x: 0, y: 0 });
  };

  const memoryTypes = ['all', 'episodic', 'semantic', 'procedural', 'working', 'core'];

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-purple-600 mx-auto mb-4"></div>
          <p className="text-slate-400">Loading knowledge graph...</p>
        </div>
      </div>
    );
  }

  if (!memories || memories.length === 0) {
    return (
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-3xl font-bold text-white flex items-center">
              <Network className="w-8 h-8 mr-3 text-purple-400" />
              Knowledge Graph
            </h2>
            <p className="text-slate-400 mt-1">
              Visualize memory relationships and connections
            </p>
          </div>
        </div>

        <Card className="bg-slate-800/50 border-slate-700 p-12">
          <div className="text-center">
            <Brain className="w-16 h-16 text-slate-600 mx-auto mb-4" />
            <h3 className="text-xl font-semibold text-white mb-2">
              No Memories Found
            </h3>
            <p className="text-slate-400 mb-6">
              Create some memories first to see the knowledge graph visualization
            </p>
            <Button onClick={() => window.location.href = '/admin/memories'} className="bg-purple-600 hover:bg-purple-700">
              Go to Memories
            </Button>
          </div>
        </Card>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-3xl font-bold text-white flex items-center gap-3">
            <Network className="w-8 h-8 text-purple-400" />
            Knowledge Graph
          </h2>
          <p className="text-slate-400 mt-1">
            Interactive force-directed graph visualization • {nodes.length} nodes • {edges.length} edges
          </p>
        </div>
        <div className="flex items-center gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={() => setShowStats(!showStats)}
            className="bg-slate-800 border-slate-700 text-white hover:bg-slate-700"
          >
            <BarChart3 className="w-4 h-4 mr-2" />
            {showStats ? 'Hide' : 'Show'} Analytics
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={loadMemoriesWrapper}
            className="bg-slate-800 border-slate-700 text-white hover:bg-slate-700"
          >
            <RefreshCw className="w-4 h-4" />
          </Button>
        </div>
      </div>

      {/* Controls Bar */}
      <div className="flex items-center gap-3">
        <div className="flex-1 relative">
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-slate-400" />
          <Input
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Search nodes by content..."
            className="pl-10 bg-slate-800 border-slate-700 text-white placeholder:text-slate-500"
          />
        </div>
        <Select value={showLabels} onValueChange={(v) => setShowLabels(v as typeof showLabels)}>
          <SelectTrigger className="w-44 bg-slate-800 border-slate-700 text-white">
            <SelectValue placeholder="Label display" />
          </SelectTrigger>
          <SelectContent className="bg-slate-800 border-slate-700">
            <SelectItem value="none" className="text-white hover:bg-slate-700">
              No Labels
            </SelectItem>
            <SelectItem value="selected" className="text-white hover:bg-slate-700">
              Selected Only
            </SelectItem>
            <SelectItem value="important" className="text-white hover:bg-slate-700">
              Important Nodes
            </SelectItem>
            <SelectItem value="all" className="text-white hover:bg-slate-700">
              All Labels
            </SelectItem>
          </SelectContent>
        </Select>
        <Select value={filterType} onValueChange={setFilterType}>
          <SelectTrigger className="w-40 bg-slate-800 border-slate-700 text-white">
            <Layers className="w-4 h-4 mr-2" />
            <SelectValue placeholder="Filter type" />
          </SelectTrigger>
          <SelectContent className="bg-slate-800 border-slate-700">
            {memoryTypes.map(type => (
              <SelectItem key={type} value={type} className="text-white hover:bg-slate-700">
                {type.charAt(0).toUpperCase() + type.slice(1)}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <Button
          variant="outline"
          size="icon"
          onClick={() => setZoom(Math.max(0.5, zoom - 0.1))}
          className="bg-slate-800 border-slate-700 text-white hover:bg-slate-700"
        >
          <ZoomOut className="w-4 h-4" />
        </Button>
        <Button
          variant="outline"
          size="icon"
          onClick={() => setZoom(Math.min(3, zoom + 0.1))}
          className="bg-slate-800 border-slate-700 text-white hover:bg-slate-700"
        >
          <ZoomIn className="w-4 h-4" />
        </Button>
        <Button
          variant="outline"
          size="icon"
          onClick={resetView}
          className="bg-slate-800 border-slate-700 text-white hover:bg-slate-700"
        >
          <Maximize2 className="w-4 h-4" />
        </Button>
      </div>

      {/* Graph Visualization */}
      <div className="grid grid-cols-1 lg:grid-cols-4 gap-6">
        <div className={cn("transition-all duration-300", showStats ? "lg:col-span-3" : "lg:col-span-4")}>
          <Card className="bg-slate-800/50 border-slate-700 p-4">
            <canvas
              ref={canvasRef}
              width={1200}
              height={800}
              className="w-full border border-slate-700 rounded-lg cursor-move bg-slate-900/50"
              onMouseDown={handleCanvasMouseDown}
              onMouseMove={handleCanvasMouseMove}
              onMouseUp={handleCanvasMouseUp}
              onMouseLeave={handleCanvasMouseUp}
              style={{ maxHeight: '800px' }}
            />
            <div className="mt-4 flex items-center justify-between">
              <div className="flex items-center gap-4 text-sm">
                <div className="flex items-center">
                  <div className="w-3 h-3 rounded-full bg-blue-500 mr-2"></div>
                  <span className="text-slate-300">Episodic</span>
                </div>
                <div className="flex items-center">
                  <div className="w-3 h-3 rounded-full bg-green-500 mr-2"></div>
                  <span className="text-slate-300">Semantic</span>
                </div>
                <div className="flex items-center">
                  <div className="w-3 h-3 rounded-full bg-yellow-500 mr-2"></div>
                  <span className="text-slate-300">Procedural</span>
                </div>
                <div className="flex items-center">
                  <div className="w-3 h-3 rounded-full bg-purple-500 mr-2"></div>
                  <span className="text-slate-300">Working</span>
                </div>
                <div className="flex items-center">
                  <div className="w-3 h-3 rounded-full bg-red-500 mr-2"></div>
                  <span className="text-slate-300">Core</span>
                </div>
              </div>
              <div className="flex items-center gap-2 text-xs text-slate-400">
                <Badge variant="outline" className="text-xs border-slate-600">
                  💡 Tip: Use &quot;Label Display&quot; to control label visibility
                </Badge>
                <span className="text-slate-500">
                  Drag nodes • Click to select • Scroll to zoom
                </span>
              </div>
            </div>
          </Card>
        </div>

        {/* Analytics Panel */}
        {showStats && (
          <div className="space-y-4">
            <Card className="bg-slate-800/50 border-slate-700 p-6">
              <h3 className="text-lg font-semibold text-white mb-4 flex items-center gap-2">
                <BarChart3 className="w-5 h-5 text-purple-400" />
                Graph Analytics
              </h3>
              {graphStats && (
                <div className="space-y-3">
                  <div className="flex justify-between items-center">
                    <span className="text-sm text-slate-400">Total Nodes</span>
                    <Badge variant="outline" className="text-white border-slate-600">
                      {graphStats.totalNodes}
                    </Badge>
                  </div>
                  <div className="flex justify-between items-center">
                    <span className="text-sm text-slate-400">Total Edges</span>
                    <Badge variant="outline" className="text-white border-slate-600">
                      {graphStats.totalEdges}
                    </Badge>
                  </div>
                  <div className="flex justify-between items-center">
                    <span className="text-sm text-slate-400">Avg Degree</span>
                    <Badge variant="outline" className="text-white border-slate-600">
                      {graphStats.avgDegree}
                    </Badge>
                  </div>
                  <div className="flex justify-between items-center">
                    <span className="text-sm text-slate-400">Max Degree</span>
                    <Badge variant="outline" className="text-white border-slate-600">
                      {graphStats.maxDegree}
                    </Badge>
                  </div>
                  <div className="flex justify-between items-center">
                    <span className="text-sm text-slate-400">Clusters</span>
                    <Badge variant="outline" className="text-white border-slate-600">
                      {graphStats.clusters}
                    </Badge>
                  </div>
                  <div className="flex justify-between items-center">
                    <span className="text-sm text-slate-400">Density</span>
                    <Badge variant="outline" className="text-white border-slate-600">
                      {graphStats.density}
                    </Badge>
                  </div>
                  <div className="flex justify-between items-center">
                    <span className="text-sm text-slate-400">Zoom Level</span>
                    <Badge variant="outline" className="text-white border-slate-600">
                      {(zoom * 100).toFixed(0)}%
                    </Badge>
                  </div>
                </div>
              )}
            </Card>

            {/* Node Details */}
            {selectedNode && (
              <Card className="bg-slate-800/50 border-slate-700 p-6">
                <h3 className="text-lg font-semibold text-white mb-4 flex items-center gap-2">
                  <Share2 className="w-5 h-5 text-purple-400" />
                  Node Details
                </h3>
                <div className="space-y-4">
                  <div>
                    <label className="text-sm font-medium text-slate-400">Type</label>
                    <Badge className="mt-1 bg-purple-600 text-white">
                      {selectedNode.type}
                    </Badge>
                  </div>
                  <div>
                    <label className="text-sm font-medium text-slate-400">Content</label>
                    <p className="text-sm text-slate-300 mt-1 break-words">
                      {selectedNode.content}
                    </p>
                  </div>
                  <div>
                    <label className="text-sm font-medium text-slate-400">Importance</label>
                    <div className="mt-1">
                      <div className="flex items-center gap-2">
                        <div className="flex-1 bg-slate-700 rounded-full h-2">
                          <div
                            className="bg-purple-500 h-2 rounded-full transition-all"
                            style={{ width: `${selectedNode.importance * 100}%` }}
                          ></div>
                        </div>
                        <span className="text-sm font-medium text-white">
                          {(selectedNode.importance * 100).toFixed(0)}%
                        </span>
                      </div>
                    </div>
                  </div>
                  <div>
                    <label className="text-sm font-medium text-slate-400">Connections</label>
                    <p className="text-sm text-slate-300 mt-1">
                      {edges.filter(e => e.source === selectedNode.id || e.target === selectedNode.id).length} edges
                    </p>
                  </div>
                </div>
              </Card>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
