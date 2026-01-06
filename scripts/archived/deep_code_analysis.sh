#!/bin/bash

echo "🔍 深度代码质量分析..."
echo ""

# 1. 查找TODO和FIXME
echo "━━━ 1. TODO/FIXME统计 ━━━"
echo "TODO数量:"
grep -r "TODO" crates --include="*.rs" | wc -l
echo "FIXME数量:"
grep -r "FIXME" crates --include="*.rs" | wc -l
echo "HACK数量:"
grep -r "HACK" crates --include="*.rs" | wc -l
echo ""

# 2. 查找unwrap和panic
echo "━━━ 2. 不安全代码模式 ━━━"
echo "unwrap()调用:"
grep -r "\.unwrap()" crates --include="*.rs" | wc -l
echo "expect()调用:"
grep -r "\.expect(" crates --include="*.rs" | wc -l
echo "panic!调用:"
grep -r "panic!" crates --include="*.rs" | wc -l
echo ""

# 3. 查找unsafe代码
echo "━━━ 3. Unsafe代码块 ━━━"
grep -r "unsafe" crates --include="*.rs" | wc -l
echo ""

# 4. 错误处理模式
echo "━━━ 4. 错误处理 ━━━"
echo "Result类型使用:"
grep -r "Result<" crates --include="*.rs" | wc -l
echo "Option类型使用:"
grep -r "Option<" crates --include="*.rs" | wc -l
echo ""

# 5. 数据库查询分析
echo "━━━ 5. 数据库操作 ━━━"
echo "SQL查询数量:"
grep -r "execute\|query" crates/agent-mem-core/src/storage --include="*.rs" | wc -l
echo ""

# 6. API路由统计
echo "━━━ 6. API路由详细统计 ━━━"
for route_file in crates/agent-mem-server/src/routes/*.rs; do
  if [ -f "$route_file" ]; then
    filename=$(basename "$route_file" .rs)
    count=$(grep "pub async fn" "$route_file" | wc -l)
    echo "  - $filename: $count endpoints"
  fi
done
echo ""

# 7. Mock数据检测
echo "━━━ 7. Mock数据检测 ━━━"
echo "Mock相关代码:"
grep -ri "mock\|fake\|dummy" crates --include="*.rs" | grep -v "test" | wc -l
echo ""

