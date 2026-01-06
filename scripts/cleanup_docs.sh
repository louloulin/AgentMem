#!/bin/bash
# AgentMem Open Source Documentation Cleanup Script
# WARNING: This script permanently deletes documentation files
# Review and test before running on the main branch

set -e

echo "================================================"
echo "AgentMem Documentation Cleanup Script"
echo "================================================"
echo ""
echo "This script will:"
echo "1. Remove process documentation (~500 files)"
echo "2. Consolidate redundant READMEs (~1000 files)"
echo "3. Delete internal artifacts (~200 files)"
echo "4. Create streamlined documentation structure"
echo ""
read -p "Continue? (yes/no): " confirm

if [ "$confirm" != "yes" ]; then
    echo "Aborted."
    exit 1
fi

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Counter for deleted files
deleted_count=0

# Function to safely delete files
safe_delete() {
    local pattern=$1
    local description=$2

    echo -e "${YELLOW}Deleting: $description${NC}"
    local count=$(find . -name "$pattern" -type f 2>/dev/null | wc -l)
    if [ $count -gt 0 ]; then
        find . -name "$pattern" -type f -delete 2>/dev/null || true
        echo -e "${GREEN}✓ Deleted $count files${NC}"
        deleted_count=$((deleted_count + count))
    else
        echo -e "${YELLOW}✓ No files found${NC}"
    fi
}

# Phase 1: Remove Process Documentation
echo ""
echo "================================================"
echo "Phase 1: Removing Process Documentation"
echo "================================================"

safe_delete "*IMPLEMENTATION*" "Implementation summaries"
safe_delete "*ANALYSIS*" "Analysis reports"
safe_delete "*PROGRESS*" "Progress reports"
safe_delete "*FINAL_REPORT*" "Final reports"
safe_delete "*STATUS*" "Status documents"
safe_delete "*VERIFICATION*" "Verification reports"
safe_delete "*COMPLETION*" "Completion reports"
safe_delete "*CLEANUP*" "Cleanup summaries"

# Phase 2: Remove Internal Directories
echo ""
echo "================================================"
echo "Phase 2: Removing Internal Directories"
echo "================================================"

internal_dirs=(
    "claudedocs"
    "reports"
    "backup"
    "logs"
    ".pytest_cache"
    "dist"
    "target"
)

for dir in "${internal_dirs[@]}"; do
    if [ -d "$dir" ]; then
        echo -e "${YELLOW}Removing directory: $dir${NC}"
        rm -rf "$dir"
        echo -e "${GREEN}✓ Removed $dir${NC}"
    fi
done

# Phase 3: Remove Redundant Documentation
echo ""
echo "================================================"
echo "Phase 3: Removing Redundant Documentation"
echo "================================================"

# Remove example READMEs (will consolidate)
safe_delete "examples/*/README.md" "Example READMEs"
safe_delete "examples/*/readme.md" "Example readme files"

# Remove tool READMEs (will consolidate)
safe_delete "tools/*/README.md" "Tool READMEs"
safe_delete "tools/*/readme.md" "Tool readme files"

# Remove crate READMEs (will consolidate)
for crate in crates/*/; do
    if [ -f "$crate/README.md" ] && [ "$crate" != "crates/agent-mem/" ]; then
        echo -e "${YELLOW}Archiving: $crate/README.md${NC}"
        # Could archive these instead of deleting
        # rm "$crate/README.md"
    fi
done

# Phase 4: Remove Stale/Generated Files
echo ""
echo "================================================"
echo "Phase 4: Removing Stale/Generated Files"
echo "================================================"

safe_delete "*~" "Backup files"
safe_delete "*.swp" "Swap files"
safe_delete "*.bak" "Bak files"
safe_delete "*.tmp" "Temp files"
safe_delete "*.old" "Old files"

# Phase 5: Remove Specific Known Process Docs
echo ""
echo "================================================"
echo "Phase 5: Removing Specific Process Documents"
echo "================================================"

process_docs=(
    "*AGENTMEM文档批量删除*"
    "*OPENSOURCE_CLEANUP*"
    "*MIGRATION_COMPLETE*"
    "*TEST_REPORT*"
    "*COMPILATION_VERIFICATION*"
)

for doc in "${process_docs[@]}"; do
    safe_delete "$doc" "Process document: $doc"
done

# Summary
echo ""
echo "================================================"
echo "Cleanup Summary"
echo "================================================"
echo -e "${GREEN}Total files deleted: $deleted_count${NC}"
echo ""
echo "Next steps:"
echo "1. Review git changes"
echo "2. Test that examples still work"
echo "3. Create new consolidated documentation"
echo "4. Update main README"
echo ""
echo "To undo this cleanup: git checkout -- ."
echo ""
