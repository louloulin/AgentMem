/**
 * Users Management Page
 * 
 * Allows viewing and managing users.
 */

'use client';

import React, { useState, useEffect } from 'react';
import { Users as UsersIcon, Mail, Calendar } from 'lucide-react';
import { Card } from '@/components/ui/card';
import { apiClient, User } from '@/lib/api-client';

export default function UsersPage() {
  const [users, setUsers] = useState<User[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadUsers();
  }, []);

  const loadUsers = async () => {
    try {
      setLoading(true);
      setError(null);
      const data = await apiClient.getUsers();
      setUsers(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load users');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h2 className="text-3xl font-bold text-gray-900 dark:text-white">
          Users
        </h2>
        <p className="text-gray-600 dark:text-gray-400 mt-1">
          Manage system users
        </p>
      </div>

      {/* Error Message */}
      {error && (
        <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
          <p className="text-red-800 dark:text-red-200">{error}</p>
        </div>
      )}

      {/* Loading State */}
      {loading && (
        <div className="flex items-center justify-center py-12">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600" />
        </div>
      )}

      {/* Users List */}
      {!loading && users.length === 0 && (
        <Card className="p-12 text-center">
          <UsersIcon className="w-12 h-12 text-gray-400 mx-auto mb-4" />
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-2">
            No users found
          </h3>
          <p className="text-gray-600 dark:text-gray-400">
            No users are registered in the system
          </p>
        </Card>
      )}

      {!loading && users.length > 0 && (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {users.map((user) => (
            <UserCard key={user.id} user={user} />
          ))}
        </div>
      )}
    </div>
  );
}

/**
 * User Card Component
 */
interface UserCardProps {
  user: User;
}

function UserCard({ user }: UserCardProps) {
  return (
    <Card className="p-6">
      <div className="flex items-center space-x-3 mb-4">
        <div className="w-12 h-12 bg-blue-100 dark:bg-blue-900 rounded-full flex items-center justify-center">
          <span className="text-xl font-semibold text-blue-600 dark:text-blue-300">
            {user.name?.[0]?.toUpperCase() || user.email[0].toUpperCase()}
          </span>
        </div>
        <div>
          <h3 className="font-semibold text-gray-900 dark:text-white">
            {user.name || 'Unnamed User'}
          </h3>
          <p className="text-sm text-gray-600 dark:text-gray-400">
            ID: {user.id.slice(0, 8)}...
          </p>
        </div>
      </div>

      <div className="space-y-2">
        <div className="flex items-center space-x-2 text-sm text-gray-600 dark:text-gray-400">
          <Mail className="w-4 h-4" />
          <span>{user.email}</span>
        </div>
        <div className="flex items-center space-x-2 text-sm text-gray-600 dark:text-gray-400">
          <Calendar className="w-4 h-4" />
          <span>Joined {new Date(user.created_at).toLocaleDateString()}</span>
        </div>
      </div>
    </Card>
  );
}

