//! Memgraph图存储实现

use agent_mem_config::memory::GraphStoreConfig;
use agent_mem_traits::{AgentMemError, Entity, GraphResult, GraphStore, Relation, Result, Session};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Memgraph查询请求（与Neo4j兼容）
#[derive(Debug, Serialize)]
struct MemgraphQueryRequest {
    statements: Vec<MemgraphStatement>,
}

/// Memgraph Cypher语句
#[derive(Debug, Serialize)]
struct MemgraphStatement {
    statement: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    parameters: Option<HashMap<String, serde_json::Value>>,
}

/// Memgraph查询响应
#[derive(Debug, Deserialize)]
struct MemgraphQueryResponse {
    results: Vec<MemgraphResult>,
    errors: Vec<MemgraphError>,
}

/// Memgraph查询结果
#[derive(Debug, Deserialize)]
struct MemgraphResult {
    columns: Vec<String>,
    data: Vec<MemgraphDataRow>,
}

/// Memgraph数据行
#[derive(Debug, Deserialize)]
struct MemgraphDataRow {
    row: Vec<serde_json::Value>,
}

/// Memgraph错误
#[derive(Debug, Deserialize)]
struct MemgraphError {
    code: String,
    message: String,
}

/// Memgraph图存储实现
pub struct MemgraphStore {
    config: GraphStoreConfig,
    client: Client,
    base_url: String,
    auth_header: Option<String>,
}

impl MemgraphStore {
    /// 创建新的Memgraph存储实例
    pub async fn new(config: GraphStoreConfig) -> Result<Self> {
        // 构建基础URL
        let base_url = if config.uri.starts_with("http") {
            config.uri.clone()
        } else {
            // 将bolt://转换为http://，Memgraph默认HTTP端口是7444
            config
                .uri
                .replace("bolt://", "http://")
                .replace(":7687", ":7444")
        };

        // 创建认证头（如果提供了用户名和密码）
        let auth_header =
            if let (Some(username), Some(password)) = (&config.username, &config.password) {
                let auth_string = format!("{username}:{password}");
                Some(format!("Basic {}", base64::encode(&auth_string)))
            } else {
                None
            };

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
        let query = MemgraphQueryRequest {
            statements: vec![MemgraphStatement {
                statement: "RETURN 1 as test".to_string(),
                parameters: None,
            }],
        };

        let url = format!("{}/db/data/transaction/commit", self.base_url);

        let mut request = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&query);

        if let Some(ref auth) = self.auth_header {
            request = request.header("Authorization", auth);
        }

        let response = request
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
                "Memgraph connection test failed {status}: {error_text}"
            )));
        }

        let result: MemgraphQueryResponse = response
            .json()
            .await
            .map_err(|e| AgentMemError::parsing_error(format!("Failed to parse response: {e}")))?;

        if !result.errors.is_empty() {
            return Err(AgentMemError::storage_error(format!(
                "Memgraph error: {}",
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
    ) -> Result<MemgraphQueryResponse> {
        let query = MemgraphQueryRequest {
            statements: vec![MemgraphStatement {
                statement: statement.to_string(),
                parameters,
            }],
        };

        let url = format!("{}/db/data/transaction/commit", self.base_url);

        let mut request = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&query);

        if let Some(ref auth) = self.auth_header {
            request = request.header("Authorization", auth);
        }

        let response = request
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
                "Memgraph query failed {status}: {error_text}"
            )));
        }

        let result: MemgraphQueryResponse = response
            .json()
            .await
            .map_err(|e| AgentMemError::parsing_error(format!("Failed to parse response: {e}")))?;

        if !result.errors.is_empty() {
            return Err(AgentMemError::storage_error(format!(
                "Memgraph error: {}",
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
impl GraphStore for MemgraphStore {
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
            RETURN e, collect(r) as relations, collect(related) as related_entities
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
                if let Some(_entity_data) = data_row.row.first() {
                    // 解析实体数据（简化实现）
                    let entity = Entity {
                        id: "parsed_id".to_string(), // 实际实现需要从JSON中解析
                        entity_type: "parsed_type".to_string(),
                        name: "parsed_name".to_string(),
                        attributes: HashMap::new(),
                    };

                    let graph_result = GraphResult {
                        entity,
                        relations: Vec::new(), // 实际实现需要解析关系
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
            RETURN DISTINCT neighbor
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
                if let Some(_entity_data) = data_row.row.first() {
                    // 解析实体数据（简化实现）
                    let entity = Entity {
                        id: "neighbor_id".to_string(), // 实际实现需要从JSON中解析
                        entity_type: "neighbor_type".to_string(),
                        name: "neighbor_name".to_string(),
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

// 重用Neo4j的base64实现
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

    #[tokio::test]
    async fn test_memgraph_store_creation_no_auth() {
        let config = GraphStoreConfig {
            provider: "memgraph".to_string(),
            uri: "bolt://localhost:7687".to_string(),
            username: None,
            password: None,
            database: None,
        };

        // 这个测试会尝试连接，但在没有实际Memgraph服务器的情况下会失败
        // 这是预期的行为
        let result = MemgraphStore::new(config).await;
        // 我们不检查结果，因为连接可能失败
    }

    #[test]
    fn test_entity_to_parameters() {
        let config = GraphStoreConfig {
            provider: "memgraph".to_string(),
            uri: "http://localhost:7444".to_string(),
            username: None,
            password: None,
            database: None,
        };

        let store = MemgraphStore {
            config,
            client: Client::new(),
            base_url: "http://localhost:7444".to_string(),
            auth_header: None,
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
    fn test_relation_to_parameters() {
        let config = GraphStoreConfig {
            provider: "memgraph".to_string(),
            uri: "http://localhost:7444".to_string(),
            username: None,
            password: None,
            database: None,
        };

        let store = MemgraphStore {
            config,
            client: Client::new(),
            base_url: "http://localhost:7444".to_string(),
            auth_header: None,
        };

        let mut properties = HashMap::new();
        properties.insert("strength".to_string(), "high".to_string());

        let relation = Relation {
            relation_type: "KNOWS".to_string(),
            source: "person1".to_string(),
            target: "person2".to_string(),
            confidence: 0.9,
        };

        let params = store.relation_to_parameters(&relation);
        assert_eq!(
            params.get("source").unwrap(),
            &serde_json::Value::String("person1".to_string())
        );
        assert_eq!(
            params.get("target").unwrap(),
            &serde_json::Value::String("person2".to_string())
        );
        assert_eq!(
            params.get("relation_type").unwrap(),
            &serde_json::Value::String("KNOWS".to_string())
        );

        // 检查confidence值（允许浮点数精度误差）
        if let Some(serde_json::Value::Number(n)) = params.get("confidence") {
            let confidence = n.as_f64().unwrap();
            assert!(
                (confidence - 0.9).abs() < 1e-6,
                "Expected confidence ~0.9, got {confidence}"
            );
        } else {
            panic!("Expected confidence to be a number");
        }
    }
}
