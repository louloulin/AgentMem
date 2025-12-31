#!/bin/bash
# Find clone hotspots in AgentMem codebase

echo "🔍 Clone hotspots 分析"
echo "======================"
echo ""

# Analyze agent-mem-core
if [ -d "crates/agent-mem-core/src" ]; then
    echo "📦 agent-mem-core:"
    for file in crates/agent-mem-core/src/**/*.rs; do
        if [ -f "$file" ]; then
            count=$(grep -o "\.clone()" "$file" 2>/dev/null | wc -l | tr -d ' ')
            if [ "$count" -gt 10 ]; then
                relative_path="${file#crates/agent-mem-core/}"
                printf "  %-50s %4d clones\n" "$relative_path" "$count"
            fi
        fi
    done | sort -t: -k2 -rn
fi

echo ""
echo "📊 总计:"
total=$(grep -r "\.clone()" crates/agent-mem-core/src --include="*.rs" 2>/dev/null | wc -l | tr -d ' ')
echo "  Total clones in agent-mem-core: $total"

echo ""
echo "💡 Top patterns:"
echo "  String.clone(): $(grep -r 'String\.clone()' crates/agent-mem-core/src --include="*.rs" 2>/dev/null | wc -l | tr -d ' ')"
echo "  Vec.clone(): $(grep -r 'Vec\.clone()' crates/agent-mem-core/src --include="*.rs" 2>/dev/null | wc -l | tr -d ' ')"
echo "  HashMap.clone(): $(grep -r 'HashMap\.clone()' crates/agent-mem-core/src --include="*.rs" 2>/dev/null | wc -l | tr -d ' ')"
