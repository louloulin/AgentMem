//! 向量操作工具函数

use agent_mem_traits::{AgentMemError, Result};

/// 向量操作工具集
pub struct VectorUtils;

impl VectorUtils {
    /// 标准化向量（L2范数）
    pub fn normalize_l2(vector: &mut [f32]) -> Result<()> {
        let norm = Self::l2_norm(vector);
        if norm == 0.0 {
            return Err(AgentMemError::validation_error(
                "Cannot normalize zero vector",
            ));
        }

        for value in vector.iter_mut() {
            *value /= norm;
        }

        Ok(())
    }

    /// 计算L2范数
    pub fn l2_norm(vector: &[f32]) -> f32 {
        vector.iter().map(|x| x * x).sum::<f32>().sqrt()
    }

    /// 计算L1范数
    pub fn l1_norm(vector: &[f32]) -> f32 {
        vector.iter().map(|x| x.abs()).sum()
    }

    /// 向量加法
    pub fn add_vectors(a: &[f32], b: &[f32]) -> Result<Vec<f32>> {
        if a.len() != b.len() {
            return Err(AgentMemError::validation_error(
                "Vector dimensions must match",
            ));
        }

        Ok(a.iter().zip(b.iter()).map(|(x, y)| x + y).collect())
    }

    /// 向量减法
    pub fn subtract_vectors(a: &[f32], b: &[f32]) -> Result<Vec<f32>> {
        if a.len() != b.len() {
            return Err(AgentMemError::validation_error(
                "Vector dimensions must match",
            ));
        }

        Ok(a.iter().zip(b.iter()).map(|(x, y)| x - y).collect())
    }

    /// 向量标量乘法
    pub fn scalar_multiply(vector: &[f32], scalar: f32) -> Vec<f32> {
        vector.iter().map(|x| x * scalar).collect()
    }

    /// 向量点积
    pub fn dot_product(a: &[f32], b: &[f32]) -> Result<f32> {
        if a.len() != b.len() {
            return Err(AgentMemError::validation_error(
                "Vector dimensions must match",
            ));
        }

        Ok(a.iter().zip(b.iter()).map(|(x, y)| x * y).sum())
    }

    /// 向量平均值
    pub fn average_vectors(vectors: &[Vec<f32>]) -> Result<Vec<f32>> {
        if vectors.is_empty() {
            return Err(AgentMemError::validation_error(
                "Cannot average empty vector list",
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

        let mut result = vec![0.0; dimension];
        for vector in vectors {
            for (i, value) in vector.iter().enumerate() {
                result[i] += value;
            }
        }

        let count = vectors.len() as f32;
        for value in result.iter_mut() {
            *value /= count;
        }

        Ok(result)
    }

    /// 检查向量是否为零向量
    pub fn is_zero_vector(vector: &[f32], epsilon: f32) -> bool {
        vector.iter().all(|&x| x.abs() < epsilon)
    }

    /// 向量维度验证
    pub fn validate_dimension(vector: &[f32], expected_dim: usize) -> Result<()> {
        if vector.len() != expected_dim {
            return Err(AgentMemError::validation_error(format!(
                "Vector dimension mismatch: expected {}, got {}",
                expected_dim,
                vector.len()
            )));
        }
        Ok(())
    }

    /// 批量验证向量维度
    pub fn validate_dimensions(vectors: &[Vec<f32>], expected_dim: usize) -> Result<()> {
        for (i, vector) in vectors.iter().enumerate() {
            if vector.len() != expected_dim {
                return Err(AgentMemError::validation_error(format!(
                    "Vector {} dimension mismatch: expected {}, got {}",
                    i,
                    expected_dim,
                    vector.len()
                )));
            }
        }
        Ok(())
    }

    /// 向量统计信息
    pub fn vector_stats(vector: &[f32]) -> VectorStats {
        if vector.is_empty() {
            return VectorStats::default();
        }

        let mut min = vector[0];
        let mut max = vector[0];
        let mut sum = 0.0;

        for &value in vector {
            if value < min {
                min = value;
            }
            if value > max {
                max = value;
            }
            sum += value;
        }

        let mean = sum / vector.len() as f32;
        let variance =
            vector.iter().map(|&x| (x - mean).powi(2)).sum::<f32>() / vector.len() as f32;

        VectorStats {
            dimension: vector.len(),
            min,
            max,
            mean,
            variance,
            std_dev: variance.sqrt(),
            l1_norm: Self::l1_norm(vector),
            l2_norm: Self::l2_norm(vector),
        }
    }
}

/// 向量统计信息结构
#[derive(Debug, Clone, PartialEq)]
pub struct VectorStats {
    pub dimension: usize,
    pub min: f32,
    pub max: f32,
    pub mean: f32,
    pub variance: f32,
    pub std_dev: f32,
    pub l1_norm: f32,
    pub l2_norm: f32,
}

impl Default for VectorStats {
    fn default() -> Self {
        Self {
            dimension: 0,
            min: 0.0,
            max: 0.0,
            mean: 0.0,
            variance: 0.0,
            std_dev: 0.0,
            l1_norm: 0.0,
            l2_norm: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_l2() {
        let mut vector = vec![3.0, 4.0];
        VectorUtils::normalize_l2(&mut vector).unwrap();

        // 3-4-5三角形，标准化后应该是[0.6, 0.8]
        assert!((vector[0] - 0.6).abs() < 1e-6);
        assert!((vector[1] - 0.8).abs() < 1e-6);

        // 验证L2范数为1
        let norm = VectorUtils::l2_norm(&vector);
        assert!((norm - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_l2_norm() {
        let vector = vec![3.0, 4.0];
        let norm = VectorUtils::l2_norm(&vector);
        assert_eq!(norm, 5.0);
    }

    #[test]
    fn test_l1_norm() {
        let vector = vec![3.0, -4.0];
        let norm = VectorUtils::l1_norm(&vector);
        assert_eq!(norm, 7.0);
    }

    #[test]
    fn test_add_vectors() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 5.0, 6.0];
        let result = VectorUtils::add_vectors(&a, &b).unwrap();
        assert_eq!(result, vec![5.0, 7.0, 9.0]);
    }

    #[test]
    fn test_subtract_vectors() {
        let a = vec![4.0, 5.0, 6.0];
        let b = vec![1.0, 2.0, 3.0];
        let result = VectorUtils::subtract_vectors(&a, &b).unwrap();
        assert_eq!(result, vec![3.0, 3.0, 3.0]);
    }

    #[test]
    fn test_scalar_multiply() {
        let vector = vec![1.0, 2.0, 3.0];
        let result = VectorUtils::scalar_multiply(&vector, 2.0);
        assert_eq!(result, vec![2.0, 4.0, 6.0]);
    }

    #[test]
    fn test_dot_product() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 5.0, 6.0];
        let result = VectorUtils::dot_product(&a, &b).unwrap();
        assert_eq!(result, 32.0); // 1*4 + 2*5 + 3*6 = 4 + 10 + 18 = 32
    }

    #[test]
    fn test_average_vectors() {
        let vectors = vec![vec![1.0, 2.0], vec![3.0, 4.0], vec![5.0, 6.0]];
        let result = VectorUtils::average_vectors(&vectors).unwrap();
        assert_eq!(result, vec![3.0, 4.0]);
    }

    #[test]
    fn test_is_zero_vector() {
        let zero_vector = vec![0.0, 0.0, 0.0];
        assert!(VectorUtils::is_zero_vector(&zero_vector, 1e-6));

        let non_zero_vector = vec![0.0, 0.0, 0.1];
        assert!(!VectorUtils::is_zero_vector(&non_zero_vector, 1e-6));
    }

    #[test]
    fn test_validate_dimension() {
        let vector = vec![1.0, 2.0, 3.0];
        assert!(VectorUtils::validate_dimension(&vector, 3).is_ok());
        assert!(VectorUtils::validate_dimension(&vector, 2).is_err());
    }

    #[test]
    fn test_vector_stats() {
        let vector = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = VectorUtils::vector_stats(&vector);

        assert_eq!(stats.dimension, 5);
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 5.0);
        assert_eq!(stats.mean, 3.0);
        assert!((stats.l2_norm - (55.0_f32).sqrt()).abs() < 1e-6);
    }

    #[test]
    fn test_dimension_mismatch_errors() {
        let a = vec![1.0, 2.0];
        let b = vec![1.0, 2.0, 3.0];

        assert!(VectorUtils::add_vectors(&a, &b).is_err());
        assert!(VectorUtils::subtract_vectors(&a, &b).is_err());
        assert!(VectorUtils::dot_product(&a, &b).is_err());
    }
}
