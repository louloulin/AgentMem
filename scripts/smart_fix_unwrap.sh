#!/bin/bash
# Smart unwrap/expect fixer - Round 2
# Focuses on safe patterns that can be auto-fixed

set -e

CRATE="${1:-crates/agent-mem-core}"

echo "🔧 Round 2: Smart unwrap fixing in $CRATE"
echo "=========================================="
echo ""

# Count before
BEFORE_UNWRAP=$(grep -r "\.unwrap()" "$CRATE/src" --include="*.rs" 2>/dev/null | grep -v test | wc -l | tr -d ' ')
BEFORE_EXPECT=$(grep -r '\.expect("' "$CRATE/src" --include="*.rs" 2>/dev/null | grep -v test | wc -l | tr -d ' ')
BEFORE_TOTAL=$((BEFORE_UNWRAP + BEFORE_EXPECT))

echo "📊 Before: $BEFORE_UNWRAP unwrap, $BEFORE_EXPECT expect (Total: $BEFORE_TOTAL)"
echo ""

# Pattern 1: .get(key).unwrap() on collections -> .get(&key).copied()
echo "🔄 Pattern 1: HashMap/Vec get().unwrap() -> copied()"
echo "   This is safe for Copy types (i32, f64, bool, etc.)"
# Find occurrences
COUNT=$(grep -r '\.get([^)]*)\.unwrap()' "$CRATE/src" --include="*.rs" 2>/dev/null | grep -v test | wc -l | tr -d ' ')
echo "   Found $COUNT occurrences"
echo "   ⚠️  Requires manual review - not all types are Copy"
echo ""

# Pattern 2: chained unwrap -> split with ?
echo "🔄 Pattern 2: Chained .unwrap().unwrap() -> nested ?"
COUNT=$(grep -r '\.unwrap()\.unwrap()' "$CRATE/src" --include="*.rs" 2>/dev/null | grep -v test | wc -l | tr -d ' ')
echo "   Found $COUNT chained unwraps"

FIXED=0
if [ "$COUNT" -gt 0 ]; then
    echo "   🔧 Auto-fixing safe patterns..."
    # Only fix when we have .get().unwrap().unwrap() which is usually row.get(i).unwrap()
    find "$CRATE/src" -name "*.rs" -type f -exec sed -i '' \
        's/\.get(\([^)]*\))\.unwrap()\.unwrap()/.get(\1).ok_or_else(|| Error::MissingValue)?/g' {} \; 2>/dev/null || true
    FIXED=$?
    echo "   ✅ Pattern applied"
fi
echo ""

# Pattern 3: expect() with error message
echo "🔄 Pattern 3: expect() -> context() or ok_or_else()"
COUNT=$(grep -r '\.expect("' "$CRATE/src" --include="*.rs" 2>/dev/null | grep -v test | wc -l | tr -d ' ')
echo "   Found $COUNT expect() calls"
echo "   📋 Manual review required for each"
echo ""

# Pattern 4: unwrap() in test assertions (skip these)
TEST_UNWRAP=$(grep -r "\.unwrap()" "$CRATE/tests" --include="*.rs" 2>/dev/null | wc -l | tr -d ' ')
echo "📋 Test files: $TEST_UNWRAP unwrap (will keep)"
echo ""

# Count after
AFTER_UNWRAP=$(grep -r "\.unwrap()" "$CRATE/src" --include="*.rs" 2>/dev/null | grep -v test | wc -l | tr -d ' ')
AFTER_EXPECT=$(grep -r '\.expect("' "$CRATE/src" --include="*.rs" 2>/dev/null | grep -v test | wc -l | tr -d ' ')
AFTER_TOTAL=$((AFTER_UNWRAP + AFTER_EXPECT))

echo "📊 After: $AFTER_UNWRAP unwrap, $AFTER_EXPECT expect (Total: $AFTER_TOTAL)"
echo ""
echo "📈 Change: -$((BEFORE_UNWRAP - AFTER_UNWRAP)) unwrap, -$((BEFORE_EXPECT - AFTER_EXPECT)) expect"
echo ""

# Show remaining hotspots
echo "🔍 Remaining Hotspots:"
echo ""
echo "Top 10 files with most unwrap():"
grep -r "\.unwrap()" "$CRATE/src" --include="*.rs" 2>/dev/null | grep -v test | sed 's/:.*//' | sort | uniq -c | sort -rn | head -10
echo ""

echo "✅ Round 2 complete!"
echo ""
echo "💡 Manual fixes needed:"
echo "   1. Review .get().unwrap() patterns"
echo "   2. Replace expect() with proper errors"
echo "   3. Add error context where needed"
echo ""
echo "📝 Suggested commands:"
echo "   # Find specific patterns"
echo "   grep -rn '\\.get(' $CRATE/src | grep unwrap"
echo "   grep -rn 'expect' $CRATE/src"
