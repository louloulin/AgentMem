#!/bin/bash
# Automated unwrap/expect fixer for AgentMem project
# This script identifies and categorizes unwrap/expect usage for systematic fixing

set -e

echo "🔍 Analyzing unwrap/expect patterns..."
echo ""

# Count unwrap/expect occurrences
UNWRAP_COUNT=$(grep -r "\.unwrap()" --include="*.rs" crates/ | wc -l | tr -d ' ')
EXPECT_COUNT=$(grep -r "\.expect(" --include="*.rs" crates/ | wc -l | tr -d ' ')
TOTAL=$((UNWRAP_COUNT + EXPECT_COUNT))

echo "📊 Current State:"
echo "   - unwrap(): $UNWRAP_COUNT"
echo "   - expect(): $EXPECT_COUNT"
echo "   - Total: $TOTAL"
echo ""

# Categorize unwrap/expect by context
echo "📂 Categorizing by crate..."
for crate in crates/*/; do
    if [ -d "$crate" ]; then
        crate_name=$(basename "$crate")
        unwrap=$(find "$crate" -name "*.rs" -exec grep -l "\.unwrap()" {} \; 2>/dev/null | wc -l | tr -d ' ')
        expect=$(find "$crate" -name "*.rs" -exec grep -l "\.expect(" {} \; 2>/dev/null | wc -l | tr -d ' ')
        if [ "$unwrap" -gt 0 ] || [ "$expect" -gt 0 ]; then
            echo "   $crate_name: unwrap in $unwrap files, expect in $expect files"
        fi
    fi
done

echo ""
echo "✅ Analysis complete!"
echo ""
echo "💡 Next steps:"
echo "   1. Prioritize crates with most unwrap/expect"
echo "   2. Fix test files separately (can use unwrap)"
echo "   3. Fix lib.rs and core modules first"
echo "   4. Use proper error handling with Result and ?"
