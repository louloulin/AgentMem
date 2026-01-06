"use client";

import React from "react";
import Link from "next/link";
import { Button } from "@/components/ui/button";
import { ArrowLeft, Home } from "lucide-react";

function NotFoundContent() {
  return (
    <div className="flex flex-col items-center justify-center min-h-[70vh] text-center px-4">
      <h1 className="text-5xl md:text-7xl font-bold bg-gradient-to-r from-purple-400 to-blue-500 text-transparent bg-clip-text mb-6">
        404
      </h1>
      <h2 className="text-2xl md:text-3xl font-semibold text-white mb-4">
        页面未找到
      </h2>
      <p className="text-slate-400 max-w-md mb-8">
        您访问的页面不存在或已被移除。请检查URL是否正确，或返回首页。
      </p>
      <div className="flex flex-col sm:flex-row gap-4">
        <Button
          variant="outline"
          className="border-slate-700 hover:bg-slate-800 text-white"
          onClick={() => window.history.back()}
        >
          <ArrowLeft className="mr-2 h-4 w-4" />
          返回上一页
        </Button>
        <Button className="bg-gradient-to-r from-purple-500 to-blue-500 hover:from-purple-600 hover:to-blue-600">
          <Link href="/" className="flex items-center">
            <Home className="mr-2 h-4 w-4" />
            返回首页
          </Link>
        </Button>
      </div>
    </div>
  );
}

/**
 * 404页面
 * 使用动态导入解决预渲染问题
 */
export default function NotFound() {
  return (
    <div className="bg-slate-900 min-h-screen">
      <div className="container mx-auto py-16">
        {/* 使用动态导入而非Suspense，解决预渲染问题 */}
        <NotFoundContent />
      </div>
    </div>
  );
}