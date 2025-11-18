#!/bin/bash
# Phase 0 持久化验证测试脚本
# 按照 ag25.md Phase 0.3 的测试步骤

set -e

echo "🧪 Phase 0: 持久化验证测试"
echo "======================================"

# 清理旧数据
echo ""
echo "📝 Step 1: 清理旧数据..."
rm -f ./data/agentmem.db ./data/agentmem.db-shm ./data/agentmem.db-wal
mkdir -p ./data
echo "✅ 数据清理完成"

# 检查编译
echo ""
echo "📝 Step 2: 检查编译状态..."
if [ ! -f "./target/release/agentmem_server" ]; then
    echo "⚠️  agentmem_server 未编译，开始编译..."
    cargo build --release --bin agentmem_server
fi
echo "✅ 编译检查完成"

# 第一次写入测试
echo ""
echo "📝 Step 3: 第一次写入测试（模拟）..."
echo "提示: 需要手动运行 ./test_zhipu_memory.sh 或启动服务器并调用API"
echo ""
echo "📊 当前数据库状态:"
if [ -f "./data/agentmem.db" ]; then
    echo "检查 memories 表记录数:"
    sqlite3 ./data/agentmem.db "SELECT COUNT(*) as total_memories FROM memories;" 2>/dev/null || echo "表不存在或数据库为空"
    
    echo ""
    echo "最近10条记忆（如果有）:"
    sqlite3 ./data/agentmem.db << 'EOF' 2>/dev/null || echo "暂无数据"
.mode column
.headers on
SELECT 
    substr(id, 1, 8) as id_prefix,
    substr(user_id, 1, 10) as user,
    substr(agent_id, 1, 10) as agent,
    substr(content, 1, 40) as content_preview,
    datetime(created_at, 'unixepoch') as created_time
FROM memories 
ORDER BY created_at DESC 
LIMIT 10;
EOF
else
    echo "❌ 数据库文件不存在，请先运行测试"
fi

echo ""
echo "======================================"
echo "📋 Phase 0 验证清单:"
echo ""
echo "手动测试步骤:"
echo "1. 运行: ./test_zhipu_memory.sh"
echo "   或启动服务器并发送消息"
echo ""
echo "2. 验证数据写入:"
echo "   ./test_phase0_persistence.sh"
echo ""
echo "3. 重启服务器"
echo ""
echo "4. 再次查询，验证数据仍在:"
echo "   sqlite3 ./data/agentmem.db 'SELECT COUNT(*) FROM memories;'"
echo ""
echo "✅ 成功标准:"
echo "   - 数据写入 SQLite memories 表"
echo "   - 重启后数据仍在"
echo "   - get_all() 返回历史记忆"
echo "======================================"
