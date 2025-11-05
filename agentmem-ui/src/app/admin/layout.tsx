/**
 * Admin Layout Component
 * 
 * Provides the main layout structure for the admin dashboard,
 * including sidebar navigation and header.
 * 
 * Enhanced with Supabase-style modern UI:
 * - Active state highlighting
 * - Smooth transitions
 * - Better visual hierarchy
 */

'use client';

import React from 'react';
import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { Bot, Brain, Users, Settings, Home, MessageSquare, Network, Puzzle } from 'lucide-react';
import { cn } from '@/lib/utils';
import { Toaster } from '@/components/ui/toaster';
import { ThemeToggle } from '@/components/ui/theme-toggle';

interface AdminLayoutProps {
  children: React.ReactNode;
}

export default function AdminLayout({ children }: AdminLayoutProps) {
  return (
    <div className="flex h-screen bg-gradient-to-br from-slate-900 via-purple-900 to-slate-900">
      {/* Sidebar */}
      <aside className="w-64 bg-slate-900/50 backdrop-blur-sm border-r border-slate-800">
        <div className="flex flex-col h-full">
          {/* Logo */}
          <div className="flex items-center justify-center h-16 border-b border-slate-800">
            <Link href="/admin" className="flex items-center space-x-2">
              <Bot className="w-8 h-8 text-purple-400 animate-pulse-glow" />
              <span className="text-xl font-bold text-white">
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
              <NavLink href="/admin/plugins" icon={<Puzzle className="w-5 h-5" />}>
                Plugins
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
          <div className="p-4 border-t border-slate-800">
            <p className="text-xs text-slate-400 text-center">
              AgentMem v2.1
            </p>
          </div>
        </div>
      </aside>

      {/* Main Content */}
      <div className="flex-1 flex flex-col overflow-hidden">
        {/* Header */}
        <header className="h-16 bg-slate-900/50 backdrop-blur-sm border-b border-slate-800">
          <div className="flex items-center justify-between h-full px-6">
            <h1 className="text-2xl font-semibold text-white">
              Admin Dashboard
            </h1>
            <div className="flex items-center space-x-4">
              <ThemeToggle />
              <Link href="/admin/settings">
                <button className="p-2 text-slate-300 hover:bg-slate-800/50 rounded-lg transition-colors">
                  <Settings className="w-5 h-5" />
                </button>
              </Link>
            </div>
          </div>
        </header>

        {/* Page Content */}
        <main className="flex-1 overflow-y-auto p-6">
          {children}
        </main>
      </div>

      {/* Toast Notifications */}
      <Toaster />
    </div>
  );
}

/**
 * Navigation Link Component
 * Enhanced with active state detection and Supabase-style highlighting
 */
interface NavLinkProps {
  href: string;
  icon: React.ReactNode;
  children: React.ReactNode;
}

function NavLink({ href, icon, children }: NavLinkProps) {
  const pathname = usePathname();
  const isActive = pathname === href || (href !== '/admin' && pathname.startsWith(href));
  
  return (
    <Link
      href={href}
      className={cn(
        "nav-item-supabase flex items-center gap-3",
        isActive && "active"
      )}
    >
      {icon}
      <span>{children}</span>
    </Link>
  );
}

