/**
 * Memory Knowledge Graph Page
 * 
 * Visualizes memory relationships as an interactive knowledge graph
 * Features:
 * - Interactive graph visualization
 * - Node filtering by type
 * - Relationship exploration
 * - Real-time updates
 */

'use client';

import React, { useState, useEffect, useRef } from 'react';
import { Network, Brain, Filter, ZoomIn, ZoomOut, Maximize2 } from 'lucide-react';
import { Card } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { apiClient, Memory } from '@/lib/api-client';

interface GraphNode {
  id: string;
  label: string;
  type: string;
  importance: number;
  x?: number;
  y?: number;
}

interface GraphEdge {
  source: string;
  target: string;
  type: string;
}

export default function GraphPage() {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [memories, setMemories] = useState<Memory[]>([]);
  const [nodes, setNodes] = useState<GraphNode[]>([]);
  const [edges, setEdges] = useState<GraphEdge[]>([]);
  const [selectedNode, setSelectedNode] = useState<GraphNode | null>(null);
  const [filterType, setFilterType] = useState<string>('all');
  const [loading, setLoading] = useState(true);
  const [zoom, setZoom] = useState(1);

  useEffect(() => {
    loadMemories();
  }, []);

  useEffect(() => {
    if (memories && memories.length > 0) {
      buildGraph();
    }
  }, [memories, filterType]);

  useEffect(() => {
    if (nodes && nodes.length > 0) {
      drawGraph();
    }
  }, [nodes, edges, zoom, selectedNode]);

  const loadMemories = async () => {
    try {
      setLoading(true);
      const allMemories = await apiClient.searchMemories('');
      setMemories(allMemories || []);
    } catch (error) {
      setMemories([]);
      console.error('Failed to load memories:', error);
    } finally {
      setLoading(false);
    }
  };

  const buildGraph = () => {
    // Filter memories by type
    const filteredMemories = filterType === 'all'
      ? (memories || [])
      : (memories || []).filter(m => m.memory_type === filterType);

    // Create nodes
    const graphNodes: GraphNode[] = filteredMemories.map((memory, index) => ({
      id: memory.id,
      label: memory.content.substring(0, 30) + '...',
      type: memory.memory_type,
      importance: memory.importance,
      x: Math.cos(index * 2 * Math.PI / filteredMemories.length) * 200 + 400,
      y: Math.sin(index * 2 * Math.PI / filteredMemories.length) * 200 + 300,
    }));

    // Create edges based on content similarity (simplified)
    const graphEdges: GraphEdge[] = [];
    for (let i = 0; i < graphNodes.length; i++) {
      for (let j = i + 1; j < graphNodes.length; j++) {
        const memory1 = filteredMemories[i];
        const memory2 = filteredMemories[j];
        
        // Simple similarity check (in production, use proper similarity calculation)
        const words1 = memory1.content.toLowerCase().split(' ');
        const words2 = memory2.content.toLowerCase().split(' ');
        const commonWords = words1.filter(w => words2.includes(w) && w.length > 3);
        
        if (commonWords.length > 2) {
          graphEdges.push({
            source: graphNodes[i].id,
            target: graphNodes[j].id,
            type: 'related',
          });
        }
      }
    }

    setNodes(graphNodes);
    setEdges(graphEdges);
  };

  const drawGraph = () => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    // Clear canvas
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    ctx.save();
    ctx.scale(zoom, zoom);

    // Draw edges
    ctx.strokeStyle = '#94a3b8';
    ctx.lineWidth = 1;
    edges.forEach(edge => {
      const sourceNode = nodes.find(n => n.id === edge.source);
      const targetNode = nodes.find(n => n.id === edge.target);
      if (sourceNode && targetNode && sourceNode.x && sourceNode.y && targetNode.x && targetNode.y) {
        ctx.beginPath();
        ctx.moveTo(sourceNode.x, sourceNode.y);
        ctx.lineTo(targetNode.x, targetNode.y);
        ctx.stroke();
      }
    });

    // Draw nodes
    nodes.forEach(node => {
      if (!node.x || !node.y) return;

      const isSelected = selectedNode?.id === node.id;
      const radius = 8 + node.importance * 12;

      // Node circle
      ctx.beginPath();
      ctx.arc(node.x, node.y, radius, 0, 2 * Math.PI);
      
      // Color by type
      const colors: Record<string, string> = {
        episodic: '#3b82f6',
        semantic: '#10b981',
        procedural: '#f59e0b',
        working: '#8b5cf6',
        core: '#ef4444',
      };
      ctx.fillStyle = colors[node.type] || '#6b7280';
      ctx.fill();

      if (isSelected) {
        ctx.strokeStyle = '#ffffff';
        ctx.lineWidth = 3;
        ctx.stroke();
      }

      // Node label
      ctx.fillStyle = '#ffffff';
      ctx.font = '12px sans-serif';
      ctx.textAlign = 'center';
      ctx.fillText(node.label, node.x, node.y + radius + 15);
    });

    ctx.restore();
  };

  const handleCanvasClick = (event: React.MouseEvent<HTMLCanvasElement>) => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const rect = canvas.getBoundingClientRect();
    const x = (event.clientX - rect.left) / zoom;
    const y = (event.clientY - rect.top) / zoom;

    // Find clicked node
    const clickedNode = nodes.find(node => {
      if (!node.x || !node.y) return false;
      const radius = 8 + node.importance * 12;
      const distance = Math.sqrt((x - node.x) ** 2 + (y - node.y) ** 2);
      return distance <= radius;
    });

    setSelectedNode(clickedNode || null);
  };

  const memoryTypes = ['all', 'episodic', 'semantic', 'procedural', 'working', 'core'];

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto mb-4"></div>
          <p className="text-gray-600 dark:text-gray-400">Loading knowledge graph...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-3xl font-bold text-gray-900 dark:text-white flex items-center">
            <Network className="w-8 h-8 mr-3 text-blue-600" />
            Knowledge Graph
          </h2>
          <p className="text-gray-600 dark:text-gray-400 mt-1">
            Visualize memory relationships and connections
          </p>
        </div>
        <div className="flex items-center space-x-2">
          <Select value={filterType} onValueChange={setFilterType}>
            <SelectTrigger className="w-40">
              <SelectValue placeholder="Filter by type" />
            </SelectTrigger>
            <SelectContent>
              {memoryTypes.map(type => (
                <SelectItem key={type} value={type}>
                  {type.charAt(0).toUpperCase() + type.slice(1)}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <Button
            variant="outline"
            size="icon"
            onClick={() => setZoom(Math.max(0.5, zoom - 0.1))}
          >
            <ZoomOut className="w-4 h-4" />
          </Button>
          <Button
            variant="outline"
            size="icon"
            onClick={() => setZoom(Math.min(2, zoom + 0.1))}
          >
            <ZoomIn className="w-4 h-4" />
          </Button>
          <Button
            variant="outline"
            size="icon"
            onClick={() => setZoom(1)}
          >
            <Maximize2 className="w-4 h-4" />
          </Button>
        </div>
      </div>

      {/* Graph Visualization */}
      <div className="grid grid-cols-1 lg:grid-cols-4 gap-6">
        <div className="lg:col-span-3">
          <Card className="p-4">
            <canvas
              ref={canvasRef}
              width={800}
              height={600}
              className="w-full border border-gray-200 dark:border-gray-700 rounded-lg cursor-pointer"
              onClick={handleCanvasClick}
            />
            <div className="mt-4 flex items-center justify-center space-x-4 text-sm">
              <div className="flex items-center">
                <div className="w-3 h-3 rounded-full bg-blue-600 mr-2"></div>
                <span>Episodic</span>
              </div>
              <div className="flex items-center">
                <div className="w-3 h-3 rounded-full bg-green-600 mr-2"></div>
                <span>Semantic</span>
              </div>
              <div className="flex items-center">
                <div className="w-3 h-3 rounded-full bg-yellow-600 mr-2"></div>
                <span>Procedural</span>
              </div>
              <div className="flex items-center">
                <div className="w-3 h-3 rounded-full bg-purple-600 mr-2"></div>
                <span>Working</span>
              </div>
              <div className="flex items-center">
                <div className="w-3 h-3 rounded-full bg-red-600 mr-2"></div>
                <span>Core</span>
              </div>
            </div>
          </Card>
        </div>

        {/* Node Details */}
        <div>
          <Card className="p-6">
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
              {selectedNode ? 'Node Details' : 'Graph Statistics'}
            </h3>
            {selectedNode ? (
              <div className="space-y-4">
                <div>
                  <label className="text-sm font-medium text-gray-600 dark:text-gray-400">
                    Type
                  </label>
                  <Badge className="mt-1">
                    {selectedNode.type}
                  </Badge>
                </div>
                <div>
                  <label className="text-sm font-medium text-gray-600 dark:text-gray-400">
                    Content
                  </label>
                  <p className="text-sm text-gray-700 dark:text-gray-300 mt-1">
                    {selectedNode.label}
                  </p>
                </div>
                <div>
                  <label className="text-sm font-medium text-gray-600 dark:text-gray-400">
                    Importance
                  </label>
                  <div className="mt-1">
                    <div className="flex items-center space-x-2">
                      <div className="flex-1 bg-gray-200 dark:bg-gray-700 rounded-full h-2">
                        <div
                          className="bg-blue-600 h-2 rounded-full"
                          style={{ width: `${selectedNode.importance * 100}%` }}
                        ></div>
                      </div>
                      <span className="text-sm font-medium">
                        {(selectedNode.importance * 100).toFixed(0)}%
                      </span>
                    </div>
                  </div>
                </div>
              </div>
            ) : (
              <div className="space-y-3">
                <div className="flex justify-between">
                  <span className="text-sm text-gray-600 dark:text-gray-400">Total Nodes</span>
                  <span className="text-sm font-semibold">{nodes.length}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-sm text-gray-600 dark:text-gray-400">Total Edges</span>
                  <span className="text-sm font-semibold">{edges.length}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-sm text-gray-600 dark:text-gray-400">Zoom Level</span>
                  <span className="text-sm font-semibold">{(zoom * 100).toFixed(0)}%</span>
                </div>
              </div>
            )}
          </Card>
        </div>
      </div>
    </div>
  );
}

