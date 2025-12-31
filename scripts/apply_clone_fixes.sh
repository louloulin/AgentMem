#!/bin/bash
# Apply practical clone optimizations

set -e

CRATE="${1:-crates/agent-mem-core}"

echo "🚀 Applying clone optimizations to $CRATE"
echo "=========================================="
echo ""

# Count before
BEFORE=$(grep -r "\.clone()" "$CRATE/src" --include="*.rs" 2>/dev/null | wc -l | tr -d ' ')
echo "📊 Before: $BEFORE clones"
echo ""

# Pattern 1: Redundant clone before deref-like operations
echo "🔄 Pattern 1: Simplify chained clones"
# Find patterns like: x.clone().as_str() -> just x.as_str() or &x
find "$CRATE/src" -name "*.rs" -type f -exec sed -i '' \
    's/\.clone()\.as_str()/.as_str()/g' {} \; 2>/dev/null || true

# Pattern 2: clone() in comparisons (usually unnecessary)
echo "🔄 Pattern 2: Remove clone in comparisons"
# Not auto-fixing, just reporting
COUNT=$(grep -r "==.*\.clone()" "$CRATE/src" --include="*.rs" 2>/dev/null | wc -l | tr -d ' ')
echo "   Found $COUNT comparisons with clone (manual review needed)"
echo ""

# Pattern 3: Report most common clone patterns
echo "🔍 Top Clone Patterns:"
echo ""
echo "1. String.clone():"
grep -r "String::clone()" "$CRATE/src" --include="*.rs" 2>/dev/null | wc -l | tr -d ' '
echo ""
echo "2. .clone().await:"
grep -r "\.clone()\.await" "$CRATE/src" --include="*.rs" 2>/dev/null | wc -l | tr -d ' '
echo ""
echo "3. Vec.clone():"
grep -r "Vec::clone()" "$CRATE/src" --include="*.rs" 2>/dev/null | wc -l | tr -d ' '
echo ""

# Count after
AFTER=$(grep -r "\.clone()" "$CRATE/src" --include="*.rs" 2>/dev/null | wc -l | tr -d ' ')

echo "📊 After: $AFTER clones"
echo ""
echo "📈 Change: -$((BEFORE - AFTER)) clones eliminated"
echo ""

echo "💡 Next manual optimizations:"
echo "   1. Change function params: String -> &str"
echo "   2. Change function params: Vec<T> -> &[T]"
echo "   3. Use Arc<T> for shared data"
echo "   4. Use &str instead of &String"
echo ""
echo "📝 Example:"
echo "   fn process(s: String)  ->  fn process(s: &str)"
echo "   fn items(v: Vec<T>)   ->  fn items(v: &[T])"
echo ""

echo "✅ Clone optimization pass complete!"
