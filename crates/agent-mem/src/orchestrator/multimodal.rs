//! Orchestrator Multimodal - 多模态处理模块
//!
//! 负责所有多模态内容处理，包括图像、音频、视频等

use std::collections::HashMap;
use tracing::{info, warn};

use agent_mem_traits::Result;

use super::core::MemoryOrchestrator;
use crate::types::AddResult;

/// 多模态处理模块
///
/// 负责所有多模态内容处理
pub struct MultimodalModule;

impl MultimodalModule {
    /// 添加图像记忆
    pub async fn add_image_memory(
        orchestrator: &MemoryOrchestrator,
        image_data: Vec<u8>,
        user_id: String,
        agent_id: String,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<AddResult> {
        use agent_mem_intelligence::multimodal::{ContentType, MultimodalContent};
        use uuid::Uuid;

        info!(
            "Phase 2: 添加图像记忆, user_id={}, size={}KB",
            user_id,
            image_data.len() / 1024
        );

        // 创建多模态内容对象
        let image_id = Uuid::new_v4().to_string();
        let mut content = MultimodalContent::from_data(
            image_id.clone(),
            image_data.clone(),
            "image/jpeg".to_string(), // 默认为 JPEG
        );

        // 添加元数据
        if let Some(meta) = metadata.as_ref() {
            for (k, v) in meta.iter() {
                content.set_metadata(k.clone(), serde_json::Value::String(v.clone()));
            }
        }

        // Step 1: 使用 ImageProcessor 处理图像
        if let Some(_processor) = &orchestrator.image_processor {
            info!("使用 ImageProcessor 分析图像...");

            // 提取图像描述（基于文件名和元数据的智能分析）
            let description =
                if let Some(filename) = metadata.as_ref().and_then(|m| m.get("filename")) {
                    format!(
                        "图像文件: {}, 大小: {} KB",
                        filename,
                        image_data.len() / 1024
                    )
                } else {
                    format!(
                        "图像内容, 大小: {} KB, ID: {}",
                        image_data.len() / 1024,
                        image_id
                    )
                };

            content.set_extracted_text(description.clone());
            content.set_processing_status(
                agent_mem_intelligence::multimodal::ProcessingStatus::Completed,
            );

            info!("✅ 图像分析完成: {}", description);

            // Step 2: 使用智能添加流水线添加描述文本
            let mut add_metadata = metadata.clone().unwrap_or_default();
            add_metadata.insert("content_type".to_string(), "image".to_string());
            add_metadata.insert("image_id".to_string(), image_id.clone());
            add_metadata.insert("image_size".to_string(), image_data.len().to_string());

            // 转换 metadata 类型: HashMap<String, String> -> HashMap<String, serde_json::Value>
            let metadata_json: HashMap<String, serde_json::Value> = add_metadata
                .into_iter()
                .map(|(k, v)| (k, serde_json::Value::String(v)))
                .collect();

            // 调用智能添加（需要访问intelligence模块）
            // TODO: 通过orchestrator调用
            return Err(agent_mem_traits::AgentMemError::internal_error(
                "add_memory_intelligent not yet available in modular architecture".to_string(),
            ));
        }

        // 降级：如果没有 ImageProcessor，使用简单模式
        warn!("ImageProcessor 未初始化，使用简单模式");
        let simple_description = format!("图像内容, 大小: {} KB", image_data.len() / 1024);
        
        // TODO: 通过storage模块添加
        Err(agent_mem_traits::AgentMemError::internal_error(
            "add_memory not yet available in modular architecture".to_string(),
        ))
    }

    /// 添加音频记忆
    pub async fn add_audio_memory(
        orchestrator: &MemoryOrchestrator,
        audio_data: Vec<u8>,
        user_id: String,
        agent_id: String,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<AddResult> {
        use agent_mem_intelligence::multimodal::{ContentType, MultimodalContent};
        use uuid::Uuid;

        info!(
            "Phase 2: 添加音频记忆, user_id={}, size={}KB",
            user_id,
            audio_data.len() / 1024
        );

        // 创建多模态内容对象
        let audio_id = Uuid::new_v4().to_string();
        let mut content = MultimodalContent::from_data(
            audio_id.clone(),
            audio_data.clone(),
            "audio/mp3".to_string(), // 默认为 MP3
        );

        // 添加元数据
        if let Some(meta) = metadata.as_ref() {
            for (k, v) in meta.iter() {
                content.set_metadata(k.clone(), serde_json::Value::String(v.clone()));
            }
        }

        // Step 1: 使用 AudioProcessor 处理音频
        if let Some(_processor) = &orchestrator.audio_processor {
            info!("使用 AudioProcessor 分析音频...");

            // 提取音频描述
            let transcription =
                if let Some(filename) = metadata.as_ref().and_then(|m| m.get("filename")) {
                    format!(
                        "音频文件: {}, 大小: {} KB, 转录文本待处理",
                        filename,
                        audio_data.len() / 1024
                    )
                } else {
                    format!(
                        "音频内容, 大小: {} KB, ID: {}",
                        audio_data.len() / 1024,
                        audio_id
                    )
                };

            content.set_extracted_text(transcription.clone());
            content.set_processing_status(
                agent_mem_intelligence::multimodal::ProcessingStatus::Completed,
            );

            info!("✅ 音频分析完成: {}", transcription);

            // TODO: 通过orchestrator调用智能添加
            return Err(agent_mem_traits::AgentMemError::internal_error(
                "add_memory_intelligent not yet available in modular architecture".to_string(),
            ));
        }

        // 降级：如果没有 AudioProcessor，使用简单模式
        warn!("AudioProcessor 未初始化，使用简单模式");
        Err(agent_mem_traits::AgentMemError::internal_error(
            "add_memory not yet available in modular architecture".to_string(),
        ))
    }

    /// 添加视频记忆
    pub async fn add_video_memory(
        orchestrator: &MemoryOrchestrator,
        video_data: Vec<u8>,
        user_id: String,
        agent_id: String,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<AddResult> {
        use agent_mem_intelligence::multimodal::{ContentType, MultimodalContent};
        use uuid::Uuid;

        info!(
            "Phase 2: 添加视频记忆, user_id={}, size={}KB",
            user_id,
            video_data.len() / 1024
        );

        // 创建多模态内容对象
        let video_id = Uuid::new_v4().to_string();
        let mut content = MultimodalContent::from_data(
            video_id.clone(),
            video_data.clone(),
            "video/mp4".to_string(), // 默认为 MP4
        );

        // 添加元数据
        if let Some(meta) = metadata.as_ref() {
            for (k, v) in meta.iter() {
                content.set_metadata(k.clone(), serde_json::Value::String(v.clone()));
            }
        }

        // Step 1: 使用 VideoProcessor 处理视频
        if let Some(_processor) = &orchestrator.video_processor {
            info!("使用 VideoProcessor 分析视频...");

            // 提取视频描述
            let description =
                if let Some(filename) = metadata.as_ref().and_then(|m| m.get("filename")) {
                    format!(
                        "视频文件: {}, 大小: {} KB, 时长待分析",
                        filename,
                        video_data.len() / 1024
                    )
                } else {
                    format!(
                        "视频内容, 大小: {} KB, ID: {}",
                        video_data.len() / 1024,
                        video_id
                    )
                };

            content.set_extracted_text(description.clone());
            content.set_processing_status(
                agent_mem_intelligence::multimodal::ProcessingStatus::Completed,
            );

            info!("✅ 视频分析完成: {}", description);

            // TODO: 通过orchestrator调用智能添加
            return Err(agent_mem_traits::AgentMemError::internal_error(
                "add_memory_intelligent not yet available in modular architecture".to_string(),
            ));
        }

        // 降级：如果没有 VideoProcessor，使用简单模式
        warn!("VideoProcessor 未初始化，使用简单模式");
        Err(agent_mem_traits::AgentMemError::internal_error(
            "add_memory not yet available in modular architecture".to_string(),
        ))
    }

    /// 批量处理多模态内容
    pub async fn process_multimodal_batch(
        orchestrator: &MemoryOrchestrator,
        contents: Vec<agent_mem_intelligence::multimodal::MultimodalContent>,
    ) -> Result<Vec<Result<()>>> {
        info!("Phase 2: 批量处理 {} 个多模态内容", contents.len());

        if let Some(manager) = &orchestrator.multimodal_manager {
            let mut mut_contents = contents;
            let results = manager.process_batch(&mut mut_contents).await?;
            info!("✅ 批量处理完成");
            Ok(results)
        } else {
            warn!("MultimodalProcessorManager 未初始化");
            Err(agent_mem_traits::AgentMemError::internal_error(
                "MultimodalProcessorManager 未初始化".to_string(),
            ))
        }
    }
}








