#!/usr/bin/env python3
"""
批量更新所有 LibSQL Repository 以支持连接池
"""
import re
import sys
from pathlib import Path

def update_repository_file(file_path: Path):
    """更新单个 repository 文件以支持连接池"""
    content = file_path.read_text(encoding='utf-8')
    original = content
    
    # 1. 更新 imports - 添加 LibSqlConnectionPool
    if 'use crate::storage::libsql::connection::LibSqlConnectionPool;' not in content:
        # 找到最后一个 use 语句的位置
        use_pattern = r'(use\s+[^;]+;)'
        uses = list(re.finditer(use_pattern, content))
        if uses:
            last_use = uses[-1]
            insert_pos = last_use.end()
            content = content[:insert_pos] + '\nuse crate::storage::libsql::connection::LibSqlConnectionPool;' + content[insert_pos:]
    
    # 2. 更新 struct 定义
    struct_pattern = r'pub struct (\w+) \{\s+conn: Arc<Mutex<Connection>>,\s+\}'
    replacement = r'''pub struct \1 {
    /// Legacy single-connection mode (Arc<Mutex<Connection>>)
    conn: Option<Arc<Mutex<Connection>>>,
    /// Preferred pooled mode
    pool: Option<Arc<LibSqlConnectionPool>>,
}'''
    content = re.sub(struct_pattern, replacement, content)
    
    # 3. 更新 new() 方法
    new_pattern = r'pub fn new\(conn: Arc<Mutex<Connection>>\) -> Self \{\s+Self \{ conn \}\s+\}'
    replacement = r'''pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self {
            conn: Some(conn),
            pool: None,
        }
    }

    /// Create a new LibSQL repository backed by a connection pool
    pub fn new_with_pool(pool: Arc<LibSqlConnectionPool>) -> Self {
        Self {
            conn: None,
            pool: Some(pool),
        }
    }

    /// Helper to get a connection (from pool if available, otherwise the single conn)
    async fn get_conn(&self) -> Result<Arc<Mutex<Connection>>> {
        if let Some(pool) = &self.pool {
            return pool
                .get()
                .await
                .map_err(|e| AgentMemError::StorageError(format!("Failed to get pooled conn: {e}")));
        }
        if let Some(conn) = &self.conn {
            return Ok(conn.clone());
        }
        Err(AgentMemError::StorageError(
            "No connection or pool available".to_string(),
        ))
    }'''
    content = re.sub(new_pattern, replacement, content)
    
    # 4. 更新所有 self.conn.lock().await 为 self.get_conn().await?; let conn = conn.lock().await;
    # 匹配模式：let conn = self.conn.lock().await;
    conn_lock_pattern = r'let conn = self\.conn\.lock\(\)\.await;'
    replacement = r'let conn = self.get_conn().await?;\n        let conn = conn.lock().await;'
    content = re.sub(conn_lock_pattern, replacement, content)
    
    # 5. 更新所有 self.conn 的其他使用（如果有）
    # 但要注意不要替换已经在 get_conn() 中的 self.conn
    
    if content != original:
        file_path.write_text(content, encoding='utf-8')
        print(f"✅ Updated: {file_path.name}")
        return True
    else:
        print(f"⏭️  Skipped (no changes): {file_path.name}")
        return False

def main():
    repo_dir = Path(__file__).parent.parent / "crates" / "agent-mem-core" / "src" / "storage" / "libsql"
    
    if not repo_dir.exists():
        print(f"❌ Directory not found: {repo_dir}")
        sys.exit(1)
    
    # 需要更新的 repository 文件（排除 memory_repository.rs，它已经更新了）
    repo_files = [
        "agent_repository.rs",
        "api_key_repository.rs",
        "association_repository.rs",
        "block_repository.rs",
        "message_repository.rs",
        "organization_repository.rs",
        "tool_repository.rs",
        "user_repository.rs",  # 已经手动更新，但可以检查
    ]
    
    updated_count = 0
    for repo_file in repo_files:
        file_path = repo_dir / repo_file
        if file_path.exists():
            if update_repository_file(file_path):
                updated_count += 1
        else:
            print(f"⚠️  File not found: {repo_file}")
    
    print(f"\n✅ Updated {updated_count} repository files")

if __name__ == "__main__":
    main()
