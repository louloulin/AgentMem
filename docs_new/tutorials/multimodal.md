# 多模态处理教程

学习如何在 AgentMem 中处理文本、图像、音频、视频等多种类型的内容。

## 目录

- [多模态概述](#多模态概述)
- [图像处理](#图像处理)
- [音频处理](#音频处理)
- [视频处理](#视频处理)
- [文档处理](#文档处理)
- [跨模态搜索](#跨模态搜索)
- [常见场景](#常见场景)

## 多模态概述

AgentMem 支持 Memory V4 架构，可以处理多种内容类型：

### 支持的内容类型

| 类型 | 说明 | 示例 |
|-----|------|-----|
| Text | 纯文本 | 用户消息、文档内容 |
| Image | 图像 | 照片、截图、图表 |
| Audio | 音频 | 语音消息、录音 |
| Video | 视频 | 视频文件、直播流 |
| Structured | 结构化数据 | JSON、XML、CSV |
| Binary | 二进制数据 | 文件、压缩包 |

### 内容存储策略

```
┌─────────────────────────────────────┐
│         Application Layer           │
└──────────────┬──────────────────────┘
               │
┌──────────────▼──────────────────────┐
│         Multimodal Processor        │
│  • 内容识别  • 格式转换  • 特征提取  │
└──┬───┬───┬───┬───┬───┬───┬───┬───┬──┘
   │   │   │   │   │   │   │   │   │
┌──▼─┐▼───▼───▼─┐▼───▼───▼───▼───▼──┐
│  Metadata Store │  Content Store   │
│  (描述信息)     │  (实际内容)       │
│  • PostgreSQL   │  • S3            │
│  • Redis        │  • 本地文件      │
└─────────────────┴──────────────────┘
```

## 图像处理

### 添加图像记忆

```rust
use agentmem::{Memory, MemoryType, Content, Metadata};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let memory = Memory::new().await?;

    // 方式 1: 存储图像 URL
    let mut metadata = Metadata::new();
    metadata.insert("content_type", "image");
    metadata.insert("url", "https://example.com/photo.jpg");
    metadata.insert("caption", "用户上传的照片");

    memory.add(
        "用户上传了一张照片",
        None,
        MemoryType::RESOURCE
    ).await?;

    // 方式 2: 存储图像 Base64
    let base64_image = "data:image/jpeg;base64,/9j/4AAQSkZJRg...";

    let image_content = Content::builder()
        .data(base64_image)
        .content_type("image/jpeg")
        .encoding("base64")
        .build();

    memory.add_content(image_content).await?;

    // 方式 3: 使用 V4 API（推荐）
    use agentmem::MemoryV4;

    let memory_v4 = MemoryV4::builder()
        .content(Content::Text("用户上传的照片".to_string()))
        .attribute("type", "image")
        .attribute("url", "https://example.com/photo.jpg")
        .attribute("width", 1920)
        .attribute("height", 1080)
        .attribute("format", "jpeg")
        .build();

    memory.add_v4(memory_v4).await?;

    Ok(())
}
```

### 图像特征提取

```rust
use agentmem::{Memory, ImageFeatures};

// 自动提取图像特征
let features = memory.extract_image_features(
    "https://example.com/photo.jpg"
).await?;

println!("图像特征:");
println!("  主要颜色: {:?}", features.dominant_colors);
println!("  对象: {:?}", features.objects);
println!("  场景: {:?}", features.scene);
println!("  文本: {:?}", features.text);
println!("  人脸数: {}", features.face_count);

// 将特征添加到记忆
let mut metadata = Metadata::new();
metadata.insert("image_features", serde_json::to_string(&features)?);

memory.add(
    "用户上传的照片包含一只猫",
    None,
    MemoryType::SEMANTIC
).await?;
```

### 图像搜索

```rust
// 基于文本描述搜索图像
let results = memory.search(
    SearchOptions::builder()
        .query("猫")
        .content_type(Some("image"))
        .build()
).await?;

// 基于图像特征搜索（相似图像）
let similar_images = memory.search_similar_images(
    "https://example.com/query.jpg",
    SimilarityOptions {
        threshold: 0.8,
        limit: 10,
    }
).await?;
```

## 音频处理

### 添加音频记忆

```rust
use agentmem::{Memory, MemoryType};

// 存储音频 URL
let mut metadata = Metadata::new();
metadata.insert("content_type", "audio");
metadata.insert("url", "https://example.com/voice.mp3");
metadata.insert("duration", 45);  // 秒
metadata.insert("format", "mp3");

memory.add(
    "用户发送了语音消息",
    None,
    MemoryType::RESOURCE
).await?;

// 存储 Base64 编码的音频
let audio_base64 = "data:audio/mp3;base64,//uQxAAAAAAAA...";

memory.add(
    &format!("语音消息: {}", audio_base64),
    None,
    MemoryType::RESOURCE
).await?;
```

### 语音转文字

```rust
// 自动转录音频
let transcription = memory.transcribe_audio(
    "https://example.com/voice.mp3"
).await?;

println!("转录结果: {}", transcription.text);
println!("语言: {}", transcription.language);
println!("置信度: {:.2}", transcription.confidence);

// 将转录结果添加到记忆
memory.add(
    &format!("用户说: {}", transcription.text),
    None,
    MemoryType::EPISODIC
).await?;
```

### 音频特征提取

```rust
use agentmem::AudioFeatures;

// 提取音频特征
let features = memory.extract_audio_features(
    "https://example.com/voice.mp3"
).await?;

println!("音频特征:");
println!("  时长: {}秒", features.duration);
println!("  采样率: {}Hz", features.sample_rate);
println!("  语言: {:?}", features.language);
println!("  情感: {:?}", features.emotion);
println!("  音量: {:.2}", features.volume);
println!("  语速: {:.2} 字/分钟", features.speech_rate);
```

## 视频处理

### 添加视频记忆

```rust
use agentmem::Memory;

// 存储视频 URL
let mut metadata = Metadata::new();
metadata.insert("content_type", "video");
metadata.insert("url", "https://example.com/video.mp4");
metadata.insert("duration", 120);  // 秒
metadata.insert("format", "mp4");
metadata.insert("resolution", "1920x1080");

memory.add(
    "用户分享了视频",
    None,
    MemoryType::RESOURCE
).await?;
```

### 视频关键帧提取

```rust
// 提取关键帧
let keyframes = memory.extract_video_keyframes(
    "https://example.com/video.mp4",
    KeyframeOptions {
        interval: 10,  // 每 10 秒提取一帧
        count: 12,     // 最多提取 12 帧
    }
).await?;

println!("提取了 {} 个关键帧", keyframes.len());

for (i, frame) in keyframes.iter().enumerate() {
    println!("帧 {}: 时间={}秒, URL={}",
        i + 1,
        frame.timestamp,
        frame.url
    );
}
```

### 视频场景检测

```rust
use agentmem::SceneDetection;

// 检测视频场景
let scenes = memory.detect_video_scenes(
    "https://example.com/video.mp4"
).await?;

println!("检测到 {} 个场景:", scenes.len());

for (i, scene) in scenes.iter().enumerate() {
    println!("场景 {}: {}秒 - {}秒",
        i + 1,
        scene.start_time,
        scene.end_time
    );
    println!("  描述: {}", scene.description);
    println!("  关键帧: {}", scene.keyframe_url);
}
```

## 文档处理

### 处理 PDF 文档

```rust
use agentmem::{Memory, MemoryType};

// 上传 PDF
let pdf_url = "https://example.com/document.pdf";

// 提取文本内容
let text = memory.extract_pdf_text(pdf_url).await?;

println!("PDF 内容预览:");
println!("{}", text.chars().take(200).collect::<String>());

// 添加到记忆
memory.add(
    &format!("PDF文档内容: {}", text),
    None,
    MemoryType::SEMANTIC
).await?;

// 提取结构化信息
let structure = memory.extract_pdf_structure(pdf_url).await?;

println!("文档结构:");
println!("  标题: {}", structure.title);
println!("  章节: {} 个", structure.chapters.len());
println!("  表格: {} 个", structure.tables.len());
println!("  图片: {} 个", structure.images.len());
```

### 处理 Word 文档

```rust
// 提取 Word 文档内容
let docx_url = "https://example.com/document.docx";

let content = memory.extract_docx_content(docx_url).await?;

println!("文档内容:");
println!("{}", content.text);
println!("样式: {} 个", content.styles.len());
```

### 处理 Excel 文件

```rust
// 提取 Excel 数据
let xlsx_url = "https://example.com/data.xlsx";

let data = memory.extract_xlsx_data(xlsx_url).await?;

println!("工作表数量: {}", data.sheets.len());

for sheet in data.sheets {
    println!("工作表: {}", sheet.name);
    println!("  行数: {}", sheet.row_count);
    println!("  列数: {}", sheet.column_count);

    // 转换为 JSON
    let json = serde_json::to_string(&sheet.data)?;
    memory.add(json, None, MemoryType::SEMANTIC).await?;
}
```

## 跨模态搜索

### 文本搜图像

```rust
// 用文本描述搜索图像
let results = memory.search(
    SearchOptions::builder()
        .query("猫在沙发上")
        .content_types(vec!["image"])
        .build()
).await?;

for result in results {
    println!("找到图像: {}", result.memory.metadata.get("url").unwrap());
    println!("相似度: {:.2}", result.score);
}
```

### 图像搜文本

```rust
// 用图像搜索相关文本
let image_url = "https://example.com/photo.jpg";

let results = memory.search_by_image(
    image_url,
    SearchOptions {
        content_types: vec!["text"],
        limit: 10,
        ..Default::default()
    }
).await?;

for result in results {
    println!("相关文本: {}", result.memory.content);
}
```

### 音频搜索

```rust
// 基于音频内容搜索
let audio_url = "https://example.com/voice.mp3";

// 转录并搜索
let transcription = memory.transcribe_audio(audio_url).await?;

let results = memory.search(
    SearchOptions::builder()
        .query(&transcription.text)
        .build()
).await?;
```

### 多模态混合搜索

```rust
// 同时搜索多种内容类型
let results = memory.search(
    SearchOptions::builder()
        .query("产品设计")
        .content_types(vec!["text", "image", "video"])
        .multimodal_fusion(MultimodalFusion::Weighted {
            text: 0.5,
            image: 0.3,
            video: 0.2,
        })
        .build()
).await?;

// 按内容类型分组
let mut by_type: std::collections::HashMap<String, Vec<_>> =
    std::collections::HashMap::new();

for result in results {
    let content_type = result.memory.metadata
        .get("content_type")
        .unwrap_or(&"unknown".to_string())
        .clone();

    by_type.entry(content_type)
        .or_insert_with(Vec::new)
        .push(result);
}

println!("搜索结果:");
for (content_type, results) in by_type {
    println!("{} ({} 条)", content_type, results.len());
    for result in results.iter().take(3) {
        println!("  - {} ({:.2})",
            result.memory.content,
            result.score
        );
    }
}
```

## 常见场景

### 场景 1: 智能相册

```rust
async fn smart_photo_album() -> Result<(), Box<dyn std::error::Error>> {
    let memory = Memory::new().await?;

    // 上传照片
    let photos = vec![
        "https://example.com/photo1.jpg",
        "https://example.com/photo2.jpg",
        "https://example.com/photo3.jpg",
    ];

    for photo in photos {
        // 提取特征
        let features = memory.extract_image_features(photo).await?;

        // 添加到记忆
        memory.add(
            &format!("照片: {}", photo),
            None,
            MemoryType::SEMANTIC
        ).await?;
    }

    // 搜索"有猫的照片"
    let cat_photos = memory.search(
        SearchOptions::builder()
            .query("猫")
            .content_type(Some("image"))
            .build()
    ).await?;

    println!("找到 {} 张有猫的照片:", cat_photos.len());

    Ok(())
}
```

### 场景 2: 语音助手

```rust
async fn voice_assistant() -> Result<(), Box<dyn std::error::Error>> {
    let memory = Memory::new().await?;

    // 接收语音消息
    let voice_url = "https://example.com/voice.mp3";

    // 转录为文字
    let transcription = memory.transcribe_audio(voice_url).await?;

    println!("用户说: {}", transcription.text);

    // 添加到对话历史
    memory.add(
        &format!("用户: {}", transcription.text),
        Some(session_id),
        MemoryType::EPISODIC
    ).await?;

    // 搜索相关上下文
    let context = memory.search(&transcription.text).await?;

    // 生成回复（这里简化）
    let reply = "好的，我明白了";

    // 转换为语音（可选）
    let audio_reply = memory.text_to_speech(reply).await?;

    // 保存回复
    memory.add(
        &format!("助手: {}", reply),
        Some(session_id),
        MemoryType::EPISODIC
    ).await?;

    Ok(())
}
```

### 场景 3: 文档问答系统

```rust
async fn document_qa() -> Result<(), Box<dyn std::error::Error>> {
    let memory = Memory::new().await?;

    // 上传文档
    let pdf_url = "https://example.com/manual.pdf";

    // 提取内容
    let text = memory.extract_pdf_text(pdf_url).await?;

    // 分块添加到记忆
    let chunks = text.split_inclusive('\n')
        .collect::<Vec<_>>()
        .chunks(100)  // 每 100 行一块
        .map(|chunk| chunk.concat())
        .collect::<Vec<_>>();

    for (i, chunk) in chunks.iter().enumerate() {
        memory.add(
            chunk,
            None,
            MemoryType::SEMANTIC
        ).await?;
    }

    // 回答问题
    let question = "如何重置密码？";

    let answers = memory.search(question).await?;

    println!("找到相关内容:");
    for answer in answers.iter().take(3) {
        println!("- {}", answer.memory.content);
    }

    Ok(())
}
```

### 场景 4: 视频内容分析

```rust
async fn video_analysis() -> Result<(), Box<dyn std::error::Error>> {
    let memory = Memory::new().await?;

    // 上传视频
    let video_url = "https://example.com/tutorial.mp4";

    // 提取关键帧
    let keyframes = memory.extract_video_keyframes(
        video_url,
        KeyframeOptions {
            interval: 30,
            count: 20,
        }
    ).await?;

    // 分析每个关键帧
    for frame in keyframes {
        let features = memory.extract_image_features(&frame.url).await?;

        memory.add(
            &format!("视频关键帧 ({}秒): {:?}",
                frame.timestamp,
                features.objects
            ),
            None,
            MemoryType::SEMANTIC
        ).await?;
    }

    // 搜索特定内容
    let results = memory.search(
        SearchOptions::builder()
            .query("教程中提到API的地方")
            .content_type(Some("video_keyframe"))
            .build()
    ).await?;

    Ok(())
}
```

## 性能优化

### 1. 异步处理

```rust
// 对于大型文件，使用异步处理
let handle = tokio::spawn(async move {
    memory.extract_pdf_text(large_pdf_url).await
});

// 继续其他操作
// ...

let result = handle.await??;
```

### 2. 内容分块

```rust
// 将大文件分块处理
let chunks = split_into_chunks(large_file, 1024 * 1024);  // 1MB 块

for chunk in chunks {
    memory.add(chunk, None, MemoryType::SEMANTIC).await?;
}
```

### 3. 缓存特征

```rust
// 缓存已提取的特征
let memory = Memory::builder()
    .feature_cache_enabled(true)
    .feature_cache_ttl(3600)  // 1 小时
    .build()
    .await?;
```

## 最佳实践

1. **选择合适的内容存储方式**: URL > Base64 > 文件
2. **提取关键特征**: 减少存储开销
3. **使用异步处理**: 避免阻塞主线程
4. **压缩大型内容**: 节省存储空间
5. **定期清理**: 删除不再需要的内容

## 下一步

- [插件开发](plugins.md) - 扩展 AgentMem 功能
- [生产部署](production.md) - 部署到生产环境
