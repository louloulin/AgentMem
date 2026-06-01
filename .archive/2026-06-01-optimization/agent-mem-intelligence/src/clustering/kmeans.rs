//! K-means聚类算法实现

use super::{ClusteringConfig, ClusteringMetrics, ClusteringUtils, MemoryCluster, MemoryClusterer};
use agent_mem_traits::{AgentMemError, Result};
use std::collections::HashMap;

/// K-means聚类器
pub struct KMeansClusterer {
    /// 随机种子
    pub random_seed: u64,
}

impl KMeansClusterer {
    /// 创建新的K-means聚类器
    pub fn new(random_seed: u64) -> Self {
        Self { random_seed }
    }

    /// 使用默认随机种子创建
    pub fn default() -> Self {
        Self::new(42)
    }

    /// 初始化聚类中心
    fn initialize_centroids(&self, vectors: &[Vec<f32>], k: usize) -> Result<Vec<Vec<f32>>> {
        if vectors.is_empty() {
            return Err(AgentMemError::validation_error(
                "Cannot initialize centroids for empty vector set",
            ));
        }

        if k == 0 {
            return Err(AgentMemError::validation_error(
                "Number of clusters must be greater than 0",
            ));
        }

        if k > vectors.len() {
            return Err(AgentMemError::validation_error(
                "Number of clusters cannot exceed number of vectors",
            ));
        }

        let dimension = vectors[0].len();
        let mut centroids = Vec::new();

        // 使用K-means++初始化方法
        // 1. 随机选择第一个中心点
        let first_idx = (self.pseudo_random(0) % vectors.len() as u64) as usize;
        centroids.push(vectors[first_idx].clone());

        // 2. 依次选择剩余的中心点
        for i in 1..k {
            let mut distances = Vec::new();
            let mut total_distance = 0.0;

            // 计算每个点到最近中心点的距离
            for vector in vectors {
                let mut min_distance = f32::INFINITY;
                for centroid in &centroids {
                    let distance = ClusteringUtils::euclidean_distance(vector, centroid)?;
                    min_distance = min_distance.min(distance.powi(2));
                }
                distances.push(min_distance);
                total_distance += min_distance;
            }

            // 基于距离概率选择下一个中心点
            let target = self.pseudo_random(i as u64) as f32 / u64::MAX as f32 * total_distance;
            let mut cumulative = 0.0;
            let mut selected_idx = 0;

            for (idx, &distance) in distances.iter().enumerate() {
                cumulative += distance;
                if cumulative >= target {
                    selected_idx = idx;
                    break;
                }
            }

            centroids.push(vectors[selected_idx].clone());
        }

        Ok(centroids)
    }

    /// 简单的伪随机数生成器
    fn pseudo_random(&self, seed_offset: u64) -> u64 {
        let seed = self.random_seed.wrapping_add(seed_offset);
        seed.wrapping_mul(1103515245).wrapping_add(12345)
    }

    /// 将向量分配到最近的聚类中心
    fn assign_to_clusters(
        &self,
        vectors: &[Vec<f32>],
        centroids: &[Vec<f32>],
    ) -> Result<Vec<usize>> {
        let mut assignments = Vec::new();

        for vector in vectors {
            let mut min_distance = f32::INFINITY;
            let mut best_cluster = 0;

            for (cluster_idx, centroid) in centroids.iter().enumerate() {
                let distance = ClusteringUtils::euclidean_distance(vector, centroid)?;
                if distance < min_distance {
                    min_distance = distance;
                    best_cluster = cluster_idx;
                }
            }

            assignments.push(best_cluster);
        }

        Ok(assignments)
    }

    /// 更新聚类中心
    fn update_centroids(
        &self,
        vectors: &[Vec<f32>],
        assignments: &[usize],
        k: usize,
    ) -> Result<Vec<Vec<f32>>> {
        if vectors.is_empty() {
            return Err(AgentMemError::validation_error(
                "Cannot update centroids for empty vector set",
            ));
        }

        let dimension = vectors[0].len();
        let mut new_centroids = vec![vec![0.0; dimension]; k];
        let mut cluster_counts = vec![0; k];

        // 累加每个聚类中的向量
        for (vector, &cluster_idx) in vectors.iter().zip(assignments.iter()) {
            if cluster_idx < k {
                for (i, &value) in vector.iter().enumerate() {
                    new_centroids[cluster_idx][i] += value;
                }
                cluster_counts[cluster_idx] += 1;
            }
        }

        // 计算平均值
        for (cluster_idx, count) in cluster_counts.iter().enumerate() {
            if *count > 0 {
                for value in new_centroids[cluster_idx].iter_mut() {
                    *value /= *count as f32;
                }
            } else {
                // 如果聚类为空，保持原来的中心点或重新初始化
                // 这里简单地使用第一个向量作为默认值
                new_centroids[cluster_idx] = vectors[0].clone();
            }
        }

        Ok(new_centroids)
    }

    /// 检查收敛性
    fn has_converged(
        &self,
        old_centroids: &[Vec<f32>],
        new_centroids: &[Vec<f32>],
        threshold: f32,
    ) -> Result<bool> {
        if old_centroids.len() != new_centroids.len() {
            return Ok(false);
        }

        for (old, new) in old_centroids.iter().zip(new_centroids.iter()) {
            let distance = ClusteringUtils::euclidean_distance(old, new)?;
            if distance > threshold {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

impl MemoryClusterer for KMeansClusterer {
    fn cluster_memories(
        &self,
        memory_vectors: &[Vec<f32>],
        memory_ids: &[String],
        config: &ClusteringConfig,
    ) -> Result<Vec<MemoryCluster>> {
        if memory_vectors.len() != memory_ids.len() {
            return Err(AgentMemError::validation_error(
                "Memory vectors and IDs must have the same length",
            ));
        }

        if memory_vectors.is_empty() {
            return Ok(Vec::new());
        }

        // 确定聚类数量
        let k = if let Some(num_clusters) = config.num_clusters {
            num_clusters
        } else {
            // 自动确定聚类数量（简单的启发式方法）
            (memory_vectors.len() as f32).sqrt() as usize + 1
        };

        let k = k.min(memory_vectors.len()).max(1);

        // 初始化聚类中心
        let mut centroids = self.initialize_centroids(memory_vectors, k)?;
        let mut assignments = vec![0; memory_vectors.len()];

        // 迭代优化
        for iteration in 0..config.max_iterations {
            // 分配向量到聚类
            let new_assignments = self.assign_to_clusters(memory_vectors, &centroids)?;

            // 更新聚类中心
            let new_centroids = self.update_centroids(memory_vectors, &new_assignments, k)?;

            // 检查收敛性
            if self.has_converged(&centroids, &new_centroids, config.convergence_threshold)? {
                break;
            }

            centroids = new_centroids;
            assignments = new_assignments;

            // 避免无限循环
            if iteration == config.max_iterations - 1 {
                println!("K-means reached maximum iterations without convergence");
            }
        }

        // 构建聚类结果
        let mut clusters = Vec::new();
        for cluster_idx in 0..k {
            let cluster_memory_ids: Vec<String> = assignments
                .iter()
                .enumerate()
                .filter_map(|(i, &assignment)| {
                    if assignment == cluster_idx {
                        Some(memory_ids[i].clone())
                    } else {
                        None
                    }
                })
                .collect();

            if cluster_memory_ids.len() >= config.min_cluster_size {
                let cluster_id = format!("kmeans_cluster_{cluster_idx}");
                let mut cluster = MemoryCluster::new(
                    cluster_id,
                    cluster_memory_ids,
                    centroids[cluster_idx].clone(),
                );

                // 计算聚类内相似度
                let cluster_vectors: Vec<&Vec<f32>> = assignments
                    .iter()
                    .enumerate()
                    .filter_map(|(i, &assignment)| {
                        if assignment == cluster_idx {
                            Some(&memory_vectors[i])
                        } else {
                            None
                        }
                    })
                    .collect();

                if cluster_vectors.len() > 1 {
                    let mut total_similarity = 0.0;
                    let mut count = 0;

                    for i in 0..cluster_vectors.len() {
                        for j in (i + 1)..cluster_vectors.len() {
                            let distance = ClusteringUtils::euclidean_distance(
                                cluster_vectors[i],
                                cluster_vectors[j],
                            )?;
                            let similarity = 1.0 / (1.0 + distance);
                            total_similarity += similarity;
                            count += 1;
                        }
                    }

                    cluster.intra_similarity = total_similarity / count as f32;
                }

                clusters.push(cluster);
            }
        }

        Ok(clusters)
    }

    fn predict_cluster(
        &self,
        memory_vector: &[f32],
        clusters: &[MemoryCluster],
    ) -> Result<Option<usize>> {
        if clusters.is_empty() {
            return Ok(None);
        }

        let mut min_distance = f32::INFINITY;
        let mut best_cluster = 0;

        for (cluster_idx, cluster) in clusters.iter().enumerate() {
            let distance = ClusteringUtils::euclidean_distance(memory_vector, &cluster.centroid)?;
            if distance < min_distance {
                min_distance = distance;
                best_cluster = cluster_idx;
            }
        }

        Ok(Some(best_cluster))
    }

    fn evaluate_clustering(
        &self,
        memory_vectors: &[Vec<f32>],
        clusters: &[MemoryCluster],
    ) -> Result<ClusteringMetrics> {
        if memory_vectors.is_empty() || clusters.is_empty() {
            return Ok(ClusteringMetrics {
                silhouette_score: 0.0,
                intra_cluster_distance: 0.0,
                inter_cluster_distance: 0.0,
                davies_bouldin_index: 0.0,
                calinski_harabasz_index: 0.0,
            });
        }

        // 计算聚类间距离
        let inter_cluster_distance = ClusteringUtils::calculate_inter_cluster_distance(clusters)?;

        // 计算聚类内平均距离
        let memory_map: HashMap<String, Vec<f32>> = clusters
            .iter()
            .flat_map(|cluster| &cluster.memory_ids)
            .zip(memory_vectors.iter())
            .map(|(id, vec)| (id.clone(), vec.clone()))
            .collect();

        let mut total_intra_distance = 0.0;
        let mut cluster_count = 0;

        for cluster in clusters {
            let intra_distance = ClusteringUtils::calculate_intra_cluster_distance(
                memory_vectors,
                cluster,
                &memory_map,
            )?;
            total_intra_distance += intra_distance;
            cluster_count += 1;
        }

        let intra_cluster_distance = if cluster_count > 0 {
            total_intra_distance / cluster_count as f32
        } else {
            0.0
        };

        // 简化的轮廓系数计算
        let silhouette_score = if inter_cluster_distance > 0.0 && intra_cluster_distance > 0.0 {
            (inter_cluster_distance - intra_cluster_distance)
                / inter_cluster_distance.max(intra_cluster_distance)
        } else {
            0.0
        };

        Ok(ClusteringMetrics {
            silhouette_score,
            intra_cluster_distance,
            inter_cluster_distance,
            davies_bouldin_index: 0.0,    // 简化实现
            calinski_harabasz_index: 0.0, // 简化实现
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kmeans_clusterer_creation() {
        let clusterer = KMeansClusterer::new(123);
        assert_eq!(clusterer.random_seed, 123);

        let default_clusterer = KMeansClusterer::default();
        assert_eq!(default_clusterer.random_seed, 42);
    }

    #[test]
    fn test_initialize_centroids() {
        let clusterer = KMeansClusterer::default();
        let vectors = vec![
            vec![1.0, 2.0],
            vec![3.0, 4.0],
            vec![5.0, 6.0],
            vec![7.0, 8.0],
        ];

        let centroids = clusterer.initialize_centroids(&vectors, 2).unwrap();
        assert_eq!(centroids.len(), 2);
        assert_eq!(centroids[0].len(), 2);
        assert_eq!(centroids[1].len(), 2);
    }

    #[test]
    fn test_assign_to_clusters() {
        let clusterer = KMeansClusterer::default();
        let vectors = vec![
            vec![1.0, 1.0],
            vec![2.0, 2.0],
            vec![10.0, 10.0],
            vec![11.0, 11.0],
        ];
        let centroids = vec![vec![1.5, 1.5], vec![10.5, 10.5]];

        let assignments = clusterer.assign_to_clusters(&vectors, &centroids).unwrap();
        assert_eq!(assignments.len(), 4);
        assert_eq!(assignments[0], 0);
        assert_eq!(assignments[1], 0);
        assert_eq!(assignments[2], 1);
        assert_eq!(assignments[3], 1);
    }

    #[test]
    fn test_cluster_memories() {
        let clusterer = KMeansClusterer::default();
        let vectors = vec![
            vec![1.0, 1.0],
            vec![2.0, 2.0],
            vec![10.0, 10.0],
            vec![11.0, 11.0],
        ];
        let memory_ids = vec![
            "mem1".to_string(),
            "mem2".to_string(),
            "mem3".to_string(),
            "mem4".to_string(),
        ];

        let mut config = ClusteringConfig::default();
        config.num_clusters = Some(2);
        config.min_cluster_size = 1;

        let clusters = clusterer
            .cluster_memories(&vectors, &memory_ids, &config)
            .unwrap();
        assert_eq!(clusters.len(), 2);

        for cluster in &clusters {
            assert!(!cluster.memory_ids.is_empty());
            assert_eq!(cluster.centroid.len(), 2);
        }
    }

    #[test]
    fn test_predict_cluster() {
        let clusterer = KMeansClusterer::default();
        let clusters = vec![
            MemoryCluster::new(
                "cluster1".to_string(),
                vec!["mem1".to_string()],
                vec![1.0, 1.0],
            ),
            MemoryCluster::new(
                "cluster2".to_string(),
                vec!["mem2".to_string()],
                vec![10.0, 10.0],
            ),
        ];

        let test_vector = vec![2.0, 2.0];
        let prediction = clusterer.predict_cluster(&test_vector, &clusters).unwrap();
        assert_eq!(prediction, Some(0));

        let test_vector2 = vec![9.0, 9.0];
        let prediction2 = clusterer.predict_cluster(&test_vector2, &clusters).unwrap();
        assert_eq!(prediction2, Some(1));
    }

    #[test]
    fn test_empty_vectors() {
        let clusterer = KMeansClusterer::default();
        let vectors: Vec<Vec<f32>> = vec![];
        let memory_ids: Vec<String> = vec![];
        let config = ClusteringConfig::default();

        let clusters = clusterer
            .cluster_memories(&vectors, &memory_ids, &config)
            .unwrap();
        assert!(clusters.is_empty());
    }

    #[test]
    fn test_single_vector() {
        let clusterer = KMeansClusterer::default();
        let vectors = vec![vec![1.0, 2.0]];
        let memory_ids = vec!["mem1".to_string()];
        let mut config = ClusteringConfig::default();
        config.num_clusters = Some(1);
        config.min_cluster_size = 1;

        let clusters = clusterer
            .cluster_memories(&vectors, &memory_ids, &config)
            .unwrap();
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].memory_ids.len(), 1);
    }
}
