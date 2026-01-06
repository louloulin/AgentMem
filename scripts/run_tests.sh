#!/bin/bash
# Comprehensive test runner for AgentMem

set -e

echo "🧪 Running AgentMem Test Suite"
echo "================================"
echo ""

# Phase 1: Quick check
echo "📦 Phase 1: Quick Compilation Check"
cargo check --workspace 2>&1 | grep -E "(error|warning|Checking|Finished)" | tail -20
echo ""

# Phase 2: Unit tests (core crates only)
echo "🔬 Phase 2: Core Crate Unit Tests"
echo "Testing agent-mem-traits..."
cargo test -p agent-mem-traits --lib 2>&1 | tail -5

echo "Testing agent-mem-utils..."
cargo test -p agent-mem-utils --lib 2>&1 | tail -5

echo "Testing agent-mem-config..."
cargo test -p agent-mem-config --lib 2>&1 | tail -5

echo ""

# Phase 3: Integration tests
echo "🔗 Phase 3: Integration Tests"
echo "Testing agent-mem-core..."
cargo test -p agent-mem-core --lib 2>&1 | tail -10

echo ""

# Summary
echo "✅ Test Suite Complete"
echo ""
echo "💡 Next steps:"
echo "   - Review any failing tests"
echo "   - Run full suite: cargo test --workspace"
echo "   - Run benchmarks: cargo bench"
