//! 图存储模块
//!
//! 提供图数据库集成，支持实体关系存储和图查询

pub mod factory;
pub mod memgraph;
pub mod neo4j;

pub use factory::GraphStoreFactory;
pub use memgraph::MemgraphStore;
pub use neo4j::Neo4jStore;

// 重新导出常用类型
pub use agent_mem_traits::{Entity, GraphResult, GraphStore, Relation, Result, Session};
