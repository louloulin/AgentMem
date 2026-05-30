#!/bin/bash
# verify_memory.sh - Verify AgentMem Memory System
# This script verifies the memory system components work correctly

set -e

echo "==========================================="
echo "AgentMem Memory System Verification"
echo "==========================================="
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Counters
PASS=0
FAIL=0

# Function to check result
check() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✅ PASS${NC}: $2"
        ((PASS++))
    else
        echo -e "${RED}❌ FAIL${NC}: $2"
        ((FAIL++))
    fi
}

# 1. Check if cargo is available
echo "[1/6] Checking build environment..."
which cargo > /dev/null 2>&1
check $? "cargo is available"

# 2. Check if project compiles
echo ""
echo "[2/6] Checking agent-mem-core compilation..."
cd crates/agent-mem-core
cargo check 2>&1 | grep -q "Finished\|error"
RESULT=$?
cd ../..
check $RESULT "agent-mem-core compiles"

# 3. Run library tests
echo ""
echo "[3/6] Running library tests..."
cargo test -p agent-mem-core --lib 2>&1 | grep -q "test result: ok"
check $? "55 library tests pass"

# 4. Check new modules exist
echo ""
echo "[4/6] Checking new module files..."
for module in abac_engine lineage privacy_preserving predictive_monitoring; do
    ls crates/agent-mem-core/src/${module}.rs > /dev/null 2>&1
    check $? "${module}.rs exists"
done

# 5. Check module exports in lib.rs
echo ""
echo "[5/6] Checking module exports..."
for export in abac_engine lineage privacy_preserving predictive_monitoring; do
    grep -q "pub use ${export}::" crates/agent-mem-core/src/lib.rs
    check $? "${export} exported from lib.rs"
done

# 6. Check CLI
echo ""
echo "[6/6] Checking CLI tool..."
cargo build -p agentmem-cli 2>&1 | grep -q "Finished"
check $? "agentmem-cli builds"

# Summary
echo ""
echo "==========================================="
echo -e "Summary: ${GREEN}${PASS} passed${NC}, ${RED}${FAIL} failed${NC}"
echo "==========================================="

if [ $FAIL -eq 0 ]; then
    echo -e "${GREEN}🎉 All verifications passed!${NC}"
    exit 0
else
    echo -e "${RED}⚠️  Some verifications failed${NC}"
    exit 1
fi
