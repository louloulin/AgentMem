#!/usr/bin/env python3
"""
自动拆分 src/lib.rs 文件的脚本
将大文件拆分为多个模块
"""

import os
import re
from pathlib import Path

def read_file(filepath):
    """读取文件内容"""
    with open(filepath, 'r', encoding='utf-8') as f:
        return f.read()

def write_file(filepath, content):
    """写入文件内容"""
    with open(filepath, 'w', encoding='utf-8') as f:
        f.write(content)

def extract_section(content, start_pattern, end_pattern=None):
    """提取代码段"""
    start_match = re.search(start_pattern, content, re.MULTILINE)
    if not start_match:
        return None, content
    
    start_pos = start_match.start()
    
    if end_pattern:
        end_match = re.search(end_pattern, content[start_pos:], re.MULTILINE)
        if end_match:
            end_pos = start_pos + end_match.start()
        else:
            end_pos = len(content)
    else:
        # 找到下一个主要结构的开始
        next_patterns = [
            r'^// [A-Z].*',
            r'^pub struct \w+',
            r'^pub enum \w+',
            r'^impl \w+',
            r'^#\[no_mangle\]'
        ]
        
        end_pos = len(content)
        for pattern in next_patterns:
            match = re.search(pattern, content[start_pos + 100:], re.MULTILINE)
            if match:
                candidate_pos = start_pos + 100 + match.start()
                if candidate_pos < end_pos:
                    end_pos = candidate_pos
    
    extracted = content[start_pos:end_pos].strip()
    remaining = content[:start_pos] + content[end_pos:]
    
    return extracted, remaining

def split_lib_rs():
    """拆分 lib.rs 文件"""
    lib_path = Path('src/lib.rs')
    if not lib_path.exists():
        print(f"错误: {lib_path} 不存在")
        return
    
    content = read_file(lib_path)
    original_content = content
    
    # 创建模块文件的映射
    modules = {
        'ffi.rs': [],
        'performance.rs': [],
        'distributed.rs': [],
        'realtime.rs': [],
        'tests.rs': []
    }
    
    # 提取 C FFI 接口
    print("提取 C FFI 接口...")
    ffi_patterns = [
        (r'// C FFI接口.*?(?=^// [A-Z]|\Z)', 'C FFI接口'),
        (r'#\[repr\(C\)\].*?(?=^// [A-Z]|\Z)', 'C结构体'),
        (r'#\[no_mangle\].*?(?=^#\[no_mangle\]|^// [A-Z]|\Z)', 'C函数')
    ]
    
    for pattern, desc in ffi_patterns:
        while True:
            match = re.search(pattern, content, re.MULTILINE | re.DOTALL)
            if not match:
                break
            
            extracted = match.group(0).strip()
            modules['ffi.rs'].append(f"// {desc}\n{extracted}\n")
            content = content[:match.start()] + content[match.end():]
            print(f"  提取了 {desc}")
    
    # 提取性能监控相关代码
    print("提取性能监控代码...")
    perf_patterns = [
        (r'// 性能监控.*?(?=^// [A-Z]|\Z)', '性能监控'),
        (r'pub struct.*?Monitor.*?(?=^pub struct|^pub enum|^impl|\Z)', '监控结构体'),
        (r'impl.*?Monitor.*?(?=^impl|^pub struct|^pub enum|\Z)', '监控实现')
    ]
    
    for pattern, desc in perf_patterns:
        while True:
            match = re.search(pattern, content, re.MULTILINE | re.DOTALL)
            if not match:
                break
            
            extracted = match.group(0).strip()
            modules['performance.rs'].append(f"// {desc}\n{extracted}\n")
            content = content[:match.start()] + content[match.end():]
            print(f"  提取了 {desc}")
    
    # 提取分布式网络相关代码
    print("提取分布式网络代码...")
    dist_patterns = [
        (r'// 分布式.*?(?=^// [A-Z]|\Z)', '分布式网络'),
        (r'pub struct.*?Network.*?(?=^pub struct|^pub enum|^impl|\Z)', '网络结构体'),
        (r'impl.*?Network.*?(?=^impl|^pub struct|^pub enum|\Z)', '网络实现')
    ]
    
    for pattern, desc in dist_patterns:
        while True:
            match = re.search(pattern, content, re.MULTILINE | re.DOTALL)
            if not match:
                break
            
            extracted = match.group(0).strip()
            modules['distributed.rs'].append(f"// {desc}\n{extracted}\n")
            content = content[:match.start()] + content[match.end():]
            print(f"  提取了 {desc}")
    
    # 提取实时数据流处理相关代码
    print("提取实时数据流处理代码...")
    realtime_patterns = [
        (r'// 实时数据流处理.*?(?=^// [A-Z]|\Z)', '实时数据流处理'),
        (r'pub struct.*?Stream.*?(?=^pub struct|^pub enum|^impl|\Z)', '流处理结构体'),
        (r'impl.*?Stream.*?(?=^impl|^pub struct|^pub enum|\Z)', '流处理实现')
    ]
    
    for pattern, desc in realtime_patterns:
        while True:
            match = re.search(pattern, content, re.MULTILINE | re.DOTALL)
            if not match:
                break
            
            extracted = match.group(0).strip()
            modules['realtime.rs'].append(f"// {desc}\n{extracted}\n")
            content = content[:match.start()] + content[match.end():]
            print(f"  提取了 {desc}")
    
    # 提取测试代码
    print("提取测试代码...")
    test_patterns = [
        (r'#\[cfg\(test\)\].*?(?=^#\[cfg\(test\)\]|^// [A-Z]|\Z)', '测试模块'),
        (r'#\[test\].*?(?=^#\[test\]|^// [A-Z]|\Z)', '测试函数')
    ]
    
    for pattern, desc in test_patterns:
        while True:
            match = re.search(pattern, content, re.MULTILINE | re.DOTALL)
            if not match:
                break
            
            extracted = match.group(0).strip()
            modules['tests.rs'].append(f"// {desc}\n{extracted}\n")
            content = content[:match.start()] + content[match.end():]
            print(f"  提取了 {desc}")
    
    # 写入模块文件
    print("\n写入模块文件...")
    for module_name, sections in modules.items():
        if sections:
            module_path = Path('src') / module_name
            
            # 添加模块头部
            module_content = f"// {module_name.replace('.rs', '').title()} 模块\n"
            module_content += "// 从 lib.rs 自动拆分生成\n\n"
            
            # 添加必要的导入
            if module_name == 'ffi.rs':
                module_content += """use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::ptr;
use crate::core::*;
use crate::agent_state::*;
use crate::memory::*;

"""
            elif module_name == 'performance.rs':
                module_content += """use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use crate::core::*;

"""
            elif module_name == 'distributed.rs':
                module_content += """use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::net::SocketAddr;
use crate::core::*;

"""
            elif module_name == 'realtime.rs':
                module_content += """use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::{Duration, Instant};
use crate::core::*;

"""
            elif module_name == 'tests.rs':
                module_content += """#[cfg(test)]
use super::*;

"""
            
            # 添加提取的代码段
            module_content += '\n'.join(sections)
            
            write_file(module_path, module_content)
            print(f"  创建了 {module_path} ({len(sections)} 个代码段)")
    
    # 创建新的简化 lib.rs
    print("\n创建简化的 lib.rs...")
    new_lib_content = """// Agent状态数据库 - 基于LanceDB的Rust实现
// 模块化架构

// 核心模块
pub mod core;
pub mod agent_state;
pub mod memory;
pub mod vector;
pub mod security;
pub mod performance;
pub mod distributed;
pub mod realtime;
pub mod ffi;

#[cfg(test)]
pub mod tests;

// 重新导出核心类型
pub use core::{
    AgentDbError, AgentState, StateType, Memory, MemoryType,
    DatabaseConfig, QueryResult, PaginationParams, SortOrder
};
pub use agent_state::AgentStateDB;
pub use memory::{MemoryManager, MemoryStatistics};
pub use vector::{AdvancedVectorEngine, VectorSearchResult, SimilarityAlgorithm};
pub use security::{SecurityManager, User, Role, Permission, AccessToken};

// 导入必要的依赖
use std::sync::Arc;
use lancedb::{connect, Connection};

// 主要的集成数据库结构
pub struct AgentDatabase {
    pub agent_state_db: AgentStateDB,
    pub memory_manager: MemoryManager,
    pub vector_engine: Option<AdvancedVectorEngine>,
    pub security_manager: Option<SecurityManager>,
    pub config: DatabaseConfig,
}

impl AgentDatabase {
    pub async fn new(config: DatabaseConfig) -> Result<Self, AgentDbError> {
        let connection = connect(&config.db_path).execute().await?;
        let agent_state_db = AgentStateDB::new(&config.db_path).await?;
        let memory_manager = MemoryManager::new(connection.clone());

        Ok(Self {
            agent_state_db,
            memory_manager,
            vector_engine: None,
            security_manager: None,
            config,
        })
    }

    pub async fn with_vector_engine(mut self, config: vector::VectorIndexConfig) -> Result<Self, AgentDbError> {
        let connection = connect(&self.config.db_path).execute().await?;
        self.vector_engine = Some(AdvancedVectorEngine::new(connection, config));
        Ok(self)
    }

    pub fn with_security_manager(mut self) -> Self {
        self.security_manager = Some(SecurityManager::new());
        self
    }

    // Agent状态操作
    pub async fn save_agent_state(&self, state: &AgentState) -> Result<(), AgentDbError> {
        self.agent_state_db.save_state(state).await
    }

    pub async fn load_agent_state(&self, agent_id: u64) -> Result<Option<AgentState>, AgentDbError> {
        self.agent_state_db.load_state(agent_id).await
    }

    // 记忆操作
    pub async fn store_memory(&self, memory: &Memory) -> Result<(), AgentDbError> {
        self.memory_manager.store_memory(memory).await
    }

    pub async fn get_memories(&self, agent_id: u64) -> Result<Vec<Memory>, AgentDbError> {
        self.memory_manager.get_memories_by_agent(agent_id).await
    }

    // 向量操作
    pub async fn add_vector(&self, id: u64, vector: Vec<f32>, metadata: std::collections::HashMap<String, String>) -> Result<(), AgentDbError> {
        if let Some(ref engine) = self.vector_engine {
            engine.add_vector(id, vector, metadata).await
        } else {
            Err(AgentDbError::Internal("Vector engine not initialized".to_string()))
        }
    }

    pub async fn search_vectors(&self, query: &[f32], limit: usize) -> Result<Vec<VectorSearchResult>, AgentDbError> {
        if let Some(ref engine) = self.vector_engine {
            engine.search_vectors(query, limit).await
        } else {
            Err(AgentDbError::Internal("Vector engine not initialized".to_string()))
        }
    }
}

// 便利函数
pub async fn create_database(db_path: &str) -> Result<AgentDatabase, AgentDbError> {
    let config = DatabaseConfig {
        db_path: db_path.to_string(),
        ..Default::default()
    };
    AgentDatabase::new(config).await
}

pub async fn create_database_with_config(config: DatabaseConfig) -> Result<AgentDatabase, AgentDbError> {
    AgentDatabase::new(config).await
}
"""
    
    # 备份原文件
    backup_path = Path('src/lib.rs.backup')
    write_file(backup_path, original_content)
    print(f"原文件备份到 {backup_path}")
    
    # 写入新的 lib.rs
    write_file(lib_path, new_lib_content)
    print(f"创建了新的简化 {lib_path}")
    
    # 统计信息
    original_lines = len(original_content.splitlines())
    new_lines = len(new_lib_content.splitlines())
    
    print(f"\n拆分完成!")
    print(f"原文件: {original_lines} 行")
    print(f"新文件: {new_lines} 行")
    print(f"减少了: {original_lines - new_lines} 行 ({((original_lines - new_lines) / original_lines * 100):.1f}%)")
    
    # 显示创建的模块
    print(f"\n创建的模块:")
    for module_name in modules.keys():
        module_path = Path('src') / module_name
        if module_path.exists():
            lines = len(read_file(module_path).splitlines())
            print(f"  {module_name}: {lines} 行")

if __name__ == "__main__":
    split_lib_rs()
