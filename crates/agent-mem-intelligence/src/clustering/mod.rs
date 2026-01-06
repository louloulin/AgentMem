//! 记忆聚类分析模块

pub mod dbscan;
pub mod hierarchical;
pub mod kmeans;

pub use dbscan::DBSCANClusterer;
pub use hierarchical::HierarchicalClusterer;
pub use kmeans::KMeansClusterer;

use agent_mem_traits::{AgentMemError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 记忆聚类结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryCluster {
    /// 聚类ID
    pub cluster_id: String,
    /// 聚类中的记忆ID列表
    pub memory_ids: Vec<String>,
    /// 聚类中心向量
    pub centroid: Vec<f32>,
    /// 聚类重要性评分
    pub importance_score: f32,
    /// 聚类大小
    pub size: usize,
    /// 聚类内平均相似度
    pub intra_similarity: f32,
    /// 聚类标签（可选）
    pub label: Option<String>,
    /// 聚类元数据
    pub metadata: HashMap<String, String>,
}

impl MemoryCluster {
    pub fn new(cluster_id: String, memory_ids: Vec<String>, centroid: Vec<f32>) -> Self {
        let size = memory_ids.len();
        Self {
            cluster_id,
            memory_ids,
            centroid,
            importance_score: 0.0,
            size,
            intra_similarity: 0.0,
            label: None,
            metadata: HashMap::new(),
        }
    }

    /// 添加记忆到聚类
    pub fn add_memory(&mut self, memory_id: String) {
        self.memory_ids.push(memory_id);
        self.size = self.memory_ids.len();
    }

    /// 从聚类中移除记忆
    pub fn remove_memory(&mut self, memory_id: &str) -> bool {
        if let Some(pos) = self.memory_ids.iter().position(|id| id == memory_id) {
            self.memory_ids.remove(pos);
            self.size = self.memory_ids.len();
            true
        } else {
            false
        }
    }

    /// 检查聚类是否包含指定记忆
    pub fn contains_memory(&self, memory_id: &str) -> bool {
        self.memory_ids.contains(&memory_id.to_string())
    }

    /// 设置聚类标签
    pub fn set_label(&mut self, label: String) {
        self.label = Some(label);
    }

    /// 添加元数据
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    /// 获取元数据
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}

/// 聚类配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusteringConfig {
    /// 聚类算法类型
    pub algorithm: String,
    /// 聚类数量（用于K-means）
    pub num_clusters: Option<usize>,
    /// 最小聚类大小
    pub min_cluster_size: usize,
    /// 最大聚类大小
    pub max_cluster_size: Option<usize>,
    /// 相似度阈值
    pub similarity_threshold: f32,
    /// 最大迭代次数
    pub max_iterations: usize,
    /// 收敛阈值
    pub convergence_threshold: f32,
}

impl Default for ClusteringConfig {
    fn default() -> Self {
        Self {
            algorithm: "kmeans".to_string(),
            num_clusters: None,
            min_cluster_size: 2,
            max_cluster_size: None,
            similarity_threshold: 0.7,
            max_iterations: 100,
            convergence_threshold: 1e-4,
        }
    }
}

/// 聚类质量评估指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusteringMetrics {
    /// 轮廓系数
    pub silhouette_score: f32,
    /// 聚类内平均距离
    pub intra_cluster_distance: f32,
    /// 聚类间平均距离
    pub inter_cluster_distance: f32,
    /// Davies-Bouldin指数
    pub davies_bouldin_index: f32,
    /// Calinski-Harabasz指数
    pub calinski_harabasz_index: f32,
}

/// 记忆聚类器接口
pub trait MemoryClusterer {
    /// 对记忆进行聚类
    fn cluster_memories(
        &self,
        memory_vectors: &[Vec<f32>],
        memory_ids: &[String],
        config: &ClusteringConfig,
    ) -> Result<Vec<MemoryCluster>>;

    /// 预测新记忆的聚类
    fn predict_cluster(
        &self,
        memory_vector: &[f32],
        clusters: &[MemoryCluster],
    ) -> Result<Option<usize>>;

    /// 评估聚类质量
    fn evaluate_clustering(
        &self,
        memory_vectors: &[Vec<f32>],
        clusters: &[MemoryCluster],
    ) -> Result<ClusteringMetrics>;
}

/// 聚类工具函数
pub struct ClusteringUtils;

impl ClusteringUtils {
    /// 计算向量间的欧几里得距离
    pub fn euclidean_distance(a: &[f32], b: &[f32]) -> Result<f32> {
        if a.len() != b.len() {
            return Err(AgentMemError::validation_error(
                "Vector dimensions must match",
            ));
        }

        let distance = a
            .iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f32>()
            .sqrt();

        Ok(distance)
    }

    /// 计算向量的质心
    pub fn calculate_centroid(vectors: &[Vec<f32>]) -> Result<Vec<f32>> {
        if vectors.is_empty() {
            return Err(AgentMemError::validation_error(
                "Cannot calculate centroid of empty vector set",
            ));
        }

        let dimension = vectors[0].len();
        for vector in vectors {
            if vector.len() != dimension {
                return Err(AgentMemError::validation_error(
                    "All vectors must have the same dimension",
                ));
            }
        }

        let mut centroid = vec![0.0; dimension];
        for vector in vectors {
            for (i, &value) in vector.iter().enumerate() {
                centroid[i] += value;
            }
        }

        let count = vectors.len() as f32;
        for value in centroid.iter_mut() {
            *value /= count;
        }

        Ok(centroid)
    }

    /// 计算聚类内平均距离
    pub fn calculate_intra_cluster_distance(
        vectors: &[Vec<f32>],
        cluster: &MemoryCluster,
        memory_vectors: &HashMap<String, Vec<f32>>,
    ) -> Result<f32> {
        let cluster_vectors: Vec<&Vec<f32>> = cluster
            .memory_ids
            .iter()
            .filter_map(|id| memory_vectors.get(id))
            .collect();

        if cluster_vectors.len() < 2 {
            return Ok(0.0);
        }

        let mut total_distance = 0.0;
        let mut count = 0;

        for i in 0..cluster_vectors.len() {
            for j in (i + 1)..cluster_vectors.len() {
                let distance = Self::euclidean_distance(cluster_vectors[i], cluster_vectors[j])?;
                total_distance += distance;
                count += 1;
            }
        }

        Ok(total_distance / count as f32)
    }

    /// 计算聚类间平均距离
    pub fn calculate_inter_cluster_distance(clusters: &[MemoryCluster]) -> Result<f32> {
        if clusters.len() < 2 {
            return Ok(0.0);
        }

        let mut total_distance = 0.0;
        let mut count = 0;

        for i in 0..clusters.len() {
            for j in (i + 1)..clusters.len() {
                let distance =
                    Self::euclidean_distance(&clusters[i].centroid, &clusters[j].centroid)?;
                total_distance += distance;
                count += 1;
            }
        }

        Ok(total_distance / count as f32)
    }

    /// 自动确定最优聚类数量（肘部法则）
    pub fn find_optimal_clusters(
        vectors: &[Vec<f32>],
        max_k: usize,
        clusterer: &dyn MemoryClusterer,
    ) -> Result<usize> {
        let mut wcss_values = Vec::new();
        let memory_ids: Vec<String> = (0..vectors.len()).map(|i| i.to_string()).collect();

        for k in 1..=max_k {
            let mut config = ClusteringConfig::default();
            config.num_clusters = Some(k);

            let clusters = clusterer.cluster_memories(vectors, &memory_ids, &config)?;
            let wcss = Self::calculate_wcss(vectors, &clusters, &memory_ids)?;
            wcss_values.push(wcss);
        }

        // 使用肘部法则找到最优K值
        let optimal_k = Self::find_elbow_point(&wcss_values);
        Ok(optimal_k + 1) // +1因为索引从0开始
    }

    /// 计算聚类内平方和（WCSS）
    fn calculate_wcss(
        vectors: &[Vec<f32>],
        clusters: &[MemoryCluster],
        memory_ids: &[String],
    ) -> Result<f32> {
        let mut wcss = 0.0;
        let memory_map: HashMap<String, &Vec<f32>> = memory_ids
            .iter()
            .zip(vectors.iter())
            .map(|(id, vec)| (id.clone(), vec))
            .collect();

        for cluster in clusters {
            for memory_id in &cluster.memory_ids {
                if let Some(vector) = memory_map.get(memory_id) {
                    let distance = Self::euclidean_distance(vector, &cluster.centroid)?;
                    wcss += distance.powi(2);
                }
            }
        }

        Ok(wcss)
    }

    /// 找到肘部点
    fn find_elbow_point(wcss_values: &[f32]) -> usize {
        if wcss_values.len() < 3 {
            return 0;
        }

        let mut max_diff = 0.0;
        let mut elbow_point = 0;

        for i in 1..wcss_values.len() - 1 {
            let diff = wcss_values[i - 1] - 2.0 * wcss_values[i] + wcss_values[i + 1];
            if diff > max_diff {
                max_diff = diff;
                elbow_point = i;
            }
        }

        elbow_point
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_cluster_creation() {
        let cluster = MemoryCluster::new(
            "cluster_1".to_string(),
            vec!["mem_1".to_string(), "mem_2".to_string()],
            vec![1.0, 2.0, 3.0],
        );

        assert_eq!(cluster.cluster_id, "cluster_1");
        assert_eq!(cluster.size, 2);
        assert_eq!(cluster.centroid, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_memory_cluster_operations() {
        let mut cluster = MemoryCluster::new(
            "cluster_1".to_string(),
            vec!["mem_1".to_string()],
            vec![1.0, 2.0],
        );

        // 添加记忆
        cluster.add_memory("mem_2".to_string());
        assert_eq!(cluster.size, 2);
        assert!(cluster.contains_memory("mem_2"));

        // 移除记忆
        assert!(cluster.remove_memory("mem_1"));
        assert_eq!(cluster.size, 1);
        assert!(!cluster.contains_memory("mem_1"));

        // 设置标签和元数据
        cluster.set_label("test_cluster".to_string());
        cluster.add_metadata("type".to_string(), "test".to_string());
        assert_eq!(cluster.label, Some("test_cluster".to_string()));
        assert_eq!(cluster.get_metadata("type"), Some(&"test".to_string()));
    }

    #[test]
    fn test_clustering_utils_euclidean_distance() {
        let a = vec![0.0, 0.0];
        let b = vec![3.0, 4.0];
        let distance = ClusteringUtils::euclidean_distance(&a, &b).unwrap();
        assert!((distance - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_clustering_utils_calculate_centroid() {
        let vectors = vec![vec![1.0, 2.0], vec![3.0, 4.0], vec![5.0, 6.0]];
        let centroid = ClusteringUtils::calculate_centroid(&vectors).unwrap();
        assert_eq!(centroid, vec![3.0, 4.0]);
    }

    #[test]
    fn test_clustering_config_default() {
        let config = ClusteringConfig::default();
        assert_eq!(config.algorithm, "kmeans");
        assert_eq!(config.min_cluster_size, 2);
        assert_eq!(config.similarity_threshold, 0.7);
    }

    #[test]
    fn test_dimension_mismatch() {
        let a = vec![1.0, 2.0];
        let b = vec![1.0, 2.0, 3.0];
        let result = ClusteringUtils::euclidean_distance(&a, &b);
        assert!(result.is_err());
    }
}
