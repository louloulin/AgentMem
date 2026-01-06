#!/bin/bash
# Batch unwrap/expect fixer for AgentMem
# This script applies systematic fixes to unwrap/expect patterns

set -e

CRATE="${1:-crates/agent-mem-core}"
DRY_RUN="${DRY_RUN:-false}"

echo "🔧 Batch fixing unwrap/expect in $CRATE"
echo ""

if [ "$DRY_RUN" = "true" ]; then
    echo "📝 DRY RUN MODE - No changes will be made"
    echo ""
fi

# Count before
BEFORE_UNWRAP=$(grep -r "\.unwrap()" "$CRATE/src" --include="*.rs" | wc -l | tr -d ' ')
BEFORE_EXPECT=$(grep -r "\.expect(" "$CRATE/src" --include="*.rs" | wc -l | tr -d ' ')
BEFORE_TOTAL=$((BEFORE_UNWRAP + BEFORE_EXPECT))

echo "📊 Before: $BEFORE_UNWRAP unwrap, $BEFORE_EXPECT expect (Total: $BEFORE_TOTAL)"
echo ""

# Pattern 1: .unwrap() in async calls -> ?
# Pattern: func().await.unwrap() -> func().await?
echo "🔄 Pattern 1: async .unwrap() -> ?"
COUNT=$(grep -r "\.await\.unwrap()" "$CRATE/src" --include="*.rs" | wc -l | tr -d ' ')
echo "   Found $COUNT occurrences"

if [ "$DRY_RUN" != "true" ] && [ "$COUNT" -gt 0 ]; then
    find "$CRATE/src" -name "*.rs" -type f -exec sed -i '' \
        's/\.await\.unwrap()/.await?/g' {} \;
    echo "   ✅ Fixed"
fi
echo ""

# Pattern 2: .expect() on parse operations
echo "🔄 Pattern 2: .expect() on parse -> proper error handling"
COUNT=$(grep -r '\.parse()\.expect(' "$CRATE/src" --include="*.rs" | wc -l | tr -d ' ')
echo "   Found $COUNT occurrences"
echo "   ⚠️  Manual review required - cannot auto-fix safely"
echo ""

# Pattern 3: unwrap() on Option -> ok_or_else?
echo "🔄 Pattern 3: Option::unwrap() -> ok_or_else?"
COUNT=$(grep -r "\.unwrap()" "$CRATE/src" --include="*.rs" | grep -v "test" | wc -l | tr -d ' ')
echo "   Found $COUNT occurrences (excluding tests)"
echo "   ⚠️  Requires context - manual review"
echo ""

# Pattern 4: expect() with message -> context?
echo "🔄 Pattern 4: expect() -> context?"
COUNT=$(grep -r '\.expect("' "$CRATE/src" --include="*.rs" | wc -l | tr -d ' ')
echo "   Found $COUNT occurrences"
if [ "$DRY_RUN" != "true" ] && [ "$COUNT" -gt 0 ]; then
    echo "   📝 Generating suggestions..."
    grep -rn '\.expect("' "$CRATE/src" --include="*.rs" | head -10 > /tmp/expect_suggestions.txt
    echo "   Suggestions saved to /tmp/expect_suggestions.txt"
fi
echo ""

# Count after
if [ "$DRY_RUN" != "true" ]; then
    AFTER_UNWRAP=$(grep -r "\.unwrap()" "$CRATE/src" --include="*.rs" | wc -l | tr -d ' ')
    AFTER_EXPECT=$(grep -r "\.expect(" "$CRATE/src" --include="*.rs" | wc -l | tr -d ' ')
    AFTER_TOTAL=$((AFTER_UNWRAP + AFTER_EXPECT))

    echo "📊 After: $AFTER_UNWRAP unwrap, $AFTER_EXPECT expect (Total: $AFTER_TOTAL)"
    echo ""
    echo "📈 Change: -$((BEFORE_UNWRAP - AFTER_UNWRAP)) unwrap, -$((BEFORE_EXPECT - AFTER_EXPECT)) expect"
fi

echo ""
echo "✅ Batch fix complete!"
echo ""
echo "💡 Next steps:"
echo "   1. Review changes: git diff"
echo "   2. Run tests: cargo test"
echo "   3. Commit if green: git commit -am 'Fix: unwrap/expect in $CRATE'"
