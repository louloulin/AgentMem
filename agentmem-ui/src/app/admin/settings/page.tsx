/**
 * Settings Page
 * 
 * System settings and configuration.
 */

'use client';

import React, { useState } from 'react';
import { Settings as SettingsIcon, Save, Database, Key, Bell } from 'lucide-react';
import { Card } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';

export default function SettingsPage() {
  const [apiUrl, setApiUrl] = useState('http://localhost:8080');
  const [apiKey, setApiKey] = useState('');
  const [saved, setSaved] = useState(false);

  const handleSave = () => {
    // Save settings to localStorage
    localStorage.setItem('agentmem_api_url', apiUrl);
    localStorage.setItem('agentmem_api_key', apiKey);
    setSaved(true);
    setTimeout(() => setSaved(false), 3000);
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h2 className="text-3xl font-bold text-gray-900 dark:text-white">
          Settings
        </h2>
        <p className="text-gray-600 dark:text-gray-400 mt-1">
          Configure system settings
        </p>
      </div>

      {/* API Settings */}
      <Card className="p-6">
        <div className="flex items-center space-x-3 mb-6">
          <Database className="w-6 h-6 text-blue-600" />
          <h3 className="text-xl font-semibold text-gray-900 dark:text-white">
            API Configuration
          </h3>
        </div>

        <div className="space-y-4">
          <div>
            <Label htmlFor="api-url">API URL</Label>
            <Input
              id="api-url"
              value={apiUrl}
              onChange={(e) => setApiUrl(e.target.value)}
              placeholder="http://localhost:8080"
            />
            <p className="text-sm text-gray-600 dark:text-gray-400 mt-1">
              The base URL of your AgentMem API server
            </p>
          </div>

          <div>
            <Label htmlFor="api-key">API Key</Label>
            <Input
              id="api-key"
              type="password"
              value={apiKey}
              onChange={(e) => setApiKey(e.target.value)}
              placeholder="Enter your API key"
            />
            <p className="text-sm text-gray-600 dark:text-gray-400 mt-1">
              Your API authentication key
            </p>
          </div>

          <div className="flex items-center space-x-3">
            <Button onClick={handleSave}>
              <Save className="w-4 h-4 mr-2" />
              Save Settings
            </Button>
            {saved && (
              <span className="text-sm text-green-600 dark:text-green-400">
                Settings saved successfully!
              </span>
            )}
          </div>
        </div>
      </Card>

      {/* System Info */}
      <Card className="p-6">
        <div className="flex items-center space-x-3 mb-6">
          <SettingsIcon className="w-6 h-6 text-blue-600" />
          <h3 className="text-xl font-semibold text-gray-900 dark:text-white">
            System Information
          </h3>
        </div>

        <div className="space-y-3">
          <InfoRow label="Version" value="2.1.0" />
          <InfoRow label="Environment" value="Development" />
          <InfoRow label="Database" value="PostgreSQL" />
          <InfoRow label="Cache" value="Redis" />
        </div>
      </Card>
    </div>
  );
}

/**
 * Info Row Component
 */
interface InfoRowProps {
  label: string;
  value: string;
}

function InfoRow({ label, value }: InfoRowProps) {
  return (
    <div className="flex items-center justify-between py-2 border-b border-gray-200 dark:border-gray-700 last:border-0">
      <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
        {label}
      </span>
      <span className="text-sm text-gray-600 dark:text-gray-400">{value}</span>
    </div>
  );
}

