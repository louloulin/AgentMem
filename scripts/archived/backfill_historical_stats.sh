#!/bin/bash
# Backfill Historical Memory Statistics
# Purpose: Generate historical data points for the last 30 days for demo purposes

DB_PATH="./data/agentmem.db"

echo "🔄 Backfilling historical memory statistics..."

# Get current memory count
CURRENT_COUNT=$(sqlite3 $DB_PATH "SELECT COUNT(*) FROM memories WHERE is_deleted = 0;")
echo "📊 Current memory count: $CURRENT_COUNT"

# Generate historical data for the last 30 days
for i in {29..0}; do
    # Calculate date
    DATE=$(date -v-${i}d +%Y-%m-%d 2>/dev/null || date -d "-${i} days" +%Y-%m-%d)
    
    # Calculate memory count (simulate gradual growth)
    # Start from 70% of current count 30 days ago, grow to 100% today
    PROGRESS=$(echo "scale=2; 0.7 + 0.3 * (30 - $i) / 30" | bc)
    TOTAL=$(echo "$CURRENT_COUNT * $PROGRESS" | bc | cut -d. -f1)
    
    # Calculate new memories (difference from previous day)
    if [ $i -eq 29 ]; then
        NEW=$TOTAL
    else
        PREV_PROGRESS=$(echo "scale=2; 0.7 + 0.3 * (29 - $i) / 30" | bc)
        PREV_TOTAL=$(echo "$CURRENT_COUNT * $PREV_PROGRESS" | bc | cut -d. -f1)
        NEW=$((TOTAL - PREV_TOTAL))
        if [ $NEW -lt 0 ]; then NEW=0; fi
    fi
    
    # Random importance (0.5 - 0.9)
    AVG_IMP=$(echo "scale=2; 0.5 + ($RANDOM % 400) / 1000" | bc)
    
    # Insert or replace
    ID="stat-$(echo $DATE | tr -d '-')"
    TIMESTAMP=$(date -j -f "%Y-%m-%d" "$DATE" +%s 2>/dev/null || date -d "$DATE" +%s)
    
    # Calculate memory type distributions
    SEMANTIC=$(($TOTAL * 6 / 10))
    EPISODIC=$(($TOTAL * 3 / 10))
    WORKING=$(($TOTAL * 1 / 10))
    
    # Create proper JSON string
    JSON_TYPE="{\"Semantic\": $SEMANTIC, \"Episodic\": $EPISODIC, \"Working\": $WORKING}"
    
    sqlite3 $DB_PATH <<SQL
INSERT OR REPLACE INTO memory_stats_daily (
    id,
    date,
    total_memories,
    new_memories,
    deleted_memories,
    memories_by_type,
    avg_importance,
    created_at,
    updated_at
) VALUES (
    '$ID',
    '$DATE',
    $TOTAL,
    $NEW,
    0,
    '$JSON_TYPE',
    $AVG_IMP,
    $TIMESTAMP,
    $TIMESTAMP
);
SQL
    
    echo "✅ $DATE: total=$TOTAL, new=$NEW, avg_importance=$AVG_IMP"
done

echo ""
echo "📊 Historical data backfill complete!"
echo ""
echo "Verifying data..."
COUNT=$(sqlite3 $DB_PATH "SELECT COUNT(*) FROM memory_stats_daily;")
echo "✅ Total daily stats records: $COUNT"

echo ""
echo "Recent records:"
sqlite3 $DB_PATH "SELECT date, total_memories, new_memories, ROUND(avg_importance, 2) FROM memory_stats_daily ORDER BY date DESC LIMIT 5;"

echo ""
echo "🎉 Backfill complete! Visit http://localhost:3001/admin to see the Memory Growth Trend"

