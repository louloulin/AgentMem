#!/usr/bin/env python3
"""
清理FFI模块，只保留C FFI相关的代码
"""

def clean_ffi_module():
    """清理FFI模块"""
    
    ffi_content = """// FFI 模块 - C语言接口
// 从 lib.rs 自动拆分生成

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::ptr;
use tokio::runtime::Runtime;
use crate::core::*;
use crate::agent_state::*;
use crate::memory::*;

// C FFI接口 - Agent状态数据库
#[repr(C)]
pub struct CAgentStateDB {
    db: *mut AgentStateDB,
}

#[no_mangle]
pub extern "C" fn agent_db_new(db_path: *const c_char) -> *mut CAgentStateDB {
    if db_path.is_null() {
        return ptr::null_mut();
    }

    let path_str = unsafe {
        match CStr::from_ptr(db_path).to_str() {
            Ok(s) => s,
            Err(_) => return ptr::null_mut(),
        }
    };

    let rt = match Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return ptr::null_mut(),
    };

    let db = match rt.block_on(async {
        AgentStateDB::new(path_str).await
    }) {
        Ok(db) => Box::into_raw(Box::new(db)),
        Err(_) => return ptr::null_mut(),
    };

    Box::into_raw(Box::new(CAgentStateDB { db }))
}

#[no_mangle]
pub extern "C" fn agent_db_free(db: *mut CAgentStateDB) {
    if !db.is_null() {
        unsafe {
            let c_db = Box::from_raw(db);
            if !c_db.db.is_null() {
                let _ = Box::from_raw(c_db.db);
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn agent_db_save_state(
    db: *mut CAgentStateDB,
    agent_id: u64,
    session_id: u64,
    state_type: c_int,
    data: *const u8,
    data_len: usize,
) -> c_int {
    if db.is_null() || data.is_null() {
        return -1;
    }

    let c_db = unsafe { &*db };
    let agent_db = unsafe { &*c_db.db };

    let state_type = match state_type {
        0 => StateType::WorkingMemory,
        1 => StateType::LongTermMemory,
        2 => StateType::Context,
        3 => StateType::TaskState,
        4 => StateType::Relationship,
        5 => StateType::Embedding,
        _ => return -1,
    };

    let data_vec = unsafe { std::slice::from_raw_parts(data, data_len).to_vec() };
    let state = AgentState::new(agent_id, session_id, state_type, data_vec);

    let rt = match Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return -1,
    };

    match rt.block_on(agent_db.save_state(&state)) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn agent_db_load_state(
    db: *mut CAgentStateDB,
    agent_id: u64,
    data_out: *mut *mut u8,
    data_len_out: *mut usize,
) -> c_int {
    if db.is_null() || data_out.is_null() || data_len_out.is_null() {
        return -1;
    }

    let c_db = unsafe { &*db };
    let agent_db = unsafe { &*c_db.db };

    let rt = match Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return -1,
    };

    match rt.block_on(agent_db.load_state(agent_id)) {
        Ok(Some(state)) => {
            let data_copy = state.data.into_boxed_slice();
            let len = data_copy.len();
            let ptr = Box::into_raw(data_copy) as *mut u8;

            unsafe {
                *data_out = ptr;
                *data_len_out = len;
            }
            0
        }
        Ok(None) => 1, // Not found
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn agent_db_free_data(data: *mut u8, data_len: usize) {
    if !data.is_null() && data_len > 0 {
        unsafe {
            let _ = Box::from_raw(std::slice::from_raw_parts_mut(data, data_len));
        }
    }
}

// C FFI接口 - 记忆管理器
#[repr(C)]
pub struct CMemoryManager {
    mgr: *mut MemoryManager,
}

#[no_mangle]
pub extern "C" fn memory_manager_new(db_path: *const c_char) -> *mut CMemoryManager {
    if db_path.is_null() {
        return ptr::null_mut();
    }

    let path_str = unsafe {
        match CStr::from_ptr(db_path).to_str() {
            Ok(s) => s,
            Err(_) => return ptr::null_mut(),
        }
    };

    let rt = match Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return ptr::null_mut(),
    };

    let connection = match rt.block_on(async {
        lancedb::connect(path_str).execute().await
    }) {
        Ok(conn) => conn,
        Err(_) => return ptr::null_mut(),
    };

    let mgr = MemoryManager::new(connection);
    let mgr_ptr = Box::into_raw(Box::new(mgr));

    Box::into_raw(Box::new(CMemoryManager { mgr: mgr_ptr }))
}

#[no_mangle]
pub extern "C" fn memory_manager_free(mgr: *mut CMemoryManager) {
    if !mgr.is_null() {
        unsafe {
            let c_mgr = Box::from_raw(mgr);
            if !c_mgr.mgr.is_null() {
                let _ = Box::from_raw(c_mgr.mgr);
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn memory_manager_store_memory(
    mgr: *mut CMemoryManager,
    agent_id: u64,
    memory_type: c_int,
    content: *const c_char,
    importance: f64,
) -> c_int {
    if mgr.is_null() || content.is_null() {
        return -1;
    }

    let c_mgr = unsafe { &*mgr };
    let memory_mgr = unsafe { &*c_mgr.mgr };

    let content_str = unsafe {
        match CStr::from_ptr(content).to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return -1,
        }
    };

    let mem_type = match memory_type {
        0 => MemoryType::Episodic,
        1 => MemoryType::Semantic,
        2 => MemoryType::Procedural,
        3 => MemoryType::Working,
        _ => return -1,
    };

    let memory = Memory::new(agent_id, mem_type, content_str, importance);

    let rt = match Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return -1,
    };

    match rt.block_on(memory_mgr.store_memory(&memory)) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn memory_manager_get_memories(
    mgr: *mut CMemoryManager,
    agent_id: u64,
    memory_count_out: *mut usize,
) -> c_int {
    if mgr.is_null() || memory_count_out.is_null() {
        return -1;
    }

    let c_mgr = unsafe { &*mgr };
    let memory_mgr = unsafe { &*c_mgr.mgr };

    let rt = match Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return -1,
    };

    match rt.block_on(memory_mgr.get_memories_by_agent(agent_id)) {
        Ok(memories) => {
            unsafe {
                *memory_count_out = memories.len();
            }
            0
        }
        Err(_) => -1,
    }
}
"""
    
    with open('src/ffi.rs', 'w', encoding='utf-8') as f:
        f.write(ffi_content)
    
    print("FFI模块已清理完成")

if __name__ == "__main__":
    clean_ffi_module()
