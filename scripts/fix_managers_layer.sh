#!/bin/bash
# Managers layer unwrap fixer
# Focuses on safe, high-impact fixes

set -e

MANAGERS_DIR="crates/agent-mem-core/src/managers"

echo "🔧 Fixing managers layer unwrap/expect"
echo "========================================"
echo ""

# Check if directory exists
if [ ! -d "$MANAGERS_DIR" ]; then
    echo "❌ Managers directory not found: $MANAGERS_DIR"
    exit 1
fi

# Count before
BEFORE=$(find "$MANAGERS_DIR" -name "*.rs" ! -name "*test*.rs" -exec grep -c "\.unwrap()" {} \; 2>/dev/null | awk '{s+=$1} END {print s}')
echo "📊 Before: $BEFORE unwrap (excluding tests)"
echo ""

# Create backup
BACKUP_DIR="/tmp/managers_backup_$(date +%Y%m%d_%H%M%S)"
mkdir -p "$BACKUP_DIR"
cp -r "$MANAGERS_DIR" "$BACKUP_DIR/"
echo "💾 Backup saved to: $BACKUP_DIR"
echo ""

# Apply safe fixes
echo "🔄 Applying safe fixes..."

# Pattern 1: In test files, we keep unwrap (they're safe)
# Pattern 2: In production code, fix only safe patterns

FILES=$(find "$MANAGERS_DIR" -name "*.rs" ! -name "*test*.rs" -type f)

for file in $FILES; do
    echo "  Processing $(basename $file)..."
    
    # Only fix async unwrap (already done in Round 1, but verify)
    # Fix simple .get().unwrap() where get returns Option
    # Fix .await.unwrap() -> .await?
    
    # Skip if file is in test directory
    if [[ "$file" == *"test"* ]]; then
        continue
    fi
    
    # Apply fixes
    sed -i.bak 's/\.await\.unwrap()/.await?/g' "$file" 2>/dev/null || true
    
    # Clean up backup files
    rm -f "${file}.bak"
done

echo ""

# Count after
AFTER=$(find "$MANAGERS_DIR" -name "*.rs" ! -name "*test*.rs" -exec grep -c "\.unwrap()" {} \; 2>/dev/null | awk '{s+=$1} END {print s}')

echo "📊 After: $AFTER unwrap (excluding tests)"
echo ""
echo "📈 Change: -$((BEFORE - AFTER)) unwrap eliminated"
echo ""

# Show remaining hotspots
echo "🔍 Remaining unwrap by file:"
echo ""
find "$MANAGERS_DIR" -name "*.rs" ! -name "*test*.rs" -exec sh -c 'count=$(grep -c "\.unwrap()" "$1" 2>/dev/null); if [ "$count" -gt 0 ]; then echo "$count: $1"; fi' _ {} \; | sort -rn | head -10

echo ""
echo "✅ Managers layer fix complete!"
echo ""
echo "💡 Next steps:"
echo "   1. Review changes: git diff crates/agent-mem-core/src/managers/"
echo "   2. Run tests: cargo test -p agent-mem-core --lib"
echo "   3. Commit: git add . && git commit -m 'Fix: reduce unwrap in managers layer'"
