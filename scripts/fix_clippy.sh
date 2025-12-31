#!/bin/bash
# Automated Clippy Warning Fixer for AgentMem project
# Categorizes and provides fixes for common clippy warnings

set -e

echo "🔍 Analyzing clippy warnings..."
echo ""

# Run clippy and save output
CLIPPY_OUTPUT=$(cargo clippy --all-targets 2>&1 || true)
WARNING_COUNT=$(echo "$CLIPPY_OUTPUT" | grep -c "warning:" || echo "0")

echo "📊 Found $WARNING_COUNT clippy warnings"
echo ""

# Categorize warnings by type
echo "📂 Warning Categories:"

# Common clippy warning patterns
declare -A categories
categories[
  "todo"]="not yet implemented"
  ["unwrap"]="unwrap used on errors"
  ["panic"]="panic call"
  ["clone"]="clone on reference"
  ["needless_return"]="needless return"
  ["bool"]="comparison to bool"
  ["if_let_redundant"]="redundant if let"
  ["map_flatten"]="map flatten"
  ["redundant_clone"]="redundant clone"
  ["large_enum"]="large enum variant"
]

for category in "${!categories[@]}"; do
    count=$(echo "$CLIPPY_OUTPUT" | grep -c "$category" || echo "0")
    if [ "$count" -gt 0 ]; then
        description=${categories[$category]}
        echo "   $category: $count ($description)"
    fi
done

echo ""
echo "📝 Top 10 files with most warnings:"
echo "$CLIPPY_OUTPUT" | grep "warning:" | sed 's/:.*//' | sort | uniq -c | sort -rn | head -10

echo ""
echo "💡 Common fixes:"
echo ""
echo "1. **unwrap() on Result**:"
echo "   ❌  let x = func().unwrap();"
echo "   ✅  let x = func()?;"
echo ""
echo "2. **unwrap() on Option**:"
echo "   ❌  let x = optional.unwrap();"
echo "   ✅  let x = optional.ok_or_else(|| \"missing value\")?;"
echo ""
echo "3. **expect() without context**:"
echo "   ❌  x.expect(\"failed\")"
echo "   ✅  x.context(\"failed to parse config\")?"
echo ""
echo "4. **needless_return**:"
echo "   ❌  return x;"
echo "   ✅  x"
echo ""
echo "5. **redundant_clone**:"
echo "   ❌  let s = string.clone(); process(&s)"
echo "   ✅  process(&string)"
echo ""

# Generate fix suggestions
echo "🔧 Auto-fixable warnings:"
AUTOFIX_COUNT=$(echo "$CLIPPY_OUTPUT" | grep -c "warning:.*help:" || echo "0")
echo "   Found $AUTOFIX_COUNT potentially auto-fixable warnings"
echo ""
echo "   To auto-fix:"
echo "   cargo clippy --fix --allow-dirty --allow-staged"
echo ""

# Save detailed report
REPORT_FILE="clippy_analysis_report.txt"
{
    echo "# Clippy Analysis Report"
    echo ""
    echo "Generated: $(date)"
    echo ""
    echo "## Summary"
    echo "- Total warnings: $WARNING_COUNT"
    echo "- Auto-fixable: ~$AUTOFIX_COUNT"
    echo ""
    echo "## All Warnings"
    echo ""
    echo '```'
    echo "$CLIPPY_OUTPUT"
    echo '```'
} > "$REPORT_FILE"

echo "📄 Detailed report saved to: $REPORT_FILE"
echo ""
echo "✅ Analysis complete!"
