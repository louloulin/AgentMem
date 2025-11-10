//! Neo4j图存储实现

use agent_mem_config::memory::GraphStoreConfig;
use agent_mem_traits::{AgentMemError, Entity, GraphResult, GraphStore, Relation, Result, Session};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Neo4j Cypher查询请求
#[derive(Debug, Serialize)]
struct Neo4jQueryRequest {
    statements: Vec<Neo4jStatement>,
}

/// Neo4j Cypher语句
#[derive(Debug, Serialize)]
struct Neo4jStatement {
    statement: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    parameters: Option<HashMap<String, serde_json::Value>>,
}

/// Neo4j查询响应
#[derive(Debug, Deserialize)]
struct Neo4jQueryResponse {
    results: Vec<Neo4jResult>,
    errors: Vec<Neo4jError>,
}

/// Neo4j查询结果
#[derive(Debug, Deserialize)]
struct Neo4jResult {
    columns: Vec<String>,
    data: Vec<Neo4jDataRow>,
}

/// Neo4j数据行
#[derive(Debug, Deserialize)]
struct Neo4jDataRow {
    row: Vec<serde_json::Value>,
}

/// Neo4j错误
#[derive(Debug, Deserialize)]
struct Neo4jError {
    code: String,
    message: String,
}

/// Neo4j图存储实现
pub struct Neo4jStore {
    config: GraphStoreConfig,
    client: Client,
    base_url: String,
    auth_header: String,
}

impl Neo4jStore {
    /// 创建新的Neo4j存储实例
    pub async fn new(config: GraphStoreConfig) -> Result<Self> {
        let username = config
            .username
            .as_ref()
            .ok_or_else(|| AgentMemError::config_error("Neo4j username is required"))?;
        let password = config
            .password
            .as_ref()
            .ok_or_else(|| AgentMemError::config_error("Neo4j password is required"))?;

        // 构建基础URL
        let base_url = if config.uri.starts_with("http") {
            config.uri.clone()
        } else {
            // 将bolt://转换为http://
            config
                .uri
                .replace("bolt://", "http://")
                .replace(":7687", ":7474")
        };

        // 创建基本认证头
        let auth_string = format!("{username}:{password}");
        let auth_header = format!("Basic {}", base64::encode(&auth_string));

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| {
                AgentMemError::network_error(format!("Failed to create HTTP client: {e}"))
            })?;

        let store = Self {
            config,
            client,
            base_url,
            auth_header,
        };

        // 测试连接
        store.test_connection().await?;

        Ok(store)
    }

    /// 测试数据库连接
    async fn test_connection(&self) -> Result<()> {
        let query = Neo4jQueryRequest {
            statements: vec![Neo4jStatement {
                statement: "RETURN 1 as test".to_string(),
                parameters: None,
            }],
        };

        let url = format!(
            "{}/db/{}/tx/commit",
            self.base_url,
            self.config
                .database
                .as_ref()
                .unwrap_or(&"neo4j".to_string())
        );

        let response = self
            .client
            .post(&url)
            .header("Authorization", &self.auth_header)
            .header("Content-Type", "application/json")
            .json(&query)
            .send()
            .await
            .map_err(|e| AgentMemError::network_error(format!("Connection test failed: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AgentMemError::storage_error(format!(
                "Neo4j connection test failed {status}: {error_text}"
            )));
        }

        let result: Neo4jQueryResponse = response
            .json()
            .await
            .map_err(|e| AgentMemError::parsing_error(format!("Failed to parse response: {e}")))?;

        if !result.errors.is_empty() {
            return Err(AgentMemError::storage_error(format!(
                "Neo4j error: {}",
                result.errors[0].message
            )));
        }

        Ok(())
    }

    /// 执行Cypher查询
    async fn execute_query(
        &self,
        statement: &str,
        parameters: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<Neo4jQueryResponse> {
        let query = Neo4jQueryRequest {
            statements: vec![Neo4jStatement {
                statement: statement.to_string(),
                parameters,
            }],
        };

        let url = format!(
            "{}/db/{}/tx/commit",
            self.base_url,
            self.config
                .database
                .as_ref()
                .unwrap_or(&"neo4j".to_string())
        );

        let response = self
            .client
            .post(&url)
            .header("Authorization", &self.auth_header)
            .header("Content-Type", "application/json")
            .json(&query)
            .send()
            .await
            .map_err(|e| AgentMemError::network_error(format!("Query execution failed: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AgentMemError::storage_error(format!(
                "Neo4j query failed {status}: {error_text}"
            )));
        }

        let result: Neo4jQueryResponse = response
            .json()
            .await
            .map_err(|e| AgentMemError::parsing_error(format!("Failed to parse response: {e}")))?;

        if !result.errors.is_empty() {
            return Err(AgentMemError::storage_error(format!(
                "Neo4j error: {}",
                result.errors[0].message
            )));
        }

        Ok(result)
    }

    /// 将Entity转换为Cypher参数
    fn entity_to_parameters(&self, entity: &Entity) -> HashMap<String, serde_json::Value> {
        let mut params = HashMap::new();
        params.insert(
            "id".to_string(),
            serde_json::Value::String(entity.id.clone()),
        );
        params.insert(
            "entity_type".to_string(),
            serde_json::Value::String(entity.entity_type.clone()),
        );
        params.insert(
            "name".to_string(),
            serde_json::Value::String(entity.name.clone()),
        );

        // 添加属性
        for (key, value) in &entity.attributes {
            params.insert(key.clone(), value.clone());
        }

        params
    }

    /// 将Relation转换为Cypher参数
    fn relation_to_parameters(&self, relation: &Relation) -> HashMap<String, serde_json::Value> {
        let mut params = HashMap::new();
        params.insert(
            "source".to_string(),
            serde_json::Value::String(relation.source.clone()),
        );
        params.insert(
            "target".to_string(),
            serde_json::Value::String(relation.target.clone()),
        );
        params.insert(
            "relation_type".to_string(),
            serde_json::Value::String(relation.relation_type.clone()),
        );
        params.insert(
            "confidence".to_string(),
            serde_json::Value::Number(
                serde_json::Number::from_f64(relation.confidence as f64).unwrap(),
            ),
        );

        params
    }
}

#[async_trait]
impl GraphStore for Neo4jStore {
    async fn add_entities(&self, entities: &[Entity], _session: &Session) -> Result<()> {
        for entity in entities {
            let statement = r#"
                MERGE (e:Entity {id: $id})
                SET e.entity_type = $entity_type,
                    e.name = $name,
                    e += $properties
                RETURN e
            "#;

            let mut parameters = self.entity_to_parameters(entity);

            // 将属性作为单独的参数传递
            let attributes: HashMap<String, serde_json::Value> = entity.attributes.clone();
            parameters.insert(
                "properties".to_string(),
                serde_json::Value::Object(attributes.into_iter().collect()),
            );

            self.execute_query(statement, Some(parameters)).await?;
        }

        Ok(())
    }

    async fn add_relations(&self, relations: &[Relation], _session: &Session) -> Result<()> {
        for relation in relations {
            let statement = r#"
                MATCH (from:Entity {id: $source})
                MATCH (to:Entity {id: $target})
                MERGE (from)-[r:RELATES {type: $relation_type, confidence: $confidence}]->(to)
                RETURN r
            "#;

            let parameters = self.relation_to_parameters(relation);

            self.execute_query(statement, Some(parameters)).await?;
        }

        Ok(())
    }

    async fn search_graph(&self, query: &str, _session: &Session) -> Result<Vec<GraphResult>> {
        // 简单的文本搜索实现
        let statement = r#"
            MATCH (e:Entity)
            WHERE e.name CONTAINS $query OR e.entity_type CONTAINS $query
            OPTIONAL MATCH (e)-[r:RELATES]-(related:Entity)
            RETURN e.id as entity_id, e.entity_type as entity_type, e.name as entity_name,
                   collect(DISTINCT {source: r.source, target: r.target, type: r.type, confidence: r.confidence}) as relations
            LIMIT 10
        "#;

        let mut parameters = HashMap::new();
        parameters.insert(
            "query".to_string(),
            serde_json::Value::String(query.to_string()),
        );

        let response = self.execute_query(statement, Some(parameters)).await?;

        let mut results = Vec::new();

        for result in response.results {
            for data_row in result.data {
                if data_row.row.len() >= 4 {
                    // 解析实体数据
                    let entity_id = data_row.row[0].as_str().unwrap_or("unknown_id").to_string();
                    let entity_type = data_row.row[1]
                        .as_str()
                        .unwrap_or("unknown_type")
                        .to_string();
                    let entity_name = data_row.row[2]
                        .as_str()
                        .unwrap_or("unknown_name")
                        .to_string();

                    let entity = Entity {
                        id: entity_id,
                        entity_type,
                        name: entity_name,
                        attributes: HashMap::new(),
                    };

                    // 解析关系数据
                    let mut relations = Vec::new();
                    if let Some(relations_array) = data_row.row[3].as_array() {
                        for relation_obj in relations_array {
                            if let Some(rel_map) = relation_obj.as_object() {
                                if let (
                                    Some(source),
                                    Some(target),
                                    Some(rel_type),
                                    Some(confidence),
                                ) = (
                                    rel_map.get("source").and_then(|v| v.as_str()),
                                    rel_map.get("target").and_then(|v| v.as_str()),
                                    rel_map.get("type").and_then(|v| v.as_str()),
                                    rel_map.get("confidence").and_then(|v| v.as_f64()),
                                ) {
                                    relations.push(Relation {
                                        relation_type: rel_type.to_string(),
                                        source: source.to_string(),
                                        target: target.to_string(),
                                        confidence: confidence as f32,
                                    });
                                }
                            }
                        }
                    }

                    let graph_result = GraphResult {
                        entity,
                        relations,
                        score: 1.0,
                    };

                    results.push(graph_result);
                }
            }
        }

        Ok(results)
    }

    async fn get_neighbors(&self, entity_id: &str, depth: usize) -> Result<Vec<Entity>> {
        let statement = format!(
            r#"
            MATCH (start:Entity {{id: $entity_id}})
            MATCH (start)-[*1..{depth}]-(neighbor:Entity)
            RETURN DISTINCT neighbor.id as id, neighbor.entity_type as entity_type, neighbor.name as name
            LIMIT 50
        "#
        );

        let mut parameters = HashMap::new();
        parameters.insert(
            "entity_id".to_string(),
            serde_json::Value::String(entity_id.to_string()),
        );

        let response = self.execute_query(&statement, Some(parameters)).await?;

        let mut entities = Vec::new();

        for result in response.results {
            for data_row in result.data {
                if data_row.row.len() >= 3 {
                    // 解析实体数据
                    let id = data_row.row[0].as_str().unwrap_or("unknown_id").to_string();
                    let entity_type = data_row.row[1]
                        .as_str()
                        .unwrap_or("unknown_type")
                        .to_string();
                    let name = data_row.row[2]
                        .as_str()
                        .unwrap_or("unknown_name")
                        .to_string();

                    let entity = Entity {
                        id,
                        entity_type,
                        name,
                        attributes: HashMap::new(),
                    };

                    entities.push(entity);
                }
            }
        }

        Ok(entities)
    }

    async fn reset(&self) -> Result<()> {
        let statement = "MATCH (n) DETACH DELETE n";
        self.execute_query(statement, None).await?;
        Ok(())
    }
}

// 添加base64编码功能的简单实现
mod base64 {
    pub fn encode(input: &str) -> String {
        const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut result = String::new();
        let bytes = input.as_bytes();

        for chunk in bytes.chunks(3) {
            let mut buf = [0u8; 3];
            for (i, &byte) in chunk.iter().enumerate() {
                buf[i] = byte;
            }

            let b = ((buf[0] as u32) << 16) | ((buf[1] as u32) << 8) | (buf[2] as u32);

            result.push(CHARS[((b >> 18) & 63) as usize] as char);
            result.push(CHARS[((b >> 12) & 63) as usize] as char);
            result.push(if chunk.len() > 1 {
                CHARS[((b >> 6) & 63) as usize] as char
            } else {
                '='
            });
            result.push(if chunk.len() > 2 {
                CHARS[(b & 63) as usize] as char
            } else {
                '='
            });
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_mem_config::memory::GraphStoreConfig;

    #[test]
    fn test_neo4j_store_creation_no_username() {
        let config = GraphStoreConfig {
            provider: "neo4j".to_string(),
            uri: "bolt://localhost:7687".to_string(),
            username: None,
            password: Some("password".to_string()),
            database: Some("neo4j".to_string()),
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(Neo4jStore::new(config));
        assert!(result.is_err());
    }

    #[test]
    fn test_neo4j_store_creation_no_password() {
        let config = GraphStoreConfig {
            provider: "neo4j".to_string(),
            uri: "bolt://localhost:7687".to_string(),
            username: Some("neo4j".to_string()),
            password: None,
            database: Some("neo4j".to_string()),
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(Neo4jStore::new(config));
        assert!(result.is_err());
    }

    #[test]
    fn test_entity_to_parameters() {
        let config = GraphStoreConfig {
            provider: "neo4j".to_string(),
            uri: "http://localhost:7474".to_string(),
            username: Some("neo4j".to_string()),
            password: Some("password".to_string()),
            database: Some("neo4j".to_string()),
        };

        let store = Neo4jStore {
            config,
            client: Client::new(),
            base_url: "http://localhost:7474".to_string(),
            auth_header: "Basic bmVvNGo6cGFzc3dvcmQ=".to_string(),
        };

        let mut properties = HashMap::new();
        properties.insert(
            "key1".to_string(),
            serde_json::Value::String("value1".to_string()),
        );

        let entity = Entity {
            id: "test-id".to_string(),
            entity_type: "Person".to_string(),
            name: "Test Person".to_string(),
            attributes: properties,
        };

        let params = store.entity_to_parameters(&entity);
        assert_eq!(
            params.get("id").unwrap(),
            &serde_json::Value::String("test-id".to_string())
        );
        assert_eq!(
            params.get("entity_type").unwrap(),
            &serde_json::Value::String("Person".to_string())
        );
        assert_eq!(
            params.get("name").unwrap(),
            &serde_json::Value::String("Test Person".to_string())
        );
        assert_eq!(
            params.get("key1").unwrap(),
            &serde_json::Value::String("value1".to_string())
        );
    }

    #[test]
    fn test_base64_encode() {
        assert_eq!(base64::encode("neo4j:password"), "bmVvNGo6cGFzc3dvcmQ=");
        assert_eq!(base64::encode("test"), "dGVzdA==");
        assert_eq!(base64::encode("hello"), "aGVsbG8=");
    }
}
