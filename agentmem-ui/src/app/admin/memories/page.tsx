/**
 * Enhanced Memories Management Page
 * 
 * Features:
 * - Table view with pagination
 * - Advanced filtering and search
 * - Importance score display (new)
 * - Decay status visualization (new)
 * - Time range selector (new)
 * - Search analytics panel (new)
 */

'use client';

import React, { useState, useEffect } from 'react';
import { Brain, Search, Trash2, Filter, Plus, RefreshCw, Eye, ChevronLeft, ChevronRight } from 'lucide-react';
import Link from 'next/link';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { 
  Table, 
  TableBody, 
  TableCell, 
  TableHead, 
  TableHeader, 
  TableRow 
} from '@/components/ui/table';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { Skeleton } from '@/components/ui/skeleton';
import { useToast } from '@/hooks/use-toast';
import { apiClient, Memory, Agent } from '@/lib/api-client';

// Import new components
import { ImportanceBadge } from '@/components/ui/importance-badge';
import { ImportanceLevel } from '@/components/ui/importance-badge';
import { DecayProgress } from '@/components/ui/decay-progress';
import { DecayStatusBadge } from '@/components/ui/decay-progress';
import { TimeRangeSelector, TimeRange, timeRangeToTimestamp } from '@/components/ui/time-range-selector';
import { SearchAnalyticsPanel } from '@/components/charts/search-analytics-panel';

// Pagination component
interface PaginationProps {
  currentPage: number;
  totalPages: number;
  onPageChange: (page: number) => void;
}

function Pagination({ currentPage, totalPages, onPageChange }: PaginationProps) {
  return (
    <div className="flex items-center justify-between px-2 py-4">
      <div className="text-sm text-gray-700 dark:text-gray-300">
        Page {currentPage + 1} of {totalPages}
      </div>
      <div className="flex gap-2">
        <Button
          variant="outline"
          size="sm"
          onClick={() => onPageChange(currentPage - 1)}
          disabled={currentPage <= 0}
        >
          <ChevronLeft className="w-4 h-4 mr-1" />
          Previous
        </Button>
        <Button
          variant="outline"
          size="sm"
          onClick={() => onPageChange(currentPage + 1)}
          disabled={currentPage >= totalPages - 1}
        >
          Next
          <ChevronRight className="w-4 h-4 ml-1" />
        </Button>
      </div>
    </div>
  );
}

export default function MemoriesPageEnhanced() {
  const { toast } = useToast();
  
  // State
  const [memories, setMemories] = useState<Memory[]>([]);
  const [agents, setAgents] = useState<Agent[]>([]);
  const [loading, setLoading] = useState(true);
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedAgentId, setSelectedAgentId] = useState<string>('all');
  const [selectedType, setSelectedType] = useState<string>('all');
  const [selectedImportance, setSelectedImportance] = useState<string>('all');
  const [timeRange, setTimeRange] = useState<TimeRange>('all');
  
  // Pagination state
  const [currentPage, setCurrentPage] = useState(0);
  const [itemsPerPage] = useState(20);
  const [totalPages, setTotalPages] = useState(0);
  const [totalCount, setTotalCount] = useState(0);
  
  // Add Memory Dialog state
  const [addDialogOpen, setAddDialogOpen] = useState(false);
  const [newMemory, setNewMemory] = useState({
    agent_id: '',
    content: '',
    memory_type: 'Semantic',
    importance: 0.8,
  });
  const [submitting, setSubmitting] = useState(false);
  
  // Show analytics panel
  const [showAnalytics, setShowAnalytics] = useState(true);
  
  // Load data when page, agent, or type changes
  useEffect(() => {
    loadData();
  }, [currentPage, selectedAgentId, selectedType, selectedImportance, timeRange]);
  
  const loadData = async () => {
    try {
      setLoading(true);
      
      const timestamp = timeRangeToTimestamp(timeRange);
      
      const [agentsData, memoriesResponse] = await Promise.all([
        apiClient.getAgents(),
        apiClient.getAllMemories(
          currentPage, 
          itemsPerPage,
          selectedAgentId !== 'all' ? selectedAgentId : undefined,
          selectedType !== 'all' ? selectedType : undefined
        ),
      ]);
      
      setAgents(agentsData || []);
      
      // Filter by importance and time if needed
      let filteredMemories = memoriesResponse?.memories || [];
      
      if (selectedImportance !== 'all') {
        const [min, max] = selectedImportance === 'high' 
          ? [0.7, 1.0]
          : selectedImportance === 'medium'
          ? [0.4, 0.7]
          : [0.0, 0.4];
        filteredMemories = filteredMemories.filter((m: any) => {
          const score = m.importance_score || m.metadata?.importance || 0;
          return score >= min && score < max;
        });
      }
      
      if (timestamp) {
        filteredMemories = filteredMemories.filter((m: any) => {
          const createdAt = new Date(m.created_at).getTime();
          return createdAt >= timestamp;
        });
      }
      
      setMemories(filteredMemories);
      
      if (memoriesResponse?.pagination) {
        setTotalPages(memoriesResponse.pagination.total_pages);
        setTotalCount(memoriesResponse.pagination.total);
      }
    } catch (err) {
      console.error('Load error:', err);
      setAgents([]);
      setMemories([]);
    } finally {
      setLoading(false);
    }
  };
  
  const handleAgentChange = async (agentId: string) => {
    setSelectedAgentId(agentId);
    setCurrentPage(0);
  };
  
  const handleTypeChange = async (type: string) => {
    setSelectedType(type);
    setCurrentPage(0);
  };
  
  const handleAddMemory = async () => {
    if (!newMemory.content.trim()) return;
    
    setSubmitting(true);
    try {
      await apiClient.createMemory({
        content: newMemory.content,
        memory_type: newMemory.memory_type as any,
        agent_id: newMemory.agent_id || '',
        importance: newMemory.importance,
      });
      
      toast({
        title: 'Memory added',
        description: 'Your memory has been saved successfully.',
      });
      
      setAddDialogOpen(false);
      setNewMemory({ agent_id: '', content: '', memory_type: 'Semantic', importance: 0.8 });
      loadData();
    } catch (err) {
      toast({
        title: 'Failed to add memory',
        description: err instanceof Error ? err.message : 'Unknown error',
        variant: 'destructive',
      });
    } finally {
      setSubmitting(false);
    }
  };
  
  const handleDeleteMemory = async (memoryId: string) => {
    try {
      await apiClient.deleteMemory(memoryId);
      toast({
        title: 'Memory deleted',
        description: 'The memory has been removed.',
      });
      loadData();
    } catch (err) {
      toast({
        title: 'Failed to delete memory',
        description: err instanceof Error ? err.message : 'Unknown error',
        variant: 'destructive',
      });
    }
  };
  
  // Get importance score from memory
  const getImportanceScore = (memory: Memory): number => {
    const anyMem = memory as any;
    return anyMem.importance_score || anyMem.metadata?.importance || 0.5;
  };
  
  // Get decay score from memory
  const getDecayScore = (memory: Memory): number => {
    const anyMem = memory as any;
    return anyMem.metadata?.decay_score ?? anyMem.metadata?.health ?? 1.0;
  };

  return (
    <div className="container mx-auto py-8 space-y-4">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <Brain className="w-8 h-8 text-blue-500" />
          <h1 className="text-2xl font-bold">Memory Management</h1>
        </div>
        <div className="flex gap-2">
          <Button
            variant="outline"
            onClick={() => setShowAnalytics(!showAnalytics)}
          >
            <Eye className="w-4 h-4 mr-2" />
            {showAnalytics ? 'Hide' : 'Show'} Analytics
          </Button>
          <Button onClick={() => setAddDialogOpen(true)}>
            <Plus className="w-4 h-4 mr-2" />
            Add Memory
          </Button>
          <Button variant="outline" onClick={loadData}>
            <RefreshCw className="w-4 h-4" />
          </Button>
        </div>
      </div>
      
      {/* Search Analytics Panel */}
      {showAnalytics && (
        <SearchAnalyticsPanel />
      )}
      
      {/* Filters Card */}
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-lg flex items-center gap-2">
            <Filter className="w-5 h-5" />
            Filters
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
            {/* Agent Filter */}
            <div className="space-y-1">
              <Label className="text-xs text-gray-500">Agent</Label>
              <Select value={selectedAgentId} onValueChange={handleAgentChange}>
                <SelectTrigger>
                  <SelectValue placeholder="All Agents" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">All Agents</SelectItem>
                  {agents.map((agent) => (
                    <SelectItem key={agent.id} value={agent.id}>
                      {agent.name || `Agent ${agent.id.slice(0, 8)}`}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            
            {/* Type Filter */}
            <div className="space-y-1">
              <Label className="text-xs text-gray-500">Type</Label>
              <Select value={selectedType} onValueChange={handleTypeChange}>
                <SelectTrigger>
                  <SelectValue placeholder="All Types" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">All Types</SelectItem>
                  <SelectItem value="Episodic">Episodic</SelectItem>
                  <SelectItem value="Semantic">Semantic</SelectItem>
                  <SelectItem value="Procedural">Procedural</SelectItem>
                  <SelectItem value="Working">Working</SelectItem>
                </SelectContent>
              </Select>
            </div>
            
            {/* Importance Filter (NEW) */}
            <div className="space-y-1">
              <Label className="text-xs text-gray-500">Importance</Label>
              <Select value={selectedImportance} onValueChange={setSelectedImportance}>
                <SelectTrigger>
                  <SelectValue placeholder="All Levels" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">All Levels</SelectItem>
                  <SelectItem value="high">High (0.7+)</SelectItem>
                  <SelectItem value="medium">Medium (0.4-0.7)</SelectItem>
                  <SelectItem value="low">Low (0-0.4)</SelectItem>
                </SelectContent>
              </Select>
            </div>
            
            {/* Time Range Filter (NEW) */}
            <div className="space-y-1">
              <Label className="text-xs text-gray-500">Time Range</Label>
              <TimeRangeSelector
                value={timeRange}
                onChange={setTimeRange}
                size="sm"
                showLabel={false}
              />
            </div>
          </div>
        </CardContent>
      </Card>
      
      {/* Memories Table Card */}
      <Card>
        <CardContent className="p-0">
          {loading ? (
            <div className="p-8 space-y-4">
              {[1, 2, 3, 4, 5].map((i) => (
                <Skeleton key={i} className="h-12 w-full" />
              ))}
            </div>
          ) : memories.length === 0 ? (
            <div className="p-8 text-center text-gray-500">
              <Brain className="w-12 h-12 mx-auto mb-4 opacity-50" />
              <p>No memories found</p>
              <p className="text-sm">Try adjusting your filters or add a new memory</p>
            </div>
          ) : (
            <>
              <div className="overflow-x-auto">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead className="w-[300px]">Content</TableHead>
                      <TableHead>Type</TableHead>
                      <TableHead>Agent</TableHead>
                      <TableHead>Importance</TableHead>
                      <TableHead>Health</TableHead>
                      <TableHead>Created</TableHead>
                      <TableHead className="text-right">Actions</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {memories.map((memory) => {
                      const anyMem = memory as any;
                      return (
                        <TableRow key={memory.id}>
                          <TableCell>
                            <div className="max-w-[280px]">
                              <p className="truncate text-sm">
                                {anyMem.content?.substring(0, 80) || 'N/A'}
                                {(anyMem.content?.length || 0) > 80 ? '...' : ''}
                              </p>
                            </div>
                          </TableCell>
                          <TableCell>
                            <span className="px-2 py-1 text-xs rounded-full bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200">
                              {anyMem.memory_type || 'Unknown'}
                            </span>
                          </TableCell>
                          <TableCell>
                            <span className="text-sm text-gray-600 dark:text-gray-400">
                              {anyMem.agent_id ? anyMem.agent_id.slice(0, 8) + '...' : 'Global'}
                            </span>
                          </TableCell>
                          {/* Importance Column (NEW) */}
                          <TableCell>
                            <div className="flex items-center gap-2">
                              <ImportanceBadge score={getImportanceScore(memory)} size="sm" />
                              <ImportanceLevel score={getImportanceScore(memory)} size="sm" />
                            </div>
                          </TableCell>
                          {/* Health/Decay Column (NEW) */}
                          <TableCell>
                            <div className="w-[100px]">
                              <DecayProgress decayScore={getDecayScore(memory)} size="sm" />
                            </div>
                          </TableCell>
                          <TableCell>
                            <span className="text-sm text-gray-500">
                              {anyMem.created_at 
                                ? new Date(anyMem.created_at).toLocaleDateString()
                                : 'N/A'}
                            </span>
                          </TableCell>
                          <TableCell className="text-right">
                            <div className="flex justify-end gap-1">
                              <Button
                                variant="ghost"
                                size="sm"
                                onClick={() => handleDeleteMemory(memory.id)}
                              >
                                <Trash2 className="w-4 h-4" />
                              </Button>
                            </div>
                          </TableCell>
                        </TableRow>
                      );
                    })}
                  </TableBody>
                </Table>
              </div>
              
              {/* Pagination */}
              {totalPages > 1 && (
                <div className="mt-4">
                  <Pagination
                    currentPage={currentPage}
                    totalPages={totalPages}
                    onPageChange={setCurrentPage}
                  />
                  <div className="text-center text-sm text-gray-500 mt-2">
                    Showing {memories.length} of {totalCount} memories
                  </div>
                </div>
              )}
            </>
          )}
        </CardContent>
      </Card>
      
      {/* Add Memory Dialog */}
      <Dialog open={addDialogOpen} onOpenChange={setAddDialogOpen}>
        <DialogContent className="sm:max-w-[600px]">
          <DialogHeader>
            <DialogTitle>Add New Memory</DialogTitle>
            <DialogDescription>
              Create a new memory for an agent.
            </DialogDescription>
          </DialogHeader>
          
          <div className="grid gap-4 py-4">
            <div className="grid gap-2">
              <Label htmlFor="agent">Agent (Optional)</Label>
              <Select
                value={newMemory.agent_id}
                onValueChange={(value) => setNewMemory({ ...newMemory, agent_id: value })}
              >
                <SelectTrigger id="agent">
                  <SelectValue placeholder="Select an agent" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="__none__">No Agent (Global)</SelectItem>
                  {agents.map((agent) => (
                    <SelectItem key={agent.id} value={agent.id}>
                      {agent.name || `Agent ${agent.id.slice(0, 8)}`}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            
            <div className="grid gap-2">
              <Label htmlFor="type">Memory Type</Label>
              <Select
                value={newMemory.memory_type}
                onValueChange={(value) => setNewMemory({ ...newMemory, memory_type: value })}
              >
                <SelectTrigger id="type">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="Episodic">Episodic</SelectItem>
                  <SelectItem value="Semantic">Semantic</SelectItem>
                  <SelectItem value="Procedural">Procedural</SelectItem>
                  <SelectItem value="Working">Working</SelectItem>
                </SelectContent>
              </Select>
            </div>
            
            <div className="grid gap-2">
              <Label htmlFor="importance">
                Importance: {newMemory.importance.toFixed(2)}
              </Label>
              <input
                id="importance"
                type="range"
                min="0"
                max="1"
                step="0.1"
                value={newMemory.importance}
                onChange={(e) => setNewMemory({ ...newMemory, importance: parseFloat(e.target.value) })}
                className="w-full"
              />
            </div>
            
            <div className="grid gap-2">
              <Label htmlFor="content">Content *</Label>
              <Textarea
                id="content"
                placeholder="Enter memory content..."
                value={newMemory.content}
                onChange={(e) => setNewMemory({ ...newMemory, content: e.target.value })}
                className="min-h-[150px]"
              />
            </div>
          </div>
          
          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              onClick={() => setAddDialogOpen(false)}
              disabled={submitting}
            >
              Cancel
            </Button>
            <Button
              type="button"
              onClick={handleAddMemory}
              disabled={submitting || !newMemory.content.trim()}
            >
              {submitting ? 'Adding...' : 'Add Memory'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
