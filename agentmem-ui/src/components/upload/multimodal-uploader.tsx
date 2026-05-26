/**
 * Multimodal Uploader Component
 * 
 * Handles image upload with drag-and-drop and preview.
 * Supports: JPEG, PNG, GIF, WebP, SVG
 */

'use client';

import React, { useState, useRef, useCallback } from 'react';
import { 
  Upload, 
  Image as ImageIcon, 
  X, 
  Search, 
  Loader2,
  CheckCircle,
  AlertCircle,
  FileImage,
} from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';

const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';
const MAX_FILE_SIZE = 100 * 1024 * 1024; // 100MB
const ACCEPTED_TYPES = ['image/jpeg', 'image/png', 'image/gif', 'image/webp', 'image/svg+xml'];

interface UploadedFile {
  id: string;
  file: File;
  preview: string;
  status: 'pending' | 'uploading' | 'success' | 'error';
  progress: number;
  error?: string;
  result?: {
    id: string;
    similarity?: number;
  };
}

interface MultimodalUploaderProps {
  /** Callback when upload completes */
  onUploadComplete?: (result: { id: string; url: string }) => void;
  /** Show similarity search after upload */
  enableSearch?: boolean;
  /** Max files allowed */
  maxFiles?: number;
  className?: string;
}

export function MultimodalUploader({
  onUploadComplete,
  enableSearch = true,
  maxFiles = 5,
  className = '',
}: MultimodalUploaderProps) {
  const [files, setFiles] = useState<UploadedFile[]>([]);
  const [isDragging, setIsDragging] = useState(false);
  const [searchResults, setSearchResults] = useState<any[]>([]);
  const [isSearching, setIsSearching] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  // Handle file selection
  const handleFileSelect = useCallback((selectedFiles: FileList | null) => {
    if (!selectedFiles) return;

    const newFiles: UploadedFile[] = [];
    
    for (const file of Array.from(selectedFiles)) {
      if (files.length + newFiles.length >= maxFiles) break;
      
      if (!ACCEPTED_TYPES.includes(file.type)) {
        console.warn(`Skipping unsupported file type: ${file.type}`);
        continue;
      }
      
      if (file.size > MAX_FILE_SIZE) {
        console.warn(`File too large: ${file.name}`);
        continue;
      }

      newFiles.push({
        id: `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
        file,
        preview: URL.createObjectURL(file),
        status: 'pending',
        progress: 0,
      });
    }

    setFiles((prev) => [...prev, ...newFiles]);
  }, [files.length, maxFiles]);

  // Drag handlers
  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(true);
  }, []);

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);
  }, []);

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);
    handleFileSelect(e.dataTransfer.files);
  }, [handleFileSelect]);

  // Remove file
  const removeFile = useCallback((id: string) => {
    setFiles((prev) => {
      const file = prev.find((f) => f.id === id);
      if (file) {
        URL.revokeObjectURL(file.preview);
      }
      return prev.filter((f) => f.id !== id);
    });
    setSearchResults([]);
  }, []);

  // Upload file
  const uploadFile = useCallback(async (uploadedFile: UploadedFile) => {
    setFiles((prev) =>
      prev.map((f) =>
        f.id === uploadedFile.id ? { ...f, status: 'uploading', progress: 0 } : f
      )
    );

    try {
      const token = typeof window !== 'undefined' ? localStorage.getItem('auth_token') : null;
      const formData = new FormData();
      formData.append('file', uploadedFile.file);
      formData.append('mime_type', uploadedFile.file.type);

      const response = await fetch(`${API_BASE_URL}/api/v1/multimodal/upload`, {
        method: 'POST',
        headers: {
          ...(token ? { Authorization: `Bearer ${token}` } : {}),
        },
        body: formData,
      });

      if (!response.ok) {
        throw new Error(`Upload failed: ${response.statusText}`);
      }

      const data = await response.json();

      setFiles((prev) =>
        prev.map((f) =>
          f.id === uploadedFile.id
            ? { ...f, status: 'success', progress: 100, result: { id: data.id } }
            : f
        )
      );

      onUploadComplete?.({ id: data.id, url: uploadedFile.preview });
    } catch (error) {
      setFiles((prev) =>
        prev.map((f) =>
          f.id === uploadedFile.id
            ? { ...f, status: 'error', error: (error as Error).message }
            : f
        )
      );
    }
  }, [onUploadComplete]);

  // Search similar images
  const searchSimilar = useCallback(async (file: UploadedFile) => {
    setIsSearching(true);
    setSearchResults([]);

    try {
      const token = typeof window !== 'undefined' ? localStorage.getItem('auth_token') : null;

      const response = await fetch(`${API_BASE_URL}/api/v1/multimodal/search`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          ...(token ? { Authorization: `Bearer ${token}` } : {}),
        },
        body: JSON.stringify({
          image_id: file.result?.id || 'temp',
          limit: 10,
        }),
      });

      if (response.ok) {
        const data = await response.json();
        setSearchResults(data.results || []);
      }
    } catch (error) {
      console.error('Search failed:', error);
    } finally {
      setIsSearching(false);
    }
  }, []);

  return (
    <Card className={className}>
      <CardHeader className="pb-2">
        <CardTitle className="text-lg flex items-center gap-2">
          <FileImage className="w-5 h-5" />
          Image Upload
        </CardTitle>
      </CardHeader>

      <CardContent className="space-y-4">
        {/* Drop Zone */}
        <div
          onDragOver={handleDragOver}
          onDragLeave={handleDragLeave}
          onDrop={handleDrop}
          onClick={() => fileInputRef.current?.click()}
          className={`
            border-2 border-dashed rounded-lg p-8 text-center cursor-pointer
            transition-colors
            ${
              isDragging
                ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20'
                : 'border-gray-300 dark:border-gray-700 hover:border-gray-400 dark:hover:border-gray-600'
            }
          `}
        >
          <input
            ref={fileInputRef}
            type="file"
            accept={ACCEPTED_TYPES.join(',')}
            multiple
            onChange={(e) => handleFileSelect(e.target.files)}
            className="hidden"
          />
          
          <Upload className={`w-10 h-10 mx-auto mb-3 ${isDragging ? 'text-blue-500' : 'text-gray-400'}`} />
          
          <p className="text-sm text-gray-600 dark:text-gray-400">
            Drag and drop images here, or click to select
          </p>
          <p className="text-xs text-gray-500 mt-1">
            Supports: JPEG, PNG, GIF, WebP, SVG (max 100MB)
          </p>
        </div>

        {/* File Previews */}
        {files.length > 0 && (
          <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
            {files.map((file) => (
              <div
                key={file.id}
                className="relative group border border-gray-200 dark:border-gray-700 rounded-lg overflow-hidden"
              >
                {/* Preview Image */}
                <div className="aspect-square bg-gray-100 dark:bg-gray-800">
                  <img
                    src={file.preview}
                    alt={file.file.name}
                    className="w-full h-full object-cover"
                  />
                </div>

                {/* Status Overlay */}
                {file.status === 'uploading' && (
                  <div className="absolute inset-0 bg-black/50 flex items-center justify-center">
                    <Loader2 className="w-8 h-8 text-white animate-spin" />
                  </div>
                )}
                
                {file.status === 'success' && (
                  <div className="absolute inset-0 bg-green-500/30 flex items-center justify-center">
                    <CheckCircle className="w-8 h-8 text-green-500" />
                  </div>
                )}
                
                {file.status === 'error' && (
                  <div className="absolute inset-0 bg-red-500/30 flex flex-col items-center justify-center p-2">
                    <AlertCircle className="w-8 h-8 text-red-500 mb-1" />
                    <span className="text-xs text-red-500 text-center">{file.error}</span>
                  </div>
                )}

                {/* Action Buttons */}
                <div className="absolute top-1 right-1 flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                  {file.status === 'success' && enableSearch && (
                    <button
                      onClick={() => searchSimilar(file)}
                      className="p-1 bg-blue-500 text-white rounded-full hover:bg-blue-600"
                      title="Search similar"
                    >
                      <Search className="w-3 h-3" />
                    </button>
                  )}
                  <button
                    onClick={() => removeFile(file.id)}
                    className="p-1 bg-gray-800/70 text-white rounded-full hover:bg-red-500"
                    title="Remove"
                  >
                    <X className="w-3 h-3" />
                  </button>
                </div>

                {/* File Name */}
                <div className="p-2 bg-white dark:bg-gray-900">
                  <p className="text-xs truncate text-gray-700 dark:text-gray-300">
                    {file.file.name}
                  </p>
                  <p className="text-xs text-gray-500">
                    {(file.file.size / 1024).toFixed(1)} KB
                  </p>
                </div>
              </div>
            ))}
          </div>
        )}

        {/* Upload Button */}
        {files.some((f) => f.status === 'pending') && (
          <div className="flex gap-2">
            <Button
              onClick={() => {
                files.filter((f) => f.status === 'pending').forEach(uploadFile);
              }}
              className="flex-1"
            >
              <Upload className="w-4 h-4 mr-2" />
              Upload {files.filter((f) => f.status === 'pending').length} Files
            </Button>
          </div>
        )}

        {/* Search Results */}
        {searchResults.length > 0 && (
          <div className="border border-gray-200 dark:border-gray-700 rounded-lg p-3">
            <h4 className="text-sm font-medium mb-2 flex items-center gap-2">
              <Search className="w-4 h-4" />
              Similar Images
            </h4>
            <div className="grid grid-cols-3 md:grid-cols-5 gap-2">
              {searchResults.map((result, idx) => (
                <div key={idx} className="relative group">
                  <div className="aspect-square bg-gray-100 dark:bg-gray-800 rounded overflow-hidden">
                    {result.preview ? (
                      <img src={result.preview} alt="" className="w-full h-full object-cover" />
                    ) : (
                      <div className="w-full h-full flex items-center justify-center">
                        <ImageIcon className="w-6 h-6 text-gray-400" />
                      </div>
                    )}
                  </div>
                  {result.similarity && (
                    <div className="absolute bottom-0 inset-x-0 bg-black/60 text-white text-xs text-center py-0.5">
                      {(result.similarity * 100).toFixed(0)}%
                    </div>
                  )}
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Loading State */}
        {isSearching && (
          <div className="flex items-center justify-center py-4">
            <Loader2 className="w-5 h-5 animate-spin mr-2" />
            <span className="text-sm text-gray-500">Searching...</span>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
