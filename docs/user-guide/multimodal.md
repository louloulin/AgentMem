# 多模态功能使用指南

> **状态**: ✅ 完整实现（14个模块）  
> **位置**: `crates/agent-mem-intelligence/src/multimodal/`  
> **验证**: 2025-10-24 源码深度分析

---

## 概述

AgentMem 提供了完整的多模态支持，包括图像、音频、视频处理，以及跨模态检索能力。这使得AI代理能够处理和记忆多种类型的媒体内容。

### 核心特性

- ✅ **图像处理**: 图像分析、描述生成、视觉搜索
- ✅ **音频处理**: 语音转文本、音频分析
- ✅ **视频处理**: 视频分析、帧提取、场景理解
- ✅ **跨模态检索**: 文本搜图、图搜文、语音搜索
- ✅ **AI模型集成**: OpenAI Vision, Whisper, 自定义模型

### 模块列表

| 模块 | 功能 | 状态 |
|------|------|------|
| `image.rs` | 图像处理核心 | ✅ |
| `real_image.rs` | 实际图像实现 | ✅ |
| `openai_vision.rs` | OpenAI Vision集成 | ✅ |
| `audio.rs` | 音频处理核心 | ✅ |
| `real_audio.rs` | 实际音频实现 | ✅ |
| `openai_whisper.rs` | OpenAI Whisper集成 | ✅ |
| `video.rs` | 视频处理核心 | ✅ |
| `video_analyzer.rs` | 视频分析器 | ✅ |
| `text.rs` | 文本处理 | ✅ |
| `cross_modal.rs` | 跨模态处理 | ✅ |
| `unified_retrieval.rs` | 统一检索 | ✅ |
| `ai_models.rs` | AI模型集成 | ✅ |
| `optimization.rs` | 性能优化 | ✅ |
| `mod.rs` | 模块组织 | ✅ |

---

## 快速开始

### 安装依赖

```toml
[dependencies]
agent-mem-intelligence = { path = "crates/agent-mem-intelligence" }
tokio = { version = "1.0", features = ["full"] }
```

### 初始化

```rust
use agent_mem_intelligence::multimodal::*;

#[tokio::main]
async fn main() -> Result<()> {
    // 创建多模态处理器
    let image_processor = ImageProcessor::new().await?;
    let audio_processor = AudioProcessor::new().await?;
    let video_processor = VideoProcessor::new().await?;
    
    Ok(())
}
```

---

## 图像处理

### 基础图像分析

```rust
use agent_mem_intelligence::multimodal::image::*;

// 加载图像
let image_data = std::fs::read("path/to/image.jpg")?;

// 创建处理器
let processor = ImageProcessor::new().await?;

// 分析图像
let result = processor.analyze(&image_data).await?;

println!("描述: {}", result.description);
println!("标签: {:?}", result.tags);
println!("置信度: {}", result.confidence);
```

### 使用OpenAI Vision

```rust
use agent_mem_intelligence::multimodal::openai_vision::*;

let vision = OpenAIVision::new("your_api_key")?;

// 生成图像描述
let description = vision.describe_image(
    &image_data,
    "详细描述这张图片中的内容"
).await?;

println!("OpenAI描述: {}", description);
```

### 图像嵌入和搜索

```rust
// 生成图像嵌入向量
let embedding = processor.embed_image(&image_data).await?;

// 存储到记忆系统
memory.add_multimodal_memory(
    "image_001",
    &image_data,
    ModalityType::Image,
    Some(embedding)
).await?;

// 图像相似度搜索
let similar_images = memory.search_by_image(&query_image).await?;
```

### 支持的图像格式

- ✅ JPEG/JPG
- ✅ PNG
- ✅ WEBP
- ✅ GIF（静态）
- ✅ BMP

---

## 音频处理

### 语音转文本

```rust
use agent_mem_intelligence::multimodal::audio::*;

// 加载音频
let audio_data = std::fs::read("path/to/audio.mp3")?;

// 创建处理器
let processor = AudioProcessor::new().await?;

// 转录音频
let transcript = processor.transcribe(&audio_data).await?;

println!("转录文本: {}", transcript.text);
println!("语言: {}", transcript.language);
println!("置信度: {}", transcript.confidence);
```

### 使用OpenAI Whisper

```rust
use agent_mem_intelligence::multimodal::openai_whisper::*;

let whisper = OpenAIWhisper::new("your_api_key")?;

// 高质量转录
let result = whisper.transcribe(
    &audio_data,
    "zh",  // 中文
    true   // 添加标点
).await?;

println!("转录: {}", result.text);
println!("时间戳: {:?}", result.timestamps);
```

### 音频分析

```rust
// 提取音频特征
let features = processor.extract_features(&audio_data).await?;

println!("时长: {}秒", features.duration);
println!("采样率: {}Hz", features.sample_rate);
println!("音量: {}dB", features.volume);
println!("音调: {}Hz", features.pitch);
```

### 音频记忆存储

```rust
// 存储音频记忆（自动转录）
let memory_id = memory.add_audio_memory(
    &audio_data,
    Some("会议录音".to_string())
).await?;

// 搜索音频内容
let results = memory.search_audio("讨论了什么").await?;
```

### 支持的音频格式

- ✅ MP3
- ✅ WAV
- ✅ OGG
- ✅ FLAC
- ✅ M4A

---

## 视频处理

### 视频分析

```rust
use agent_mem_intelligence::multimodal::video::*;
use agent_mem_intelligence::multimodal::video_analyzer::*;

// 加载视频
let video_path = "path/to/video.mp4";

// 创建分析器
let analyzer = VideoAnalyzer::new().await?;

// 分析视频
let analysis = analyzer.analyze_video(video_path).await?;

println!("时长: {}秒", analysis.duration);
println!("帧率: {}fps", analysis.fps);
println!("分辨率: {}x{}", analysis.width, analysis.height);
println!("场景数: {}", analysis.scenes.len());
```

### 关键帧提取

```rust
// 提取关键帧
let keyframes = analyzer.extract_keyframes(
    video_path,
    10  // 提取10个关键帧
).await?;

for (index, frame) in keyframes.iter().enumerate() {
    println!("关键帧 {}: 时间 {}秒", index, frame.timestamp);
    
    // 分析每一帧
    let frame_desc = image_processor.analyze(&frame.data).await?;
    println!("描述: {}", frame_desc.description);
}
```

### 场景检测

```rust
// 检测场景变化
let scenes = analyzer.detect_scenes(video_path).await?;

for scene in scenes {
    println!("场景: {}秒 - {}秒", scene.start, scene.end);
    println!("描述: {}", scene.description);
}
```

### 视频记忆存储

```rust
// 存储视频记忆（自动提取关键帧和音频）
let memory_id = memory.add_video_memory(
    video_path,
    Some("产品演示视频".to_string())
).await?;

// 搜索视频内容
let results = memory.search_video("产品功能演示").await?;
```

### 支持的视频格式

- ✅ MP4
- ✅ AVI
- ✅ MOV
- ✅ MKV
- ✅ WEBM

---

## 跨模态检索

### 文本搜索图像

```rust
use agent_mem_intelligence::multimodal::cross_modal::*;

let cross_modal = CrossModalRetrieval::new().await?;

// 用文本搜索图像
let images = cross_modal.search_images_by_text(
    "一只可爱的猫咪",
    10  // 返回前10个结果
).await?;

for image in images {
    println!("图像ID: {}", image.id);
    println!("相似度: {}", image.similarity);
}
```

### 图像搜索文本

```rust
// 用图像搜索相关文本
let texts = cross_modal.search_texts_by_image(
    &query_image,
    10
).await?;

for text in texts {
    println!("文本内容: {}", text.content);
    println!("相关度: {}", text.relevance);
}
```

### 语音搜索

```rust
// 用语音搜索相关内容
let audio_query = std::fs::read("query.mp3")?;

let results = cross_modal.search_by_audio(
    &audio_query,
    ModalityType::All,  // 搜索所有模态
    10
).await?;

for result in results {
    match result.modality {
        ModalityType::Text => println!("找到文本: {}", result.content),
        ModalityType::Image => println!("找到图像: {}", result.id),
        ModalityType::Audio => println!("找到音频: {}", result.id),
        ModalityType::Video => println!("找到视频: {}", result.id),
    }
}
```

### 统一检索

```rust
use agent_mem_intelligence::multimodal::unified_retrieval::*;

let unified = UnifiedRetrieval::new().await?;

// 统一检索接口（自动识别查询类型）
let results = unified.search(
    query,  // 可以是文本、图像、音频
    SearchConfig {
        modalities: vec![
            ModalityType::Text,
            ModalityType::Image,
            ModalityType::Video
        ],
        limit: 20,
        threshold: 0.7,
    }
).await?;
```

---

## AI模型集成

### 配置模型

```rust
use agent_mem_intelligence::multimodal::ai_models::*;

let config = AIModelConfig {
    // 图像模型
    vision_model: ModelType::OpenAI("gpt-4-vision-preview".to_string()),
    vision_api_key: "your_openai_key".to_string(),
    
    // 音频模型
    audio_model: ModelType::OpenAI("whisper-1".to_string()),
    audio_api_key: "your_openai_key".to_string(),
    
    // 自定义模型
    custom_models: HashMap::new(),
};

let models = AIModels::from_config(config).await?;
```

### 使用自定义模型

```rust
// 注册自定义图像模型
models.register_vision_model(
    "my-model",
    Box::new(MyCustomVisionModel::new()?)
).await?;

// 使用自定义模型
let result = models.process_image(
    "my-model",
    &image_data
).await?;
```

### 支持的AI服务

| 服务 | 图像 | 音频 | 视频 |
|------|------|------|------|
| OpenAI | ✅ GPT-4V | ✅ Whisper | ⚠️ 间接 |
| Google | ⚠️ 规划 | ⚠️ 规划 | ⚠️ 规划 |
| 自定义 | ✅ | ✅ | ✅ |

---

## 性能优化

### 批量处理

```rust
// 批量处理图像
let images = vec![image1, image2, image3];
let results = processor.batch_analyze(images, 4).await?;  // 4个并发
```

### 缓存策略

```rust
use agent_mem_intelligence::multimodal::optimization::*;

// 启用缓存
let processor = ImageProcessor::with_cache(
    CacheConfig {
        max_size: 1000,  // 缓存1000个结果
        ttl: 3600,       // 1小时过期
    }
).await?;
```

### 流式处理

```rust
// 流式处理视频（减少内存使用）
let stream = analyzer.analyze_video_stream(video_path).await?;

while let Some(frame_result) = stream.next().await {
    // 处理每一帧
    process_frame(frame_result?).await?;
}
```

---

## 实际应用示例

### 示例1: 智能相册

```rust
async fn build_smart_album() -> Result<()> {
    let image_processor = ImageProcessor::new().await?;
    let memory = Memory::new().await?;
    
    // 批量导入照片
    let photos = std::fs::read_dir("photos/")?;
    
    for photo in photos {
        let path = photo?.path();
        let data = std::fs::read(&path)?;
        
        // 分析图像
        let analysis = image_processor.analyze(&data).await?;
        
        // 存储到记忆系统
        memory.add_image_memory(
            &data,
            Some(format!("照片: {}", analysis.description)),
            analysis.tags
        ).await?;
    }
    
    // 智能搜索
    let beach_photos = memory.search_images("海滩").await?;
    let family_photos = memory.search_images("家人").await?;
    
    Ok(())
}
```

### 示例2: 会议记录系统

```rust
async fn process_meeting_recording() -> Result<()> {
    let audio_processor = AudioProcessor::new().await?;
    let memory = Memory::new().await?;
    
    // 处理会议录音
    let audio_data = std::fs::read("meeting.mp3")?;
    
    // 转录
    let transcript = audio_processor.transcribe(&audio_data).await?;
    
    // 提取关键信息
    let key_points = extract_key_points(&transcript.text)?;
    
    // 存储记忆
    for point in key_points {
        memory.add(format!("会议要点: {}", point)).await?;
    }
    
    // 可搜索的会议记录
    let action_items = memory.search("行动项").await?;
    
    Ok(())
}
```

### 示例3: 视频内容分析

```rust
async fn analyze_video_content() -> Result<()> {
    let video_analyzer = VideoAnalyzer::new().await?;
    let image_processor = ImageProcessor::new().await?;
    let memory = Memory::new().await?;
    
    // 分析视频
    let analysis = video_analyzer.analyze_video("tutorial.mp4").await?;
    
    // 提取关键帧
    let keyframes = video_analyzer.extract_keyframes("tutorial.mp4", 20).await?;
    
    // 分析每个关键帧
    for (i, frame) in keyframes.iter().enumerate() {
        let desc = image_processor.analyze(&frame.data).await?;
        
        memory.add_multimodal_memory(
            &format!("tutorial_frame_{}", i),
            &frame.data,
            ModalityType::Image,
            Some(frame.embedding.clone())
        ).await?;
    }
    
    // 搜索特定场景
    let coding_scenes = memory.search_video("编程演示").await?;
    
    Ok(())
}
```

---

## 最佳实践

### 1. 选择合适的模型

```rust
// 对于简单任务，使用本地模型
let local_processor = ImageProcessor::local()?;

// 对于复杂任务，使用云端API
let cloud_processor = ImageProcessor::with_openai("api_key")?;
```

### 2. 批量处理优化

```rust
// 收集待处理项
let mut batch = Vec::new();

for item in items {
    batch.push(item);
    
    // 每100个批量处理一次
    if batch.len() >= 100 {
        process_batch(&batch).await?;
        batch.clear();
    }
}
```

### 3. 错误处理

```rust
// 优雅的降级处理
let description = match vision.describe_image(&image).await {
    Ok(desc) => desc,
    Err(e) => {
        warn!("Vision API失败: {}, 使用备用方案", e);
        fallback_describe_image(&image)?
    }
};
```

### 4. 元数据管理

```rust
// 存储完整的元数据
memory.add_multimodal_memory_with_metadata(
    &data,
    ModalityType::Image,
    HashMap::from([
        ("source", "camera"),
        ("timestamp", "2025-10-24"),
        ("location", "Beijing"),
        ("tags", "family,vacation"),
    ])
).await?;
```

---

## 故障排除

### 常见问题

**Q: OpenAI API调用失败？**
```rust
// 检查API密钥
let api_key = std::env::var("OPENAI_API_KEY")?;

// 添加重试逻辑
let result = retry_with_backoff(|| {
    vision.describe_image(&image)
}, 3).await?;
```

**Q: 内存占用过高？**
```rust
// 使用流式处理
let stream = processor.process_stream(large_video).await?;

// 定期清理缓存
processor.clear_cache().await?;
```

**Q: 处理速度慢？**
```rust
// 启用并行处理
let results = processor.parallel_process(
    items,
    num_cpus::get()  // 使用所有CPU核心
).await?;
```

---

## 性能指标

| 操作 | 平均耗时 | 内存使用 |
|------|---------|---------|
| 图像分析 | 100-500ms | 10-50MB |
| 音频转录（1分钟） | 2-5秒 | 50-100MB |
| 视频关键帧提取 | 1-3秒/分钟 | 100-200MB |
| 跨模态搜索 | 50-200ms | 20-100MB |

---

## 下一步

- 📖 阅读 [图记忆指南](graph-memory-guide.md)
- 📖 阅读 [搜索引擎指南](search-engines-guide.md)
- 🔗 查看 [API文档](https://docs.rs/agent-mem-intelligence)
- 💡 查看 [多模态示例](../examples/multimodal-demo)

---

**最后更新**: 2025-10-24  
**版本**: v1.0  
**反馈**: 请在GitHub Issues提交问题或建议

