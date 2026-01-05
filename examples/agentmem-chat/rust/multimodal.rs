//! AgentMem 多模态处理示例
//!
//! 这个示例演示了如何处理多种类型的数据：
//! - 图像记忆
//! - 音频记忆
//! - 文本记忆
//! - 跨模态搜索
//!
//! # 运行方式
//!
//! ```bash
//! export OPENAI_API_KEY=sk-...
//! cargo run --example multimodal
//! ```
//!
//! # 预期输出
//!
//! ```text
//! 🎨 AgentMem 多模态处理示例
//!
//! ✅ 初始化完成
//!
//! 📸 步骤 1: 图像记忆
//!    添加图像描述: "一张日落的海滩照片"
//!    ✅ 图像记忆已保存
//!
//! 🎵 步骤 2: 音频记忆
//!    添加音频转录: "会议讨论了项目进度"
//!    ✅ 音频记忆已保存
//!
//! 📝 步骤 3: 文本记忆
//!    添加笔记: "项目截止日期是下周五"
//!    ✅ 文本记忆已保存
//!
//! 🔍 步骤 4: 跨模态搜索
//!    搜索: "项目"
//!    ✅ 找到 2 条相关记忆:
//!      1. 会议讨论了项目进度 (音频)
//!      2. 项目截止日期是下周五 (文本)
//!
//! 🎉 完成！
//! ```

use agent_mem::{GetAllOptions, Memory};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 多模态内容类型
#[derive(Debug, Clone, Serialize, Deserialize)]
enum ModalType {
    Text,
    Image,
    Audio,
    Video,
}

/// 多模态记忆
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MultimodalMemory {
    content: String,
    modal_type: ModalType,
    metadata: HashMap<String, String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎨 AgentMem 多模态处理示例\n");
    println!("这个示例演示了:");
    println!("  1. 图像记忆（通过文本描述）");
    println!("  2. 音频记忆（通过转录文本）");
    println!("  3. 文本记忆");
    println!("  4. 跨模态搜索");
    println!();

    // 初始化
    let mem = Memory::new().await?;
    println!("✅ 初始化完成\n");

    // ============================================
    // 步骤 1: 图像记忆
    // ============================================
    println!("📸 步骤 1: 图像记忆");
    println!("---");

    let image_memories = vec![
        "一张日落的海滩照片，有橙色的天空",
        "城市的夜景，灯光璀璨",
        "一只猫在阳光下睡觉",
    ];

    for desc in image_memories {
        println!("   添加图像描述: \"{}\"", desc);

        // 添加图像记忆（实际应用中，这里可以包含图像 URL 或 base64）
        let result = mem.add(desc).await?;

        println!("   ✅ 图像记忆已保存: {}", result.id);
        println!();
    }

    // ============================================
    // 步骤 2: 音频记忆
    // ============================================
    println!("🎵 步骤 2: 音频记忆");
    println!("---");

    let audio_memories = vec![
        "会议讨论了 Q4 的项目进度和目标",
        "电话留言: 明天下午三点开会",
        "播客摘要: 讨论了 AI 的未来发展趋势",
    ];

    for transcription in audio_memories {
        println!("   添加音频转录: \"{}\"", transcription);

        // 添加音频记忆（实际应用中，这里可以包含音频 URL）
        let result = mem.add(transcription).await?;

        println!("   ✅ 音频记忆已保存: {}", result.id);
        println!();
    }

    // ============================================
    // 步骤 3: 文本记忆
    // ============================================
    println!("📝 步骤 3: 文本记忆");
    println!("---");

    let text_memories = vec![
        "项目截止日期是下周五，需要完成所有功能",
        "购物清单: 牛奶、面包、鸡蛋、水果",
        "笔记: 学习 Rust 的所有权概念",
    ];

    for text in text_memories {
        println!("   添加笔记: \"{}\"", text);

        let result = mem.add(text).await?;

        println!("   ✅ 文本记忆已保存: {}", result.id);
        println!();
    }

    // ============================================
    // 步骤 4: 跨模态搜索
    // ============================================
    println!("🔍 步骤 4: 跨模态搜索");
    println!("---");

    let searches = vec![
        ("项目", "搜索与项目相关的所有内容"),
        ("会议", "搜索会议和讨论相关内容"),
        ("学习", "搜索学习和教育相关内容"),
    ];

    for (query, description) in searches {
        println!("   搜索: \"{}\" ({})", query, description);

        let results = mem.search(query).await?;

        println!("   ✅ 找到 {} 条相关记忆:", results.len());

        for (i, result) in results.iter().take(3).enumerate() {
            let score = result.score.unwrap_or(0.0);
            println!("      {}. {} (相似度: {:.2})", i + 1, result.content, score);
        }

        println!();
    }

    // ============================================
    // 步骤 5: 多模态分类（模拟）
    // ============================================
    println!("📊 步骤 5: 多模态分类");
    println!("---");

    let all_memories = mem.get_all(GetAllOptions::default()).await?;

    println!("   总记忆数: {}", all_memories.len());
    println!();

    // 简单分类统计（基于关键词）
    let mut image_count = 0;
    let mut audio_count = 0;
    let mut text_count = 0;

    for memory in &all_memories {
        if memory.content.contains("照片") || memory.content.contains("图像") {
            image_count += 1;
        } else if memory.content.contains("会议") || memory.content.contains("电话") || memory.content.contains("播客") {
            audio_count += 1;
        } else {
            text_count += 1;
        }
    }

    println!("   分类统计:");
    println!("   📸 图像相关: {} 条", image_count);
    println!("   🎵 音频相关: {} 条", audio_count);
    println!("   📝 文本相关: {} 条", text_count);
    println!();

    // ============================================
    // 完成
    // ============================================
    println!("🎉 完成！多模态处理演示完毕。\n");

    println!("💡 实际应用中的多模态处理:");
    println!("   1. 图像: 使用 Vision API 生成描述，然后存储");
    println!("   2. 音频: 使用 Speech-to-Text 转录，然后存储");
    println!("   3. 视频: 提取关键帧和音频，分别处理");
    println!("   4. 文档: 提取文本和图片，分别索引");
    println!();
    println!("🔍 跨模态搜索的优势:");
    println!("   - 可以用文本搜索图像内容");
    println!("   - 可以用文本搜索音频内容");
    println!("   - 统一的语义空间");

    Ok(())
}

// ============================================
// 高级示例: 实际图像处理
// ============================================
//
// 如果你想处理真实的图像，可以使用 Vision API:
//
// ```rust
// async fn process_image(image_url: &str) -> Result<String, Box<dyn std::error::Error>> {
//     // 使用 OpenAI Vision API 或其他视觉模型
//     let description = vision_analyzer.describe_image(image_url).await?;
//     Ok(description)
// }
//
// // 然后保存描述
// let description = process_image("https://example.com/image.jpg").await?;
// mem.add(&format!("图像描述: {}", description)).await?;
// ```
//
// ============================================
// 高级示例: 实际音频处理
// ============================================
//
// 如果你想处理真实的音频，可以使用 Whisper:
//
// ```rust
// async fn transcribe_audio(audio_url: &str) -> Result<String, Box<dyn std::error::Error>> {
//     // 使用 OpenAI Whisper 或其他 STT 模型
//     let transcription = whisper.transcribe(audio_url).await?;
//     Ok(transcription)
// }
//
// // 然后保存转录
// let transcription = transcribe_audio("https://example.com/audio.mp3").await?;
// mem.add(&format!("音频转录: {}", transcription)).await?;
// ```
//
// ============================================
// 高级示例: 元数据管理
// ============================================
//
// 你可以为多模态记忆添加元数据以便检索:
//
// ```rust
// use agent_mem::{Memory, Metadata};
//
// let mut metadata = Metadata::new();
// metadata.insert("type".to_string(), "image".to_string());
// metadata.insert("url".to_string(), "https://example.com/image.jpg".to_string());
// metadata.insert("timestamp".to_string(), "2025-01-01T00:00:00Z".to_string());
//
// mem.add_with_metadata("图像描述", metadata).await?;
// ```
//
// 然后可以根据元数据过滤:
//
// ```rust
// let results = mem.search_with_metadata(
//     "海滩",
//     |metadata| metadata.get("type") == Some(&"image".to_string())
// ).await?;
// ```
