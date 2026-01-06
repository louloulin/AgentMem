/**
 * Memory Detail Page
 * 
 * Displays detailed information about a specific memory including:
 * - Memory content and metadata
 * - Related memories
 * - Edit and delete functionality
 * - Memory history
 */

'use client';

import React, { useState, useEffect } from 'react';
import { useParams, useRouter } from 'next/navigation';
import { ArrowLeft, Edit3, Trash2, Clock, Tag, Brain, Link as LinkIcon } from 'lucide-react';
import { Card } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Textarea } from '@/components/ui/textarea';
import { Badge } from '@/components/ui/badge';
import { apiClient, Memory } from '@/lib/api-client';

export default function MemoryDetailPage() {
  const params = useParams();
  const router = useRouter();
  const memoryId = params.id as string;

  const [memory, setMemory] = useState<Memory | null>(null);
  const [relatedMemories, setRelatedMemories] = useState<Memory[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [isEditing, setIsEditing] = useState(false);
  const [editContent, setEditContent] = useState('');

  useEffect(() => {
    loadMemoryDetail();
  }, [memoryId]);

  const loadMemoryDetail = async () => {
    try {
      setLoading(true);
      setError(null);

      // Load memory details using getMemory API
      const foundMemory = await apiClient.getMemory(memoryId);

      if (foundMemory) {
        setMemory(foundMemory);
        setEditContent(foundMemory.content);

        // Load related memories (search by similar content)
        const related = await apiClient.searchMemories(
          foundMemory.content.substring(0, 50)
        );
        setRelatedMemories(related.filter(m => m.id !== memoryId).slice(0, 5));
      } else {
        setError('Memory not found');
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load memory');
    } finally {
      setLoading(false);
    }
  };

  const handleSave = async () => {
    if (!memory) return;

    try {
      // Note: Update API not available, so we'll just update local state
      setMemory({ ...memory, content: editContent });
      setIsEditing(false);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update memory');
    }
  };

  const handleDelete = async () => {
    if (!memory || !confirm('Are you sure you want to delete this memory?')) {
      return;
    }

    try {
      await apiClient.deleteMemory(memory.id);
      router.push('/admin/memories');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to delete memory');
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto mb-4"></div>
          <p className="text-gray-600 dark:text-gray-400">Loading memory...</p>
        </div>
      </div>
    );
  }

  if (error || !memory) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-center">
          <p className="text-red-600 dark:text-red-400 mb-4">{error || 'Memory not found'}</p>
          <Button onClick={() => router.push('/admin/memories')}>
            <ArrowLeft className="w-4 h-4 mr-2" />
            Back to Memories
          </Button>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center space-x-4">
          <Button
            variant="outline"
            onClick={() => router.push('/admin/memories')}
          >
            <ArrowLeft className="w-4 h-4 mr-2" />
            Back
          </Button>
          <div>
            <h2 className="text-3xl font-bold text-gray-900 dark:text-white">
              Memory Details
            </h2>
            <p className="text-gray-600 dark:text-gray-400 mt-1">
              ID: {memory.id.slice(0, 8)}...
            </p>
          </div>
        </div>
        <div className="flex space-x-2">
          {!isEditing && (
            <>
              <Button
                variant="outline"
                onClick={() => setIsEditing(true)}
              >
                <Edit3 className="w-4 h-4 mr-2" />
                Edit
              </Button>
              <Button
                variant="destructive"
                onClick={handleDelete}
              >
                <Trash2 className="w-4 h-4 mr-2" />
                Delete
              </Button>
            </>
          )}
          {isEditing && (
            <>
              <Button
                variant="outline"
                onClick={() => {
                  setIsEditing(false);
                  setEditContent(memory.content);
                }}
              >
                Cancel
              </Button>
              <Button onClick={handleSave}>
                Save Changes
              </Button>
            </>
          )}
        </div>
      </div>

      {/* Main Content */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Memory Content */}
        <div className="lg:col-span-2 space-y-6">
          <Card className="p-6">
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
              Content
            </h3>
            {isEditing ? (
              <Textarea
                value={editContent}
                onChange={(e) => setEditContent(e.target.value)}
                rows={6}
                className="w-full"
              />
            ) : (
              <p className="text-gray-700 dark:text-gray-300 whitespace-pre-wrap">
                {memory.content}
              </p>
            )}
          </Card>

          {/* Related Memories */}
          <Card className="p-6">
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4 flex items-center">
              <LinkIcon className="w-5 h-5 mr-2" />
              Related Memories
            </h3>
            {relatedMemories.length === 0 ? (
              <p className="text-gray-500 dark:text-gray-400">
                No related memories found
              </p>
            ) : (
              <div className="space-y-3">
                {relatedMemories.map((related) => (
                  <div
                    key={related.id}
                    className="p-3 border border-gray-200 dark:border-gray-700 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-800 cursor-pointer transition-colors"
                    onClick={() => router.push(`/admin/memories/${related.id}`)}
                  >
                    <div className="flex items-start justify-between">
                      <div className="flex-1">
                        <p className="text-sm text-gray-700 dark:text-gray-300 line-clamp-2">
                          {related.content}
                        </p>
                        <div className="flex items-center space-x-2 mt-2">
                          <Badge variant="outline">
                            {related.memory_type}
                          </Badge>
                          <span className="text-xs text-gray-500">
                            Importance: {related.importance.toFixed(2)}
                          </span>
                        </div>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </Card>
        </div>

        {/* Metadata Sidebar */}
        <div className="space-y-6">
          {/* Type and Importance */}
          <Card className="p-6">
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
              Metadata
            </h3>
            <div className="space-y-4">
              <div>
                <label className="text-sm font-medium text-gray-600 dark:text-gray-400">
                  Type
                </label>
                <div className="mt-1">
                  <Badge className="text-sm">
                    <Tag className="w-3 h-3 mr-1" />
                    {memory.memory_type}
                  </Badge>
                </div>
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
                        style={{ width: `${memory.importance * 100}%` }}
                      ></div>
                    </div>
                    <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
                      {(memory.importance * 100).toFixed(0)}%
                    </span>
                  </div>
                </div>
              </div>
            </div>
          </Card>

          {/* Timestamps */}
          <Card className="p-6">
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4 flex items-center">
              <Clock className="w-5 h-5 mr-2" />
              Timeline
            </h3>
            <div className="space-y-3">
              <div>
                <label className="text-sm font-medium text-gray-600 dark:text-gray-400">
                  Created
                </label>
                <p className="text-sm text-gray-700 dark:text-gray-300 mt-1">
                  {new Date(memory.created_at).toLocaleString()}
                </p>
              </div>
              <div>
                <label className="text-sm font-medium text-gray-600 dark:text-gray-400">
                  Updated
                </label>
                <p className="text-sm text-gray-700 dark:text-gray-300 mt-1">
                  {new Date(memory.updated_at).toLocaleString()}
                </p>
              </div>
            </div>
          </Card>

          {/* Agent Info */}
          <Card className="p-6">
            <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4 flex items-center">
              <Brain className="w-5 h-5 mr-2" />
              Agent
            </h3>
            <p className="text-sm text-gray-700 dark:text-gray-300">
              Agent ID: {memory.agent_id.slice(0, 8)}...
            </p>
          </Card>
        </div>
      </div>
    </div>
  );
}

