#!/bin/bash
# Clone optimization script for AgentMem
# Focuses on high-impact, low-risk clone reductions

set -e

CRATE="${1:-.}"
DRY_RUN="${DRY_RUN:-false}"

echo "🚀 Optimizing clones in $CRATE"
echo ""

if [ "$DRY_RUN" = "true" ]; then
    echo "📝 DRY RUN MODE - No changes will be made"
    echo ""
fi

# Count before
BEFORE_CLONES=$(grep -r "\.clone()" "$CRATE/src" --include="*.rs" 2>/dev/null | wc -l | tr -d ' ')

echo "📊 Before: $BEFORE_CLONES clones"
echo ""

# Pattern 1: .clone() on &str in function calls
# Pattern: func(&str.clone()) -> func(&str)
echo "🔄 Pattern 1: Unnecessary &str.clone() in function args"
COUNT=$(grep -r '&.*\.clone()' "$CRATE/src" --include="*.rs" 2>/dev/null | wc -l | tr -d ' ')
echo "   Found $COUNT occurrences"
echo "   ⚠️  Manual review required"
echo ""

# Pattern 2: Redundant clones before deref
# Pattern: x.clone().deref() -> x
echo "🔄 Pattern 2: Redundant .clone().deref()"
COUNT=$(grep -r '\.clone()\.deref()' "$CRATE/src" --include="*.rs" 2>/dev/null | wc -l | tr -d ' ')
echo "   Found $COUNT occurrences"

if [ "$DRY_RUN" != "true" ] && [ "$COUNT" -gt 0 ]; then
    find "$CRATE/src" -name "*.rs" -type f -exec sed -i '' \
        's/\.clone()\.deref()/.as_ref()/g' {} \;
    echo "   ✅ Fixed"
fi
echo ""

# Pattern 3: Clone in loops
echo "🔄 Pattern 3: Clones in loop iterations"
# Find patterns like: for item in items { process(item.clone()) }
echo "   Requires manual refactoring - see clone_optimization_guide.md"
echo ""

# Pattern 4: String clones
echo "🔄 Pattern 4: Unnecessary String.clone() -> &str"
# Find: String::clone() when &str would work
COUNT=$(grep -r 'String::clone()' "$CRATE/src" --include="*.rs" 2>/dev/null | wc -l | tr -d ' ')
echo "   Found $COUNT String::clone() calls"
echo "   ⚠️  Review for refactoring"
echo ""

# Pattern 5: Vec clones
echo "🔄 Pattern 5: Vec<T>.clone() -> &[T]"
COUNT=$(grep -r 'Vec::clone()' "$CRATE/src" --include="*.rs" 2>/dev/null | wc -l | tr -d ' ')
echo "   Found $COUNT Vec::clone() calls"
echo "   ⚠️  Review for slice conversion"
echo ""

# Count after
if [ "$DRY_RUN" != "true" ]; then
    AFTER_CLONES=$(grep -r "\.clone()" "$CRATE/src" --include="*.rs" 2>/dev/null | wc -l | tr -d ' ')

    echo "📊 After: $AFTER_CLONES clones"
    echo ""
    echo "📈 Change: -$((BEFORE_CLONES - AFTER_CLONES)) clones"
fi

echo ""
echo "✅ Clone optimization pass complete!"
echo ""
echo "💡 Manual optimization opportunities:"
echo "   1. Replace String with &str in function signatures"
echo "   2. Use &[T] instead of Vec<T> for read-only access"
echo "   3. Use Arc<T> for shared ownership"
echo "   4. Borrow instead of clone in loops"
echo "   5. See: scripts/clone_optimization_guide.md"
