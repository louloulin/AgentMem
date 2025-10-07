/**
 * Admin Layout Component
 * 
 * Provides the main layout structure for the admin dashboard,
 * including sidebar navigation and header.
 */

import React from 'react';
import Link from 'next/link';
import { Bot, Brain, Users, Settings, Home, MessageSquare, Network } from 'lucide-react';

interface AdminLayoutProps {
  children: React.ReactNode;
}

export default function AdminLayout({ children }: AdminLayoutProps) {
  return (
    <div className="flex h-screen bg-gray-50 dark:bg-gray-900">
      {/* Sidebar */}
      <aside className="w-64 bg-white dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700">
        <div className="flex flex-col h-full">
          {/* Logo */}
          <div className="flex items-center justify-center h-16 border-b border-gray-200 dark:border-gray-700">
            <Link href="/admin" className="flex items-center space-x-2">
              <Bot className="w-8 h-8 text-blue-600" />
              <span className="text-xl font-bold text-gray-900 dark:text-white">
                AgentMem
              </span>
            </Link>
          </div>

          {/* Navigation */}
          <nav className="flex-1 overflow-y-auto py-4">
            <div className="px-3 space-y-1">
              <NavLink href="/admin" icon={<Home className="w-5 h-5" />}>
                Dashboard
              </NavLink>
              <NavLink href="/admin/agents" icon={<Bot className="w-5 h-5" />}>
                Agents
              </NavLink>
              <NavLink href="/admin/chat" icon={<MessageSquare className="w-5 h-5" />}>
                Chat
              </NavLink>
              <NavLink href="/admin/memories" icon={<Brain className="w-5 h-5" />}>
                Memories
              </NavLink>
              <NavLink href="/admin/graph" icon={<Network className="w-5 h-5" />}>
                Knowledge Graph
              </NavLink>
              <NavLink href="/admin/users" icon={<Users className="w-5 h-5" />}>
                Users
              </NavLink>
              <NavLink href="/admin/settings" icon={<Settings className="w-5 h-5" />}>
                Settings
              </NavLink>
            </div>
          </nav>

          {/* Footer */}
          <div className="p-4 border-t border-gray-200 dark:border-gray-700">
            <p className="text-xs text-gray-500 dark:text-gray-400 text-center">
              AgentMem v2.1
            </p>
          </div>
        </div>
      </aside>

      {/* Main Content */}
      <div className="flex-1 flex flex-col overflow-hidden">
        {/* Header */}
        <header className="h-16 bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700">
          <div className="flex items-center justify-between h-full px-6">
            <h1 className="text-2xl font-semibold text-gray-900 dark:text-white">
              Admin Dashboard
            </h1>
            <div className="flex items-center space-x-4">
              <button className="p-2 text-gray-600 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg">
                <Settings className="w-5 h-5" />
              </button>
            </div>
          </div>
        </header>

        {/* Page Content */}
        <main className="flex-1 overflow-y-auto p-6">
          {children}
        </main>
      </div>
    </div>
  );
}

/**
 * Navigation Link Component
 */
interface NavLinkProps {
  href: string;
  icon: React.ReactNode;
  children: React.ReactNode;
}

function NavLink({ href, icon, children }: NavLinkProps) {
  return (
    <Link
      href={href}
      className="flex items-center space-x-3 px-3 py-2 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
    >
      {icon}
      <span>{children}</span>
    </Link>
  );
}

