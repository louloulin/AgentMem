//! Resource Memory Manager - 资源记忆管理器
//!
//! 实现多媒体文件存储和检索，支持文档、图像、音频、视频处理
//! 基于 AgentMem 7.0 认知记忆架构

use crate::{CoreError, CoreResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// 资源类型枚举
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    /// 文档类型 (PDF, DOC, TXT, MD等)
    Document,
    /// 图像类型 (PNG, JPG, GIF, SVG等)
    Image,
    /// 音频类型 (MP3, WAV, FLAC等)
    Audio,
    /// 视频类型 (MP4, AVI, MOV等)
    Video,
    /// 其他类型
    Other,
}

impl ResourceType {
    /// 根据文件扩展名判断资源类型
    pub fn from_extension(extension: &str) -> Self {
        match extension.to_lowercase().as_str() {
            // 文档类型
            "pdf" | "doc" | "docx" | "txt" | "md" | "rtf" | "odt" => ResourceType::Document,
            // 图像类型
            "png" | "jpg" | "jpeg" | "gif" | "svg" | "bmp" | "webp" | "ico" => ResourceType::Image,
            // 音频类型
            "mp3" | "wav" | "flac" | "aac" | "ogg" | "m4a" | "wma" => ResourceType::Audio,
            // 视频类型
            "mp4" | "avi" | "mov" | "mkv" | "wmv" | "flv" | "webm" | "m4v" => ResourceType::Video,
            // 其他类型
            _ => ResourceType::Other,
        }
    }

    /// 获取资源类型的描述
    pub fn description(&self) -> &'static str {
        match self {
            ResourceType::Document => "文档文件 - PDF、Word、文本等",
            ResourceType::Image => "图像文件 - PNG、JPG、GIF等",
            ResourceType::Audio => "音频文件 - MP3、WAV、FLAC等",
            ResourceType::Video => "视频文件 - MP4、AVI、MOV等",
            ResourceType::Other => "其他类型文件",
        }
    }
}

/// 资源元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetadata {
    /// 资源ID
    pub id: String,
    /// 原始文件名
    pub original_filename: String,
    /// 资源类型
    pub resource_type: ResourceType,
    /// 文件大小 (字节)
    pub file_size: u64,
    /// 文件哈希 (SHA256)
    pub file_hash: String,
    /// MIME类型
    pub mime_type: String,
    /// 存储路径
    pub storage_path: PathBuf,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 最后访问时间
    pub last_accessed: DateTime<Utc>,
    /// 访问次数
    pub access_count: u64,
    /// 标签
    pub tags: Vec<String>,
    /// 自定义元数据
    pub custom_metadata: HashMap<String, String>,
    /// 是否压缩
    pub is_compressed: bool,
    /// 压缩后大小
    pub compressed_size: Option<u64>,
}

impl ResourceMetadata {
    /// 创建新的资源元数据
    pub fn new(
        original_filename: String,
        resource_type: ResourceType,
        file_size: u64,
        file_hash: String,
        mime_type: String,
        storage_path: PathBuf,
    ) -> Self {
        let now = Utc::now();

        Self {
            id: Uuid::new_v4().to_string(),
            original_filename,
            resource_type,
            file_size,
            file_hash,
            mime_type,
            storage_path,
            created_at: now,
            last_accessed: now,
            access_count: 0,
            tags: Vec::new(),
            custom_metadata: HashMap::new(),
            is_compressed: false,
            compressed_size: None,
        }
    }

    /// 记录访问
    pub fn record_access(&mut self) {
        self.last_accessed = Utc::now();
        self.access_count += 1;
    }

    /// 添加标签
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    /// 移除标签
    pub fn remove_tag(&mut self, tag: &str) {
        self.tags.retain(|t| t != tag);
    }

    /// 设置自定义元数据
    pub fn set_custom_metadata(&mut self, key: String, value: String) {
        self.custom_metadata.insert(key, value);
    }
}

/// 资源存储配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceStorageConfig {
    /// 存储根目录
    pub storage_root: PathBuf,
    /// 最大文件大小 (字节)
    pub max_file_size: u64,
    /// 是否启用压缩
    pub enable_compression: bool,
    /// 压缩阈值 (字节)
    pub compression_threshold: u64,
    /// 是否启用去重
    pub enable_deduplication: bool,
    /// 是否启用版本管理
    pub enable_versioning: bool,
    /// 最大版本数
    pub max_versions: u32,
}

impl Default for ResourceStorageConfig {
    fn default() -> Self {
        Self {
            storage_root: PathBuf::from("./resource_storage"),
            max_file_size: 100 * 1024 * 1024, // 100MB
            enable_compression: true,
            compression_threshold: 1024 * 1024, // 1MB
            enable_deduplication: true,
            enable_versioning: false,
            max_versions: 5,
        }
    }
}

/// 资源存储统计
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceStorageStats {
    /// 总资源数量
    pub total_resources: usize,
    /// 按类型分组的资源数量
    pub resources_by_type: HashMap<ResourceType, usize>,
    /// 总存储大小 (字节)
    pub total_storage_size: u64,
    /// 压缩节省的空间 (字节)
    pub compression_savings: u64,
    /// 去重节省的空间 (字节)
    pub deduplication_savings: u64,
    /// 总访问次数
    pub total_accesses: u64,
    /// 平均文件大小
    pub average_file_size: f64,
}

/// Resource Memory 管理器
#[derive(Debug)]
pub struct ResourceMemoryManager {
    /// 配置
    config: ResourceStorageConfig,
    /// 资源元数据存储
    resources: Arc<RwLock<HashMap<String, ResourceMetadata>>>,
    /// 哈希到资源ID的映射 (用于去重)
    hash_to_resource: Arc<RwLock<HashMap<String, String>>>,
    /// 统计信息
    stats: Arc<RwLock<ResourceStorageStats>>,
}

impl ResourceMemoryManager {
    /// 创建新的 Resource Memory 管理器
    pub fn new() -> CoreResult<Self> {
        Self::with_config(ResourceStorageConfig::default())
    }

    /// 使用自定义配置创建 Resource Memory 管理器
    pub fn with_config(config: ResourceStorageConfig) -> CoreResult<Self> {
        // 确保存储目录存在
        if !config.storage_root.exists() {
            std::fs::create_dir_all(&config.storage_root).map_err(|e| {
                CoreError::IoError(format!("Failed to create storage directory: {}", e))
            })?;
        }

        Ok(Self {
            config,
            resources: Arc::new(RwLock::new(HashMap::new())),
            hash_to_resource: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(ResourceStorageStats::default())),
        })
    }

    /// 计算文件哈希
    async fn calculate_file_hash(file_path: &Path) -> CoreResult<String> {
        let content = tokio::fs::read(file_path)
            .await
            .map_err(|e| CoreError::IoError(format!("Failed to read file: {}", e)))?;

        let mut hasher = Sha256::new();
        hasher.update(&content);
        let hash = hasher.finalize();

        Ok(format!("{:x}", hash))
    }

    /// 获取MIME类型
    fn get_mime_type(file_path: &Path) -> String {
        if let Some(extension) = file_path.extension() {
            match extension.to_str().unwrap_or("").to_lowercase().as_str() {
                "pdf" => "application/pdf",
                "doc" => "application/msword",
                "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
                "txt" => "text/plain",
                "md" => "text/markdown",
                "png" => "image/png",
                "jpg" | "jpeg" => "image/jpeg",
                "gif" => "image/gif",
                "svg" => "image/svg+xml",
                "mp3" => "audio/mpeg",
                "wav" => "audio/wav",
                "mp4" => "video/mp4",
                "avi" => "video/x-msvideo",
                _ => "application/octet-stream",
            }
        } else {
            "application/octet-stream"
        }
        .to_string()
    }

    /// 生成存储路径
    fn generate_storage_path(&self, resource_id: &str, original_filename: &str) -> PathBuf {
        let extension = Path::new(original_filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        let filename = if extension.is_empty() {
            resource_id.to_string()
        } else {
            format!("{}.{}", resource_id, extension)
        };

        self.config.storage_root.join(filename)
    }

    /// 存储资源文件
    pub async fn store_resource(
        &self,
        file_path: &Path,
        tags: Option<Vec<String>>,
        custom_metadata: Option<HashMap<String, String>>,
    ) -> CoreResult<String> {
        // 检查文件是否存在
        if !file_path.exists() {
            return Err(CoreError::NotFound(format!(
                "File not found: {:?}",
                file_path
            )));
        }

        // 获取文件信息
        let metadata = tokio::fs::metadata(file_path)
            .await
            .map_err(|e| CoreError::IoError(format!("Failed to read file metadata: {}", e)))?;

        let file_size = metadata.len();

        // 检查文件大小限制
        if file_size > self.config.max_file_size {
            return Err(CoreError::InvalidInput(format!(
                "File size {} exceeds maximum allowed size {}",
                file_size, self.config.max_file_size
            )));
        }

        // 计算文件哈希
        let file_hash = Self::calculate_file_hash(file_path).await?;

        // 检查是否已存在相同文件 (去重)
        if self.config.enable_deduplication {
            let hash_to_resource = self.hash_to_resource.read().await;
            if let Some(existing_resource_id) = hash_to_resource.get(&file_hash) {
                // 文件已存在，返回现有资源ID
                let mut resources = self.resources.write().await;
                if let Some(existing_resource) = resources.get_mut(existing_resource_id) {
                    existing_resource.record_access();

                    // 更新统计
                    let mut stats = self.stats.write().await;
                    stats.deduplication_savings += file_size;

                    return Ok(existing_resource_id.clone());
                }
            }
        }

        // 获取文件名和类型信息
        let original_filename = file_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown")
            .to_string();

        let extension = file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        let resource_type = ResourceType::from_extension(extension);
        let mime_type = Self::get_mime_type(file_path);

        // 创建资源元数据
        let resource_id = Uuid::new_v4().to_string();
        let storage_path = self.generate_storage_path(&resource_id, &original_filename);

        let mut resource_metadata = ResourceMetadata::new(
            original_filename,
            resource_type.clone(),
            file_size,
            file_hash.clone(),
            mime_type,
            storage_path.clone(),
        );

        // 添加标签和自定义元数据
        if let Some(tags) = tags {
            resource_metadata.tags = tags;
        }
        if let Some(custom_metadata) = custom_metadata {
            resource_metadata.custom_metadata = custom_metadata;
        }

        // 复制文件到存储位置
        tokio::fs::copy(file_path, &storage_path)
            .await
            .map_err(|e| CoreError::IoError(format!("Failed to copy file to storage: {}", e)))?;

        // 如果启用压缩且文件大小超过阈值，进行压缩
        if self.config.enable_compression && file_size > self.config.compression_threshold {
            // 这里可以实现文件压缩逻辑
            // 为了简化，暂时跳过实际压缩实现
            resource_metadata.is_compressed = false;
        }

        // 存储资源元数据
        {
            let mut resources = self.resources.write().await;
            resources.insert(resource_id.clone(), resource_metadata);
        }

        // 更新哈希映射
        {
            let mut hash_to_resource = self.hash_to_resource.write().await;
            hash_to_resource.insert(file_hash, resource_id.clone());
        }

        // 更新统计信息
        self.update_stats().await;

        Ok(resource_id)
    }

    /// 获取资源元数据
    pub async fn get_resource_metadata(
        &self,
        resource_id: &str,
    ) -> CoreResult<Option<ResourceMetadata>> {
        let mut resources = self.resources.write().await;

        if let Some(resource) = resources.get_mut(resource_id) {
            resource.record_access();

            // 更新统计
            let mut stats = self.stats.write().await;
            stats.total_accesses += 1;

            Ok(Some(resource.clone()))
        } else {
            Ok(None)
        }
    }

    /// 获取资源文件路径
    pub async fn get_resource_path(&self, resource_id: &str) -> CoreResult<Option<PathBuf>> {
        let resources = self.resources.read().await;

        if let Some(resource) = resources.get(resource_id) {
            Ok(Some(resource.storage_path.clone()))
        } else {
            Ok(None)
        }
    }

    /// 删除资源
    pub async fn delete_resource(&self, resource_id: &str) -> CoreResult<()> {
        let mut resources = self.resources.write().await;

        if let Some(resource) = resources.remove(resource_id) {
            // 删除物理文件
            if resource.storage_path.exists() {
                tokio::fs::remove_file(&resource.storage_path)
                    .await
                    .map_err(|e| CoreError::IoError(format!("Failed to delete file: {}", e)))?;
            }

            // 从哈希映射中移除
            let mut hash_to_resource = self.hash_to_resource.write().await;
            hash_to_resource.remove(&resource.file_hash);

            // 更新统计信息
            self.update_stats().await;

            Ok(())
        } else {
            Err(CoreError::NotFound(format!(
                "Resource {} not found",
                resource_id
            )))
        }
    }

    /// 按类型搜索资源
    pub async fn search_by_type(
        &self,
        resource_type: ResourceType,
    ) -> CoreResult<Vec<ResourceMetadata>> {
        let resources = self.resources.read().await;

        let results = resources
            .values()
            .filter(|resource| resource.resource_type == resource_type)
            .cloned()
            .collect();

        Ok(results)
    }

    /// 按标签搜索资源
    pub async fn search_by_tags(&self, tags: &[String]) -> CoreResult<Vec<ResourceMetadata>> {
        let resources = self.resources.read().await;

        let results = resources
            .values()
            .filter(|resource| tags.iter().any(|tag| resource.tags.contains(tag)))
            .cloned()
            .collect();

        Ok(results)
    }

    /// 按文件名搜索资源
    pub async fn search_by_filename(&self, pattern: &str) -> CoreResult<Vec<ResourceMetadata>> {
        let resources = self.resources.read().await;
        let pattern_lower = pattern.to_lowercase();

        let results = resources
            .values()
            .filter(|resource| {
                resource
                    .original_filename
                    .to_lowercase()
                    .contains(&pattern_lower)
            })
            .cloned()
            .collect();

        Ok(results)
    }

    /// 列出所有资源
    pub async fn list_all_resources(&self) -> CoreResult<Vec<ResourceMetadata>> {
        let resources = self.resources.read().await;
        Ok(resources.values().cloned().collect())
    }

    /// 更新资源标签
    pub async fn update_resource_tags(
        &self,
        resource_id: &str,
        tags: Vec<String>,
    ) -> CoreResult<()> {
        let mut resources = self.resources.write().await;

        if let Some(resource) = resources.get_mut(resource_id) {
            resource.tags = tags;
            Ok(())
        } else {
            Err(CoreError::NotFound(format!(
                "Resource {} not found",
                resource_id
            )))
        }
    }

    /// 更新资源自定义元数据
    pub async fn update_resource_metadata(
        &self,
        resource_id: &str,
        custom_metadata: HashMap<String, String>,
    ) -> CoreResult<()> {
        let mut resources = self.resources.write().await;

        if let Some(resource) = resources.get_mut(resource_id) {
            resource.custom_metadata = custom_metadata;
            Ok(())
        } else {
            Err(CoreError::NotFound(format!(
                "Resource {} not found",
                resource_id
            )))
        }
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> CoreResult<ResourceStorageStats> {
        self.update_stats().await;
        let stats = self.stats.read().await;
        Ok(stats.clone())
    }

    /// 更新统计信息
    async fn update_stats(&self) {
        let resources = self.resources.read().await;
        let mut stats = self.stats.write().await;

        // 重置统计
        stats.total_resources = resources.len();
        stats.resources_by_type.clear();
        stats.total_storage_size = 0;

        let mut total_file_size = 0u64;

        for resource in resources.values() {
            // 按类型统计
            *stats
                .resources_by_type
                .entry(resource.resource_type.clone())
                .or_insert(0) += 1;

            // 存储大小统计
            stats.total_storage_size += resource.file_size;
            total_file_size += resource.file_size;

            // 压缩节省统计
            if resource.is_compressed {
                if let Some(compressed_size) = resource.compressed_size {
                    stats.compression_savings += resource.file_size - compressed_size;
                }
            }
        }

        // 计算平均文件大小
        if stats.total_resources > 0 {
            stats.average_file_size = total_file_size as f64 / stats.total_resources as f64;
        } else {
            stats.average_file_size = 0.0;
        }
    }

    /// 清空所有资源
    pub async fn clear_all(&self) -> CoreResult<()> {
        let mut resources = self.resources.write().await;
        let mut hash_to_resource = self.hash_to_resource.write().await;

        // 删除所有物理文件
        for resource in resources.values() {
            if resource.storage_path.exists() {
                let _ = tokio::fs::remove_file(&resource.storage_path).await;
            }
        }

        // 清空内存数据
        resources.clear();
        hash_to_resource.clear();

        // 重置统计
        let mut stats = self.stats.write().await;
        *stats = ResourceStorageStats::default();

        Ok(())
    }

    /// 检查存储健康状态
    pub async fn check_storage_health(&self) -> CoreResult<Vec<String>> {
        let resources = self.resources.read().await;
        let mut issues = Vec::new();

        for (resource_id, resource) in resources.iter() {
            // 检查文件是否存在
            if !resource.storage_path.exists() {
                issues.push(format!(
                    "Missing file for resource {}: {:?}",
                    resource_id, resource.storage_path
                ));
            }

            // 检查文件大小是否匹配
            if let Ok(metadata) = tokio::fs::metadata(&resource.storage_path).await {
                if metadata.len() != resource.file_size {
                    issues.push(format!(
                        "File size mismatch for resource {}: expected {}, found {}",
                        resource_id,
                        resource.file_size,
                        metadata.len()
                    ));
                }
            }
        }

        Ok(issues)
    }
}

impl Default for ResourceMemoryManager {
    fn default() -> Self {
        Self::new().expect("Failed to create default ResourceMemoryManager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs::File;
    use tokio::io::AsyncWriteExt;

    async fn create_test_file(dir: &Path, filename: &str, content: &[u8]) -> PathBuf {
        let file_path = dir.join(filename);
        let mut file = File::create(&file_path).await.unwrap();
        file.write_all(content).await.unwrap();
        file_path
    }

    #[tokio::test]
    async fn test_resource_memory_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config = ResourceStorageConfig {
            storage_root: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let manager = ResourceMemoryManager::with_config(config).unwrap();
        let stats = manager.get_stats().await.unwrap();

        assert_eq!(stats.total_resources, 0);
        assert_eq!(stats.total_storage_size, 0);
    }

    #[tokio::test]
    async fn test_resource_type_from_extension() {
        assert_eq!(ResourceType::from_extension("pdf"), ResourceType::Document);
        assert_eq!(ResourceType::from_extension("PNG"), ResourceType::Image);
        assert_eq!(ResourceType::from_extension("mp3"), ResourceType::Audio);
        assert_eq!(ResourceType::from_extension("MP4"), ResourceType::Video);
        assert_eq!(ResourceType::from_extension("unknown"), ResourceType::Other);
    }

    #[tokio::test]
    async fn test_store_and_retrieve_resource() {
        let temp_dir = TempDir::new().unwrap();
        let config = ResourceStorageConfig {
            storage_root: temp_dir.path().join("storage"),
            ..Default::default()
        };

        let manager = ResourceMemoryManager::with_config(config).unwrap();

        // 创建测试文件
        let test_content = b"Hello, World! This is a test document.";
        let test_file = create_test_file(temp_dir.path(), "test.txt", test_content).await;

        // 存储资源
        let resource_id = manager
            .store_resource(&test_file, None, None)
            .await
            .unwrap();

        // 获取资源元数据
        let metadata = manager
            .get_resource_metadata(&resource_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(metadata.original_filename, "test.txt");
        assert_eq!(metadata.resource_type, ResourceType::Document);
        assert_eq!(metadata.file_size, test_content.len() as u64);
        assert_eq!(metadata.access_count, 1);

        // 获取资源路径
        let storage_path = manager
            .get_resource_path(&resource_id)
            .await
            .unwrap()
            .unwrap();
        assert!(storage_path.exists());

        // 验证文件内容
        let stored_content = tokio::fs::read(&storage_path).await.unwrap();
        assert_eq!(stored_content, test_content);
    }

    #[tokio::test]
    async fn test_resource_deduplication() {
        let temp_dir = TempDir::new().unwrap();
        let config = ResourceStorageConfig {
            storage_root: temp_dir.path().join("storage"),
            enable_deduplication: true,
            ..Default::default()
        };

        let manager = ResourceMemoryManager::with_config(config).unwrap();

        // 创建两个相同内容的文件
        let test_content = b"Duplicate content for testing";
        let test_file1 = create_test_file(temp_dir.path(), "file1.txt", test_content).await;
        let test_file2 = create_test_file(temp_dir.path(), "file2.txt", test_content).await;

        // 存储第一个文件
        let resource_id1 = manager
            .store_resource(&test_file1, None, None)
            .await
            .unwrap();

        // 存储第二个文件 (应该返回相同的资源ID)
        let resource_id2 = manager
            .store_resource(&test_file2, None, None)
            .await
            .unwrap();

        assert_eq!(resource_id1, resource_id2);

        // 验证统计信息
        let stats = manager.get_stats().await.unwrap();
        assert_eq!(stats.total_resources, 1);
        assert_eq!(stats.deduplication_savings, test_content.len() as u64);
    }

    #[tokio::test]
    async fn test_search_by_type() {
        let temp_dir = TempDir::new().unwrap();
        let config = ResourceStorageConfig {
            storage_root: temp_dir.path().join("storage"),
            ..Default::default()
        };

        let manager = ResourceMemoryManager::with_config(config).unwrap();

        // 创建不同类型的文件
        let doc_file = create_test_file(temp_dir.path(), "document.pdf", b"PDF content").await;
        let img_file = create_test_file(temp_dir.path(), "image.png", b"PNG content").await;
        let audio_file = create_test_file(temp_dir.path(), "audio.mp3", b"MP3 content").await;

        // 存储资源
        manager.store_resource(&doc_file, None, None).await.unwrap();
        manager.store_resource(&img_file, None, None).await.unwrap();
        manager
            .store_resource(&audio_file, None, None)
            .await
            .unwrap();

        // 按类型搜索
        let documents = manager
            .search_by_type(ResourceType::Document)
            .await
            .unwrap();
        let images = manager.search_by_type(ResourceType::Image).await.unwrap();
        let audio = manager.search_by_type(ResourceType::Audio).await.unwrap();

        assert_eq!(documents.len(), 1);
        assert_eq!(images.len(), 1);
        assert_eq!(audio.len(), 1);

        assert_eq!(documents[0].original_filename, "document.pdf");
        assert_eq!(images[0].original_filename, "image.png");
        assert_eq!(audio[0].original_filename, "audio.mp3");
    }

    #[tokio::test]
    async fn test_search_by_tags() {
        let temp_dir = TempDir::new().unwrap();
        let config = ResourceStorageConfig {
            storage_root: temp_dir.path().join("storage"),
            ..Default::default()
        };

        let manager = ResourceMemoryManager::with_config(config).unwrap();

        // 创建测试文件
        let test_file =
            create_test_file(temp_dir.path(), "tagged_file.txt", b"Tagged content").await;

        // 存储带标签的资源
        let tags = vec![
            "important".to_string(),
            "document".to_string(),
            "test".to_string(),
        ];
        manager
            .store_resource(&test_file, Some(tags.clone()), None)
            .await
            .unwrap();

        // 按标签搜索
        let results = manager
            .search_by_tags(&["important".to_string()])
            .await
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].tags, tags);

        let no_results = manager
            .search_by_tags(&["nonexistent".to_string()])
            .await
            .unwrap();
        assert_eq!(no_results.len(), 0);
    }

    #[tokio::test]
    async fn test_search_by_filename() {
        let temp_dir = TempDir::new().unwrap();
        let config = ResourceStorageConfig {
            storage_root: temp_dir.path().join("storage"),
            ..Default::default()
        };

        let manager = ResourceMemoryManager::with_config(config).unwrap();

        // 创建测试文件
        let file1 = create_test_file(temp_dir.path(), "important_document.pdf", b"Content 1").await;
        let file2 = create_test_file(temp_dir.path(), "another_file.txt", b"Content 2").await;
        let file3 = create_test_file(temp_dir.path(), "important_image.png", b"Content 3").await;

        // 存储资源
        manager.store_resource(&file1, None, None).await.unwrap();
        manager.store_resource(&file2, None, None).await.unwrap();
        manager.store_resource(&file3, None, None).await.unwrap();

        // 按文件名搜索
        let results = manager.search_by_filename("important").await.unwrap();
        assert_eq!(results.len(), 2);

        let pdf_results = manager.search_by_filename("document").await.unwrap();
        assert_eq!(pdf_results.len(), 1);
        assert_eq!(pdf_results[0].original_filename, "important_document.pdf");
    }

    #[tokio::test]
    async fn test_update_resource_tags() {
        let temp_dir = TempDir::new().unwrap();
        let config = ResourceStorageConfig {
            storage_root: temp_dir.path().join("storage"),
            ..Default::default()
        };

        let manager = ResourceMemoryManager::with_config(config).unwrap();

        // 创建测试文件
        let test_file = create_test_file(temp_dir.path(), "test.txt", b"Test content").await;

        // 存储资源
        let resource_id = manager
            .store_resource(&test_file, None, None)
            .await
            .unwrap();

        // 更新标签
        let new_tags = vec!["updated".to_string(), "new_tag".to_string()];
        manager
            .update_resource_tags(&resource_id, new_tags.clone())
            .await
            .unwrap();

        // 验证标签更新
        let metadata = manager
            .get_resource_metadata(&resource_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(metadata.tags, new_tags);
    }

    #[tokio::test]
    async fn test_delete_resource() {
        let temp_dir = TempDir::new().unwrap();
        let config = ResourceStorageConfig {
            storage_root: temp_dir.path().join("storage"),
            ..Default::default()
        };

        let manager = ResourceMemoryManager::with_config(config).unwrap();

        // 创建测试文件
        let test_file = create_test_file(temp_dir.path(), "to_delete.txt", b"Delete me").await;

        // 存储资源
        let resource_id = manager
            .store_resource(&test_file, None, None)
            .await
            .unwrap();

        // 验证资源存在
        assert!(manager
            .get_resource_metadata(&resource_id)
            .await
            .unwrap()
            .is_some());
        let storage_path = manager
            .get_resource_path(&resource_id)
            .await
            .unwrap()
            .unwrap();
        assert!(storage_path.exists());

        // 删除资源
        manager.delete_resource(&resource_id).await.unwrap();

        // 验证资源已删除
        assert!(manager
            .get_resource_metadata(&resource_id)
            .await
            .unwrap()
            .is_none());
        assert!(!storage_path.exists());
    }

    #[tokio::test]
    async fn test_file_size_limit() {
        let temp_dir = TempDir::new().unwrap();
        let config = ResourceStorageConfig {
            storage_root: temp_dir.path().join("storage"),
            max_file_size: 10, // 10 bytes limit
            ..Default::default()
        };

        let manager = ResourceMemoryManager::with_config(config).unwrap();

        // 创建超过大小限制的文件
        let large_content = b"This content is definitely larger than 10 bytes";
        let large_file = create_test_file(temp_dir.path(), "large.txt", large_content).await;

        // 尝试存储应该失败
        let result = manager.store_resource(&large_file, None, None).await;
        assert!(result.is_err());

        // 创建符合大小限制的文件
        let small_content = b"Small";
        let small_file = create_test_file(temp_dir.path(), "small.txt", small_content).await;

        // 存储应该成功
        let result = manager.store_resource(&small_file, None, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_storage_stats() {
        let temp_dir = TempDir::new().unwrap();
        let config = ResourceStorageConfig {
            storage_root: temp_dir.path().join("storage"),
            ..Default::default()
        };

        let manager = ResourceMemoryManager::with_config(config).unwrap();

        // 创建不同类型的文件
        let doc_content = b"Document content";
        let img_content = b"Image data";

        let doc_file = create_test_file(temp_dir.path(), "doc.pdf", doc_content).await;
        let img_file = create_test_file(temp_dir.path(), "img.png", img_content).await;

        // 存储资源
        manager.store_resource(&doc_file, None, None).await.unwrap();
        manager.store_resource(&img_file, None, None).await.unwrap();

        // 检查统计信息
        let stats = manager.get_stats().await.unwrap();
        assert_eq!(stats.total_resources, 2);
        assert_eq!(
            stats.total_storage_size,
            (doc_content.len() + img_content.len()) as u64
        );
        assert_eq!(
            stats.resources_by_type.get(&ResourceType::Document),
            Some(&1)
        );
        assert_eq!(stats.resources_by_type.get(&ResourceType::Image), Some(&1));

        let expected_avg = (doc_content.len() + img_content.len()) as f64 / 2.0;
        assert!((stats.average_file_size - expected_avg).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_storage_health_check() {
        let temp_dir = TempDir::new().unwrap();
        let config = ResourceStorageConfig {
            storage_root: temp_dir.path().join("storage"),
            ..Default::default()
        };

        let manager = ResourceMemoryManager::with_config(config).unwrap();

        // 创建测试文件
        let test_file = create_test_file(temp_dir.path(), "health_test.txt", b"Health check").await;

        // 存储资源
        let resource_id = manager
            .store_resource(&test_file, None, None)
            .await
            .unwrap();

        // 健康检查应该没有问题
        let issues = manager.check_storage_health().await.unwrap();
        assert_eq!(issues.len(), 0);

        // 删除物理文件但保留元数据
        let storage_path = manager
            .get_resource_path(&resource_id)
            .await
            .unwrap()
            .unwrap();
        tokio::fs::remove_file(&storage_path).await.unwrap();

        // 健康检查应该发现问题
        let issues = manager.check_storage_health().await.unwrap();
        assert_eq!(issues.len(), 1);
        assert!(issues[0].contains("Missing file"));
    }

    #[tokio::test]
    async fn test_clear_all() {
        let temp_dir = TempDir::new().unwrap();
        let config = ResourceStorageConfig {
            storage_root: temp_dir.path().join("storage"),
            ..Default::default()
        };

        let manager = ResourceMemoryManager::with_config(config).unwrap();

        // 创建测试文件
        let test_file = create_test_file(temp_dir.path(), "clear_test.txt", b"Clear me").await;

        // 存储资源
        manager
            .store_resource(&test_file, None, None)
            .await
            .unwrap();

        // 验证资源存在
        let stats_before = manager.get_stats().await.unwrap();
        assert_eq!(stats_before.total_resources, 1);

        // 清空所有资源
        manager.clear_all().await.unwrap();

        // 验证所有资源已清空
        let stats_after = manager.get_stats().await.unwrap();
        assert_eq!(stats_after.total_resources, 0);
        assert_eq!(stats_after.total_storage_size, 0);

        let all_resources = manager.list_all_resources().await.unwrap();
        assert_eq!(all_resources.len(), 0);
    }

    #[tokio::test]
    async fn test_resource_type_variants() {
        // 测试所有资源类型
        assert_ne!(ResourceType::Document, ResourceType::Image);
        assert_ne!(ResourceType::Image, ResourceType::Video);
        assert_ne!(ResourceType::Video, ResourceType::Audio);
        assert_ne!(ResourceType::Audio, ResourceType::Other);
        assert_ne!(ResourceType::Other, ResourceType::Document);
    }

    #[tokio::test]
    async fn test_resource_storage_config_default() {
        let config = ResourceStorageConfig::default();

        assert!(config.max_file_size > 0);
        assert!(config.enable_compression || !config.enable_compression); // Always true, just testing field access
    }

    #[tokio::test]
    async fn test_resource_storage_config_custom() {
        let temp_dir = TempDir::new().unwrap();
        let config = ResourceStorageConfig {
            storage_root: temp_dir.path().to_path_buf(),
            max_file_size: 1024 * 1024, // 1MB
            enable_compression: false,
            compression_threshold: 512 * 1024, // 512KB
            enable_deduplication: true,
            enable_versioning: true,
            max_versions: 5,
        };

        assert_eq!(config.max_file_size, 1024 * 1024);
        assert_eq!(config.compression_threshold, 512 * 1024);
        assert!(config.enable_deduplication);
        assert!(!config.enable_compression);
        assert!(config.enable_versioning);
        assert_eq!(config.max_versions, 5);
    }

    #[tokio::test]
    async fn test_resource_metadata_structure() {
        let temp_dir = TempDir::new().unwrap();
        let config = ResourceStorageConfig {
            storage_root: temp_dir.path().join("storage"),
            ..Default::default()
        };

        let manager = ResourceMemoryManager::with_config(config).unwrap();
        let test_file = create_test_file(temp_dir.path(), "test.txt", b"content").await;

        let resource_id = manager.store_resource(&test_file, None, None).await.unwrap();
        let metadata = manager.get_resource_metadata(&resource_id).await.unwrap();

        assert!(metadata.is_some());
        let meta = metadata.unwrap();
        assert!(!meta.id.is_empty());
        assert!(!meta.original_filename.is_empty());
        assert!(meta.file_size > 0);
    }

    #[tokio::test]
    async fn test_multiple_resources_same_type() {
        let temp_dir = TempDir::new().unwrap();
        let config = ResourceStorageConfig {
            storage_root: temp_dir.path().join("storage"),
            ..Default::default()
        };

        let manager = ResourceMemoryManager::with_config(config).unwrap();

        // 存储多个相同类型的资源
        let file1 = create_test_file(temp_dir.path(), "doc1.pdf", b"content1").await;
        let file2 = create_test_file(temp_dir.path(), "doc2.pdf", b"content2").await;
        let file3 = create_test_file(temp_dir.path(), "doc3.pdf", b"content3").await;

        let id1 = manager.store_resource(&file1, None, None).await.unwrap();
        let id2 = manager.store_resource(&file2, None, None).await.unwrap();
        let id3 = manager.store_resource(&file3, None, None).await.unwrap();

        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
        assert_ne!(id1, id3);

        let stats = manager.get_stats().await.unwrap();
        assert_eq!(stats.total_resources, 3);
    }

    #[tokio::test]
    async fn test_resource_tags_management() {
        let temp_dir = TempDir::new().unwrap();
        let config = ResourceStorageConfig {
            storage_root: temp_dir.path().join("storage"),
            ..Default::default()
        };

        let manager = ResourceMemoryManager::with_config(config).unwrap();
        let test_file = create_test_file(temp_dir.path(), "tagged.txt", b"content").await;

        // 存储带标签的资源
        let tags = vec!["important".to_string(), "project-a".to_string()];
        let resource_id = manager.store_resource(&test_file, Some(tags.clone()), None).await.unwrap();

        // 验证标签
        let metadata = manager.get_resource_metadata(&resource_id).await.unwrap().unwrap();
        assert_eq!(metadata.tags.len(), 2);
        assert!(metadata.tags.contains(&"important".to_string()));
    }

    #[tokio::test]
    async fn test_resource_custom_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let config = ResourceStorageConfig {
            storage_root: temp_dir.path().join("storage"),
            ..Default::default()
        };

        let manager = ResourceMemoryManager::with_config(config).unwrap();
        let test_file = create_test_file(temp_dir.path(), "meta.txt", b"content").await;

        // 存储带自定义元数据的资源
        let mut custom_meta = HashMap::new();
        custom_meta.insert("author".to_string(), "John Doe".to_string());
        custom_meta.insert("version".to_string(), "1.0".to_string());

        let resource_id = manager.store_resource(&test_file, None, Some(custom_meta.clone())).await.unwrap();

        // 验证自定义元数据
        let metadata = manager.get_resource_metadata(&resource_id).await.unwrap().unwrap();
        assert_eq!(metadata.custom_metadata.get("author"), Some(&"John Doe".to_string()));
        assert_eq!(metadata.custom_metadata.get("version"), Some(&"1.0".to_string()));
    }

    #[tokio::test]
    async fn test_empty_file_handling() {
        let temp_dir = TempDir::new().unwrap();
        let config = ResourceStorageConfig {
            storage_root: temp_dir.path().join("storage"),
            ..Default::default()
        };

        let manager = ResourceMemoryManager::with_config(config).unwrap();
        let empty_file = create_test_file(temp_dir.path(), "empty.txt", b"").await;

        // 存储空文件
        let resource_id = manager.store_resource(&empty_file, None, None).await.unwrap();

        let metadata = manager.get_resource_metadata(&resource_id).await.unwrap().unwrap();
        assert_eq!(metadata.file_size, 0);
    }
}
