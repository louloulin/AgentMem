/**
 * Enhanced Memories Management Page
 * 
 * Features:
 * - Table view with pagination
 * - Advanced filtering and search
 * - Toast notifications
 * - Skeleton loading states
 * - Supabase-style modern UI
 */

'use client';

import React, { useState, useEffect } from 'react';
import { Brain, Search, Trash2, Filter, Plus, RefreshCw } from 'lucide-react';
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
        Page {currentPage} of {totalPages}
      </div>
      <div className="flex gap-2">
        <Button
          variant="outline"
          size="sm"
          onClick={() => onPageChange(currentPage - 1)}
          disabled={currentPage <= 1}
        >
          Previous
        </Button>
        <Button
          variant="outline"
          size="sm"
          onClick={() => onPageChange(currentPage + 1)}
          disabled={currentPage >= totalPages}
        >
          Next
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
  
  // Pagination state
  const [currentPage, setCurrentPage] = useState(1);
  const [itemsPerPage] = useState(10);
  
  // Add Memory Dialog state
  const [addDialogOpen, setAddDialogOpen] = useState(false);
  const [newMemory, setNewMemory] = useState({
    agent_id: '',
    content: '',
    memory_type: 'Semantic',
    importance: 0.8,
  });
  const [submitting, setSubmitting] = useState(false);
  
  // Load data on mount
  useEffect(() => {
    loadData();
  }, []);
  
  const loadData = async () => {
    try {
      setLoading(true);
      const agentsData = await apiClient.getAgents();
      setAgents(agentsData || []);
      
      // Load memories for first agent if available
      if (agentsData && agentsData.length > 0) {
        const memoriesData = await apiClient.getMemories(agentsData[0].id);
        setMemories(memoriesData || []);
      } else {
        setMemories([]);
      }
      
      toast({
        title: "Data loaded",
        description: `Loaded ${agentsData?.length || 0} agents`,
      });
    } catch (err) {
      setAgents([]);
      setMemories([]);
      toast({
        title: "Failed to load data",
        description: err instanceof Error ? err.message : 'Unknown error',
        variant: "destructive",
      });
    } finally {
      setLoading(false);
    }
  };
  
  const handleAgentChange = async (agentId: string) => {
    setSelectedAgentId(agentId);
    setCurrentPage(1);
    
    if (agentId === 'all') {
      setMemories([]);
      return;
    }
    
    try {
      setLoading(true);
      const data = await apiClient.getMemories(agentId);
      setMemories(data || []);
      
      toast({
        title: "Memories loaded",
        description: `Found ${data?.length || 0} memories`,
      });
    } catch (err) {
      setMemories([]);
      toast({
        title: "Failed to load memories",
        description: err instanceof Error ? err.message : 'Unknown error',
        variant: "destructive",
      });
    } finally {
      setLoading(false);
    }
  };
  
  const handleAddMemory = async () => {
    if (!newMemory.agent_id) {
      toast({
        title: "Validation Error",
        description: "Please select an agent",
        variant: "destructive",
      });
      return;
    }
    
    if (!newMemory.content.trim()) {
      toast({
        title: "Validation Error",
        description: "Memory content cannot be empty",
        variant: "destructive",
      });
      return;
    }
    
    try {
      setSubmitting(true);
      
      await apiClient.createMemory({
        agent_id: newMemory.agent_id,
        content: newMemory.content,
        memory_type: newMemory.memory_type,
        importance: newMemory.importance,
      });
      
      toast({
        title: "Memory Added",
        description: "Memory has been created successfully",
      });
      
      // Close dialog and reset form
      setAddDialogOpen(false);
      setNewMemory({
        agent_id: '',
        content: '',
        memory_type: 'Semantic',
        importance: 0.8,
      });
      
      // Reload memories if currently viewing the same agent
      if (selectedAgentId === newMemory.agent_id) {
        await handleAgentChange(newMemory.agent_id);
      }
    } catch (err) {
      toast({
        title: "Failed to add memory",
        description: err instanceof Error ? err.message : 'Unknown error',
        variant: "destructive",
      });
    } finally {
      setSubmitting(false);
    }
  };
  
  const handleSearch = async () => {
    if (!searchQuery.trim()) {
      return;
    }
    
    try {
      setLoading(true);
      setCurrentPage(1);
      const data = await apiClient.searchMemories(
        searchQuery,
        selectedAgentId !== 'all' ? selectedAgentId : undefined
      );
      setMemories(data || []);
      
      toast({
        title: "Search completed",
        description: `Found ${data?.length || 0} results`,
      });
    } catch (err) {
      setMemories([]);
      toast({
        title: "Search failed",
        description: err instanceof Error ? err.message : 'Unknown error',
        variant: "destructive",
      });
    } finally {
      setLoading(false);
    }
  };
  
  const handleDeleteMemory = async (memoryId: string) => {
    try {
      await apiClient.deleteMemory(memoryId);
      setMemories((prev) => (prev || []).filter((m) => m.id !== memoryId));
      
      toast({
        title: "Memory deleted",
        description: "Memory has been successfully deleted",
      });
    } catch (err) {
      toast({
        title: "Failed to delete memory",
        description: err instanceof Error ? err.message : 'Unknown error',
        variant: "destructive",
      });
    }
  };
  
  // Filter memories by type
  const filteredMemories = (memories || []).filter((memory) => {
    if (selectedType && selectedType !== 'all') {
      return memory.memory_type === selectedType;
    }
    return true;
  });
  
  // Paginate filtered memories
  const totalPages = Math.ceil(filteredMemories.length / itemsPerPage);
  const paginatedMemories = filteredMemories.slice(
    (currentPage - 1) * itemsPerPage,
    currentPage * itemsPerPage
  );
  
  // Format date
  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleString();
  };
  
  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-3xl font-bold text-gray-900 dark:text-white flex items-center gap-2">
            <Brain className="w-8 h-8" />
            Memories
          </h2>
          <p className="text-gray-600 dark:text-gray-400 mt-1">
            View and manage agent memories
          </p>
        </div>
        <div className="flex gap-2">
          <Button onClick={loadData} variant="outline" size="sm">
            <RefreshCw className="w-4 h-4 mr-2" />
            Refresh
          </Button>
          <Button 
            size="sm"
            onClick={() => setAddDialogOpen(true)}
          >
            <Plus className="w-4 h-4 mr-2" />
            Add Memory
          </Button>
        </div>
      </div>
      
      {/* Filters */}
      <Card>
        <CardContent className="pt-6">
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            {/* Agent Filter */}
            <div className="space-y-2">
              <label className="text-sm font-medium text-gray-700 dark:text-gray-300">
                Agent
              </label>
              <Select value={selectedAgentId} onValueChange={handleAgentChange}>
                <SelectTrigger>
                  <SelectValue placeholder="Select agent" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">All Agents</SelectItem>
                  {(agents || []).map((agent) => (
                    <SelectItem key={agent.id} value={agent.id}>
                      {agent.name || agent.id}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            
            {/* Type Filter */}
            <div className="space-y-2">
              <label className="text-sm font-medium text-gray-700 dark:text-gray-300">
                Memory Type
              </label>
              <Select value={selectedType} onValueChange={setSelectedType}>
                <SelectTrigger>
                  <SelectValue placeholder="Select type" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">All Types</SelectItem>
                  <SelectItem value="episodic">Episodic</SelectItem>
                  <SelectItem value="semantic">Semantic</SelectItem>
                  <SelectItem value="procedural">Procedural</SelectItem>
                  <SelectItem value="working">Working</SelectItem>
                </SelectContent>
              </Select>
            </div>
            
            {/* Search */}
            <div className="space-y-2">
              <label className="text-sm font-medium text-gray-700 dark:text-gray-300">
                Search
              </label>
              <div className="flex gap-2">
                <Input
                  placeholder="Search memories..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  onKeyPress={(e) => e.key === 'Enter' && handleSearch()}
                />
                <Button onClick={handleSearch} size="sm">
                  <Search className="w-4 h-4" />
                </Button>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>
      
      {/* Table */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg font-semibold">
            {filteredMemories.length} Memories
          </CardTitle>
        </CardHeader>
        <CardContent>
          {loading ? (
            // Loading skeleton
            <div className="space-y-2">
              {[...Array(5)].map((_, i) => (
                <Skeleton key={i} className="h-12 w-full" />
              ))}
            </div>
          ) : filteredMemories.length === 0 ? (
            // Empty state
            <div className="text-center py-12">
              <Brain className="w-12 h-12 mx-auto text-gray-400 mb-4" />
              <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-2">
                No memories found
              </h3>
              <p className="text-gray-600 dark:text-gray-400">
                Select an agent or adjust your filters
              </p>
            </div>
          ) : (
            <>
              {/* Table */}
              <div className="rounded-md border">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead className="w-[40%]">Content</TableHead>
                      <TableHead>Type</TableHead>
                      <TableHead>Agent</TableHead>
                      <TableHead>Created</TableHead>
                      <TableHead className="text-right">Actions</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {paginatedMemories.map((memory) => (
                      <TableRow key={memory.id} className="hover:bg-gray-50 dark:hover:bg-gray-800/50">
                        <TableCell className="font-medium">
                          <div className="max-w-md truncate" title={memory.content}>
                            {memory.content}
                          </div>
                        </TableCell>
                        <TableCell>
                          <span className="inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium bg-blue-100 text-blue-800 dark:bg-blue-900/20 dark:text-blue-400">
                            {memory.memory_type || 'Unknown'}
                          </span>
                        </TableCell>
                        <TableCell>
                          {(agents || []).find((a) => a.id === memory.agent_id)?.name || 'Unknown'}
                        </TableCell>
                        <TableCell className="text-sm text-gray-600 dark:text-gray-400">
                          {formatDate(memory.created_at)}
                        </TableCell>
                        <TableCell className="text-right">
                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => handleDeleteMemory(memory.id)}
                            className="hover:bg-red-50 hover:text-red-600 dark:hover:bg-red-900/20 dark:hover:text-red-400"
                          >
                            <Trash2 className="w-4 h-4" />
                          </Button>
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </div>
              
              {/* Pagination */}
              {totalPages > 1 && (
                <Pagination
                  currentPage={currentPage}
                  totalPages={totalPages}
                  onPageChange={setCurrentPage}
                />
              )}
            </>
          )}
        </CardContent>
      </Card>
    </div>
  );
}

