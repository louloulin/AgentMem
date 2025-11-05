/**
 * Plugins Management Page
 * 
 * Provides interface for:
 * - Viewing installed plugins
 * - Uploading new plugins
 * - Managing plugin configurations
 */

'use client';

import React, { useState, useEffect } from 'react';
import { Card } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Badge } from '@/components/ui/badge';
import { Skeleton } from '@/components/ui/skeleton';
import { Separator } from '@/components/ui/separator';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { 
  Puzzle, 
  Upload, 
  CheckCircle, 
  XCircle, 
  AlertCircle,
  Package,
  RefreshCw,
  Plus,
  FileCode,
  Settings
} from 'lucide-react';
import { apiClient, type Plugin, type PluginType, type PluginRegistrationRequest } from '@/lib/api-client';
import { useToast } from '@/hooks/use-toast';
import { cn } from '@/lib/utils';

export default function PluginsPage() {
  const [plugins, setPlugins] = useState<Plugin[]>([]);
  const [loading, setLoading] = useState(true);
  const [uploading, setUploading] = useState(false);
  const { toast } = useToast();

  // Form state for plugin registration
  const [showUploadForm, setShowUploadForm] = useState(false);
  
  // Plugin details dialog state
  const [selectedPlugin, setSelectedPlugin] = useState<Plugin | null>(null);
  const [showDetailsDialog, setShowDetailsDialog] = useState(false);
  const [formData, setFormData] = useState<PluginRegistrationRequest>({
    name: '',
    description: '',
    version: '1.0.0',
    plugin_type: 'memory_processor',
    wasm_path: '',
    config: {},
  });
  const [selectedFile, setSelectedFile] = useState<File | null>(null);

  const loadPlugins = async () => {
    try {
      setLoading(true);
      const data = await apiClient.getPlugins();
      setPlugins(data);
    } catch (error) {
      toast({
        title: "Error",
        description: error instanceof Error ? error.message : "Failed to load plugins",
        variant: "destructive",
      });
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadPlugins();
    
    // Auto-refresh every 30 seconds
    const interval = setInterval(loadPlugins, 30000);
    return () => clearInterval(interval);
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const handleFileSelect = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (file) {
      if (!file.name.endsWith('.wasm')) {
        toast({
          title: "Invalid File",
          description: "Please select a .wasm file",
          variant: "destructive",
        });
        return;
      }
      setSelectedFile(file);
      // Auto-fill name from filename
      const nameWithoutExt = file.name.replace('.wasm', '').replace(/_/g, ' ');
      setFormData(prev => ({
        ...prev,
        name: prev.name || nameWithoutExt,
      }));
    }
  };

  const handleUploadPlugin = async (e: React.FormEvent) => {
    e.preventDefault();
    
    if (!selectedFile) {
      toast({
        title: "No File Selected",
        description: "Please select a WASM file to upload",
        variant: "destructive",
      });
      return;
    }

    try {
      setUploading(true);

      // For now, we use the absolute path approach since upload endpoint may not exist
      // In production, you would upload the file first:
      // const uploadResult = await apiClient.uploadWasmFile(selectedFile);
      // const wasmPath = uploadResult.path;
      
      // Temporarily use a path based on filename
      const wasmPath = `target/wasm32-wasip1/release/${selectedFile.name}`;
      
      const registrationData: PluginRegistrationRequest = {
        ...formData,
        wasm_path: wasmPath,
      };

      await apiClient.registerPlugin(registrationData);
      
      toast({
        title: "Success",
        description: `Plugin "${formData.name}" registered successfully`,
      });

      // Reset form
      setShowUploadForm(false);
      setFormData({
        name: '',
        description: '',
        version: '1.0.0',
        plugin_type: 'memory_processor',
        wasm_path: '',
        config: {},
      });
      setSelectedFile(null);

      // Reload plugins
      await loadPlugins();
    } catch (error) {
      toast({
        title: "Error",
        description: error instanceof Error ? error.message : "Failed to register plugin",
        variant: "destructive",
      });
    } finally {
      setUploading(false);
    }
  };

  const getPluginTypeBadge = (pluginType: PluginType): { label: string; variant: 'default' | 'secondary' | 'destructive' | 'outline' } => {
    if (typeof pluginType === 'string') {
      const variants: Record<string, { label: string; variant: 'default' | 'secondary' | 'destructive' | 'outline' }> = {
        memory_processor: { label: 'Memory Processor', variant: 'default' },
        code_analyzer: { label: 'Code Analyzer', variant: 'secondary' },
        search_algorithm: { label: 'Search Algorithm', variant: 'outline' },
        data_source: { label: 'Data Source', variant: 'default' },
        multimodal: { label: 'Multimodal', variant: 'secondary' },
      };
      return variants[pluginType] || { label: pluginType, variant: 'outline' };
    } else {
      return { label: `Custom: ${pluginType.custom}`, variant: 'outline' };
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'registered':
      case 'active':
        return <CheckCircle className="w-4 h-4 text-green-500" />;
      case 'disabled':
        return <AlertCircle className="w-4 h-4 text-yellow-500" />;
      case 'error':
        return <XCircle className="w-4 h-4 text-red-500" />;
      default:
        return <AlertCircle className="w-4 h-4 text-gray-500" />;
    }
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-white flex items-center gap-3">
            <Puzzle className="w-8 h-8 text-purple-400" />
            Plugin Management
          </h1>
          <p className="text-slate-400 mt-2">
            Manage WASM plugins to extend AgentMem functionality
          </p>
        </div>
        <div className="flex gap-3">
          <Button
            variant="outline"
            onClick={loadPlugins}
            disabled={loading}
            className="bg-slate-800 border-slate-700 text-white hover:bg-slate-700"
          >
            <RefreshCw className={cn("w-4 h-4 mr-2", loading && "animate-spin")} />
            Refresh
          </Button>
          <Button
            onClick={() => setShowUploadForm(!showUploadForm)}
            className="bg-purple-600 hover:bg-purple-700 text-white"
          >
            <Plus className="w-4 h-4 mr-2" />
            Add Plugin
          </Button>
        </div>
      </div>

      {/* Stats Cards */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <Card className="bg-slate-800/50 border-slate-700 p-4">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-slate-400 text-sm">Total Plugins</p>
              <p className="text-2xl font-bold text-white mt-1">{plugins.length}</p>
            </div>
            <Package className="w-8 h-8 text-purple-400" />
          </div>
        </Card>
        <Card className="bg-slate-800/50 border-slate-700 p-4">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-slate-400 text-sm">Active</p>
              <p className="text-2xl font-bold text-green-400 mt-1">
                {plugins.filter(p => p.status === 'active' || p.status === 'registered').length}
              </p>
            </div>
            <CheckCircle className="w-8 h-8 text-green-400" />
          </div>
        </Card>
        <Card className="bg-slate-800/50 border-slate-700 p-4">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-slate-400 text-sm">Disabled</p>
              <p className="text-2xl font-bold text-yellow-400 mt-1">
                {plugins.filter(p => p.status === 'disabled').length}
              </p>
            </div>
            <AlertCircle className="w-8 h-8 text-yellow-400" />
          </div>
        </Card>
        <Card className="bg-slate-800/50 border-slate-700 p-4">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-slate-400 text-sm">Errors</p>
              <p className="text-2xl font-bold text-red-400 mt-1">
                {plugins.filter(p => p.status === 'error').length}
              </p>
            </div>
            <XCircle className="w-8 h-8 text-red-400" />
          </div>
        </Card>
      </div>

      {/* Upload Form */}
      {showUploadForm && (
        <Card className="bg-slate-800/50 border-slate-700 p-6">
          <div className="flex items-center gap-2 mb-4">
            <Upload className="w-5 h-5 text-purple-400" />
            <h2 className="text-xl font-semibold text-white">Register New Plugin</h2>
          </div>
          <form onSubmit={handleUploadPlugin} className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div>
                <Label htmlFor="name" className="text-slate-300">Plugin Name *</Label>
                <Input
                  id="name"
                  value={formData.name}
                  onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                  placeholder="My Awesome Plugin"
                  required
                  className="bg-slate-900/50 border-slate-700 text-white"
                />
              </div>
              <div>
                <Label htmlFor="version" className="text-slate-300">Version *</Label>
                <Input
                  id="version"
                  value={formData.version}
                  onChange={(e) => setFormData({ ...formData, version: e.target.value })}
                  placeholder="1.0.0"
                  required
                  className="bg-slate-900/50 border-slate-700 text-white"
                />
              </div>
            </div>

            <div>
              <Label htmlFor="description" className="text-slate-300">Description *</Label>
              <Input
                id="description"
                value={formData.description}
                onChange={(e) => setFormData({ ...formData, description: e.target.value })}
                placeholder="Brief description of what this plugin does"
                required
                className="bg-slate-900/50 border-slate-700 text-white"
              />
            </div>

            <div>
              <Label htmlFor="plugin_type" className="text-slate-300">Plugin Type *</Label>
              <select
                id="plugin_type"
                value={typeof formData.plugin_type === 'string' ? formData.plugin_type : 'custom'}
                onChange={(e) => setFormData({ ...formData, plugin_type: e.target.value as PluginType })}
                required
                className="w-full bg-slate-900/50 border border-slate-700 text-white rounded-md px-3 py-2"
              >
                <option value="memory_processor">Memory Processor</option>
                <option value="code_analyzer">Code Analyzer</option>
                <option value="search_algorithm">Search Algorithm</option>
                <option value="data_source">Data Source</option>
                <option value="multimodal">Multimodal</option>
              </select>
            </div>

            <div>
              <Label htmlFor="wasm_file" className="text-slate-300">WASM File *</Label>
              <div className="flex items-center gap-2">
                <Input
                  id="wasm_file"
                  type="file"
                  accept=".wasm"
                  onChange={handleFileSelect}
                  required
                  className="bg-slate-900/50 border-slate-700 text-white file:mr-4 file:py-2 file:px-4 file:rounded-md file:border-0 file:bg-purple-600 file:text-white hover:file:bg-purple-700"
                />
                {selectedFile && (
                  <Badge variant="outline" className="text-green-400 border-green-400">
                    <FileCode className="w-3 h-3 mr-1" />
                    {selectedFile.name}
                  </Badge>
                )}
              </div>
              <p className="text-xs text-slate-500 mt-1">
                Select a compiled .wasm plugin file
              </p>
            </div>

            <Separator className="bg-slate-700" />

            <div className="flex justify-end gap-3">
              <Button
                type="button"
                variant="outline"
                onClick={() => {
                  setShowUploadForm(false);
                  setSelectedFile(null);
                }}
                className="bg-slate-800 border-slate-700 text-white hover:bg-slate-700"
              >
                Cancel
              </Button>
              <Button
                type="submit"
                disabled={uploading}
                className="bg-purple-600 hover:bg-purple-700 text-white"
              >
                {uploading ? (
                  <>
                    <RefreshCw className="w-4 h-4 mr-2 animate-spin" />
                    Registering...
                  </>
                ) : (
                  <>
                    <Upload className="w-4 h-4 mr-2" />
                    Register Plugin
                  </>
                )}
              </Button>
            </div>
          </form>
        </Card>
      )}

      {/* Plugins List */}
      <Card className="bg-slate-800/50 border-slate-700">
        <div className="p-6">
          <h2 className="text-xl font-semibold text-white mb-4 flex items-center gap-2">
            <Package className="w-5 h-5 text-purple-400" />
            Installed Plugins
          </h2>
          
          {loading ? (
            <div className="space-y-3">
              {[1, 2, 3].map((i) => (
                <Skeleton key={i} className="h-24 bg-slate-900/50" />
              ))}
            </div>
          ) : plugins.length === 0 ? (
            <div className="text-center py-12">
              <Puzzle className="w-16 h-16 text-slate-600 mx-auto mb-4" />
              <p className="text-slate-400 text-lg">No plugins installed</p>
              <p className="text-slate-500 text-sm mt-2">
                Click &quot;Add Plugin&quot; to register your first WASM plugin
              </p>
            </div>
          ) : (
            <div className="space-y-3">
              {plugins.map((plugin) => {
                const typeBadge = getPluginTypeBadge(plugin.plugin_type);
                return (
                  <Card key={plugin.id} className="bg-slate-900/50 border-slate-700 p-4 hover:border-purple-500/50 transition-colors">
                    <div className="flex items-start justify-between">
                      <div className="flex-1">
                        <div className="flex items-center gap-3 mb-2">
                          {getStatusIcon(plugin.status)}
                          <h3 className="text-lg font-semibold text-white">{plugin.name}</h3>
                          <Badge variant={typeBadge.variant} className="text-xs">
                            {typeBadge.label}
                          </Badge>
                          <Badge variant="outline" className="text-xs text-slate-400 border-slate-600">
                            v{plugin.version}
                          </Badge>
                        </div>
                        <p className="text-slate-400 text-sm mb-2">{plugin.description}</p>
                        <div className="flex items-center gap-4 text-xs text-slate-500">
                          <span className="flex items-center gap-1">
                            <FileCode className="w-3 h-3" />
                            {plugin.wasm_path?.split('/').pop() || 'N/A'}
                          </span>
                          <span>ID: {plugin.id}</span>
                          <span>Status: <span className={cn(
                            "font-semibold",
                            (plugin.status === 'active' || plugin.status === 'registered') && "text-green-400",
                            plugin.status === 'disabled' && "text-yellow-400",
                            plugin.status === 'error' && "text-red-400"
                          )}>{plugin.status}</span></span>
                        </div>
                      </div>
                      <div className="flex gap-2">
                        <Button
                          size="sm"
                          variant="outline"
                          className="bg-slate-800 border-slate-700 text-white hover:bg-slate-700"
                          onClick={() => {
                            setSelectedPlugin(plugin);
                            setShowDetailsDialog(true);
                          }}
                        >
                          View Details
                        </Button>
                      </div>
                    </div>
                  </Card>
                );
              })}
            </div>
          )}
        </div>
      </Card>

      {/* Plugin Details Dialog */}
      <Dialog open={showDetailsDialog} onOpenChange={setShowDetailsDialog}>
        <DialogContent className="bg-slate-800 border-slate-700 text-white max-w-2xl max-h-[80vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle className="text-2xl font-bold flex items-center gap-2">
              <Puzzle className="w-6 h-6 text-purple-400" />
              Plugin Details
            </DialogTitle>
            <DialogDescription className="text-slate-400">
              Complete information about the selected plugin
            </DialogDescription>
          </DialogHeader>

          {selectedPlugin && (
            <div className="space-y-6 mt-4">
              {/* Basic Information */}
              <div>
                <h3 className="text-lg font-semibold text-purple-400 mb-3 flex items-center gap-2">
                  <Package className="w-5 h-5" />
                  Basic Information
                </h3>
                <div className="space-y-3 bg-slate-900/50 p-4 rounded-lg">
                  <div className="grid grid-cols-3 gap-2">
                    <span className="text-slate-400">Name:</span>
                    <span className="col-span-2 font-semibold text-white">{selectedPlugin.name}</span>
                  </div>
                  <Separator className="bg-slate-700" />
                  <div className="grid grid-cols-3 gap-2">
                    <span className="text-slate-400">ID:</span>
                    <span className="col-span-2 font-mono text-sm text-slate-300">{selectedPlugin.id}</span>
                  </div>
                  <Separator className="bg-slate-700" />
                  <div className="grid grid-cols-3 gap-2">
                    <span className="text-slate-400">Version:</span>
                    <span className="col-span-2">
                      <Badge variant="outline" className="text-slate-300 border-slate-600">
                        v{selectedPlugin.version}
                      </Badge>
                    </span>
                  </div>
                  <Separator className="bg-slate-700" />
                  <div className="grid grid-cols-3 gap-2">
                    <span className="text-slate-400">Description:</span>
                    <span className="col-span-2 text-slate-300">{selectedPlugin.description}</span>
                  </div>
                </div>
              </div>

              {/* Status and Type */}
              <div>
                <h3 className="text-lg font-semibold text-purple-400 mb-3 flex items-center gap-2">
                  <CheckCircle className="w-5 h-5" />
                  Status & Type
                </h3>
                <div className="space-y-3 bg-slate-900/50 p-4 rounded-lg">
                  <div className="grid grid-cols-3 gap-2">
                    <span className="text-slate-400">Status:</span>
                    <span className="col-span-2 flex items-center gap-2">
                      {getStatusIcon(selectedPlugin.status)}
                      <Badge 
                        variant={
                          (selectedPlugin.status === 'active' || selectedPlugin.status === 'registered') ? 'default' :
                          selectedPlugin.status === 'disabled' ? 'secondary' : 'destructive'
                        }
                        className="text-xs"
                      >
                        {selectedPlugin.status}
                      </Badge>
                    </span>
                  </div>
                  <Separator className="bg-slate-700" />
                  <div className="grid grid-cols-3 gap-2">
                    <span className="text-slate-400">Type:</span>
                    <span className="col-span-2">
                      <Badge 
                        variant={getPluginTypeBadge(selectedPlugin.plugin_type).variant}
                        className="text-xs"
                      >
                        {getPluginTypeBadge(selectedPlugin.plugin_type).label}
                      </Badge>
                    </span>
                  </div>
                  <Separator className="bg-slate-700" />
                  <div className="grid grid-cols-3 gap-2">
                    <span className="text-slate-400">WASM Path:</span>
                    <span className="col-span-2 font-mono text-xs text-slate-300 break-all">
                      {selectedPlugin.wasm_path || 'N/A'}
                    </span>
                  </div>
                </div>
              </div>

              {/* Configuration */}
              {selectedPlugin.config && Object.keys(selectedPlugin.config).length > 0 && (
                <div>
                  <h3 className="text-lg font-semibold text-purple-400 mb-3 flex items-center gap-2">
                    <Settings className="w-5 h-5" />
                    Configuration
                  </h3>
                  <div className="bg-slate-900/50 p-4 rounded-lg">
                    <pre className="text-xs text-slate-300 overflow-x-auto">
                      {JSON.stringify(selectedPlugin.config, null, 2)}
                    </pre>
                  </div>
                </div>
              )}

              {/* Timestamps */}
              <div>
                <h3 className="text-lg font-semibold text-purple-400 mb-3 flex items-center gap-2">
                  <AlertCircle className="w-5 h-5" />
                  Timestamps
                </h3>
                <div className="space-y-3 bg-slate-900/50 p-4 rounded-lg">
                  <div className="grid grid-cols-3 gap-2">
                    <span className="text-slate-400">Created:</span>
                    <span className="col-span-2 text-slate-300 text-sm">
                      {new Date(selectedPlugin.created_at).toLocaleString()}
                    </span>
                  </div>
                  <Separator className="bg-slate-700" />
                  <div className="grid grid-cols-3 gap-2">
                    <span className="text-slate-400">Updated:</span>
                    <span className="col-span-2 text-slate-300 text-sm">
                      {new Date(selectedPlugin.updated_at).toLocaleString()}
                    </span>
                  </div>
                </div>
              </div>

              {/* Actions */}
              <div className="flex justify-end gap-3 pt-4 border-t border-slate-700">
                <Button
                  variant="outline"
                  onClick={() => setShowDetailsDialog(false)}
                  className="bg-slate-700 border-slate-600 text-white hover:bg-slate-600"
                >
                  Close
                </Button>
              </div>
            </div>
          )}
        </DialogContent>
      </Dialog>
    </div>
  );
}

