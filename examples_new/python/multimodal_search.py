#!/usr/bin/env python3
"""
AgentMem Python SDK - 多模态搜索示例

这个示例演示了如何使用 AgentMem 处理和搜索多种类型的数据：
- 图像描述搜索
- 音频转录搜索
- 文档搜索
- 跨模态检索

运行方式:
```bash
export AGENTMEM_API_BASE_URL=http://localhost:8080
export AGENTMEM_API_KEY=your_api_key

python multimodal_search.py
```

预期输出:
```
🎨 AgentMem 多模态搜索示例

✅ 初始化完成

📸 步骤 1: 图像描述索引
   ✅ 索引: "日落海滩照片，橙色天空"
   ✅ 索引: "城市夜景，灯光璀璨"
   ✅ 索引: "猫在阳光下睡觉"

🎵 步骤 2: 音频转录索引
   ✅ 索引: "会议讨论了项目进度"
   ✅ 索引: "电话留言: 明天开会"
   ✅ 索引: "播客: AI 的未来"

📄 步骤 3: 文档索引
   ✅ 索引: "项目报告..."
   ✅ 索引: "会议纪要..."

🔍 步骤 4: 跨模态搜索
   搜索: "会议"
   ✅ 找到 3 条结果:
      1. [音频] 会议讨论了项目进度
      2. [音频] 电话留言: 明天开会
      3. [文档] 会议纪要

🎯 步骤 5: 模态过滤
   搜索: "照片" + 过滤: type=image
   ✅ 找到 3 条图像结果

🎉 完成！
```
"""

import asyncio
import os
from typing import List, Dict, Optional
from enum import Enum
from dataclasses import dataclass


try:
    from agentmem import AgentMemClient, Config, SearchQuery, MemoryType
except ImportError:
    print("⚠️  AgentMem SDK 未安装")
    print("   安装方式: pip install agentmem")
    exit(1)


class ContentType(Enum):
    """内容类型"""
    TEXT = "text"
    IMAGE = "image"
    AUDIO = "audio"
    VIDEO = "video"
    DOCUMENT = "document"


@dataclass
class MultimodalContent:
    """多模态内容"""
    content: str
    content_type: ContentType
    source: str = ""
    metadata: Dict[str, str] = None

    def __post_init__(self):
        if self.metadata is None:
            self.metadata = {}


class MultimodalSearchEngine:
    """多模态搜索引擎"""

    def __init__(self, client: AgentMemClient, user_id: str):
        """初始化搜索引擎"""
        self.client = client
        self.user_id = user_id
        self.agent_id = "multimodal_search"

    async def index_content(self, item: MultimodalContent) -> str:
        """索引内容"""
        metadata = {
            "type": item.content_type.value,
            "source": item.source,
            **item.metadata,
        }

        memory_id = await self.client.add_memory(
            content=item.content,
            agent_id=self.agent_id,
            user_id=self.user_id,
            memory_type=MemoryType.SEMANTIC,
            metadata=metadata,
        )

        return memory_id

    async def batch_index(self, items: List[MultimodalContent]) -> List[str]:
        """批量索引"""
        memory_ids = []
        for item in items:
            memory_id = await self.index_content(item)
            memory_ids.append(memory_id)
        return memory_ids

    async def search(
        self,
        query: str,
        content_type: Optional[ContentType] = None,
        top_k: int = 5,
        threshold: float = 0.7
    ) -> List[dict]:
        """搜索内容"""
        search_query = SearchQuery(
            query=query,
            user_id=self.user_id,
            limit=top_k,
            threshold=threshold,
        )

        results = await self.client.search_memories(search_query)

        # 如果指定了内容类型，进行过滤
        if content_type:
            results = [
                r for r in results
                if r.get("metadata", {}).get("type") == content_type.value
            ]

        return results

    async def get_stats(self) -> Dict[str, int]:
        """获取统计信息"""
        all_memories = await self.client.get_all_memories(
            user_id=self.user_id,
            limit=1000,
        )

        stats = {}
        for memory in all_memories:
            content_type = memory.get("metadata", {}).get("type", "unknown")
            stats[content_type] = stats.get(content_type, 0) + 1

        return stats


async def demo_image_indexing(engine: MultimodalSearchEngine):
    """演示图像索引"""
    print("\n📸 步骤 1: 图像描述索引")
    print("---")

    images = [
        MultimodalContent(
            content="日落海滩照片，有橙色的天空和蓝色的大海",
            content_type=ContentType.IMAGE,
            source="image_001.jpg",
            metadata={"time": "2025-01-01 18:00", "location": "海滩"},
        ),
        MultimodalContent(
            content="城市的夜景，灯光璀璨，高楼林立",
            content_type=ContentType.IMAGE,
            source="image_002.jpg",
            metadata={"time": "2025-01-02 20:00", "location": "市中心"},
        ),
        MultimodalContent(
            content="一只橘色的猫在阳光下睡觉，姿态可爱",
            content_type=ContentType.IMAGE,
            source="image_003.jpg",
            metadata={"time": "2025-01-03 14:00", "location": "家里"},
        ),
    ]

    memory_ids = await engine.batch_index(images)

    for img, memory_id in zip(images, memory_ids):
        print(f"   ✅ 索引: \"{img.content[:30]}...\" -> {memory_id}")


async def demo_audio_indexing(engine: MultimodalSearchEngine):
    """演示音频索引"""
    print("\n🎵 步骤 2: 音频转录索引")
    print("---")

    audio_files = [
        MultimodalContent(
            content="会议讨论了 Q4 的项目进度，确定了下一阶段的目标",
            content_type=ContentType.AUDIO,
            source="meeting_001.mp3",
            metadata={"duration": "15:30", "speaker": "项目经理"},
        ),
        MultimodalContent(
            content="电话留言: 明天下午三点开会，请准时参加",
            content_type=ContentType.AUDIO,
            source="voicemail_001.mp3",
            metadata={"duration": "0:45", "caller": "张三"},
        ),
        MultimodalContent(
            content="播客摘要: 讨论了 AI 技术的未来发展趋势和应用前景",
            content_type=ContentType.AUDIO,
            source="podcast_001.mp3",
            metadata={"duration": "45:00", "host": "科技主播"},
        ),
    ]

    memory_ids = await engine.batch_index(audio_files)

    for audio, memory_id in zip(audio_files, memory_ids):
        print(f"   ✅ 索引: \"{audio.content[:30]}...\" -> {memory_id}")


async def demo_document_indexing(engine: MultimodalSearchEngine):
    """演示文档索引"""
    print("\n📄 步骤 3: 文档索引")
    print("---")

    documents = [
        MultimodalContent(
            content="项目报告: 本季度完成了核心功能开发，测试覆盖率达到 85%",
            content_type=ContentType.DOCUMENT,
            source="report_q4.pdf",
            metadata={"pages": "15", "author": "项目经理"},
        ),
        MultimodalContent(
            content="会议纪要: 讨论了新功能的设计方案和实现计划",
            content_type=ContentType.DOCUMENT,
            source="minutes_001.docx",
            metadata={"pages": "3", "date": "2025-01-01"},
        ),
        MultimodalContent(
            content="技术文档: Rust 语言的并发编程模型和最佳实践",
            content_type=ContentType.DOCUMENT,
            source="rust_concurrency.md",
            metadata={"pages": "20", "author": "技术团队"},
        ),
    ]

    memory_ids = await engine.batch_index(documents)

    for doc, memory_id in zip(documents, memory_ids):
        print(f"   ✅ 索引: \"{doc.content[:30]}...\" -> {memory_id}")


async def demo_cross_modal_search(engine: MultimodalSearchEngine):
    """演示跨模态搜索"""
    print("\n🔍 步骤 4: 跨模态搜索")
    print("---")

    searches = [
        ("会议", "搜索会议相关内容"),
        ("项目", "搜索项目相关内容"),
        ("夜景", "搜索夜景图片"),
    ]

    for query, description in searches:
        print(f"\n   搜索: \"{query}\" ({description})")

        results = await engine.search(query, top_k=3)

        print(f"   ✅ 找到 {len(results)} 条结果:")

        for i, result in enumerate(results, 1):
            content = result.get("content", "")
            metadata = result.get("metadata", {})
            content_type = metadata.get("type", "unknown")
            source = metadata.get("source", "")
            score = result.get("score", 0.0)

            print(f"      {i}. [{content_type}] {content[:40]}... ({source}, {score:.2f})")


async def demo_type_filtering(engine: MultimodalSearchEngine):
    """演示类型过滤"""
    print("\n🎯 步骤 5: 模态过滤")
    print("---")

    # 只搜索图像
    print("\n   搜索: \"照片\" + 只看图像")

    results = await engine.search(
        "照片",
        content_type=ContentType.IMAGE,
        top_k=5,
    )

    print(f"   ✅ 找到 {len(results)} 条图像结果:")

    for i, result in enumerate(results, 1):
        content = result.get("content", "")
        source = result.get("metadata", {}).get("source", "")
        print(f"      {i}. {content[:50]}... ({source})")

    # 只搜索音频
    print("\n   搜索: \"会议\" + 只听音频")

    results = await engine.search(
        "会议",
        content_type=ContentType.AUDIO,
        top_k=5,
    )

    print(f"   ✅ 找到 {len(results)} 条音频结果:")

    for i, result in enumerate(results, 1):
        content = result.get("content", "")
        source = result.get("metadata", {}).get("source", "")
        print(f"      {i}. {content[:50]}... ({source})")


async def demo_semantic_understanding(engine: MultimodalSearchEngine):
    """演示语义理解"""
    print("\n💡 步骤 6: 语义理解")
    print("---")

    print("\n   测试跨模态语义理解:")

    tests = [
        ("美丽的风景", "应该找到海滩照片和城市夜景"),
        ("重要的讨论", "应该找到会议音频和会议纪要"),
        ("技术学习", "应该找到技术文档"),
    ]

    for query, expectation in tests:
        print(f"\n   查询: \"{query}\"")
        print(f"   期望: {expectation}")

        results = await engine.search(query, top_k=3)

        print(f"   结果: 找到 {len(results)} 条")

        if results:
            for i, result in enumerate(results[:2], 1):
                content = result.get("content", "")
                metadata = result.get("metadata", {})
                content_type = metadata.get("type", "unknown")
                print(f"      {i}. [{content_type}] {content[:40]}...")


async def main():
    """主函数"""
    print("🎨 AgentMem 多模态搜索示例\n")
    print("这个示例演示了:")
    print("  1. 图像描述索引")
    print("  2. 音频转录索引")
    print("  3. 文档索引")
    print("  4. 跨模态搜索")
    print("  5. 模态过滤")
    print("  6. 语义理解")
    print()

    # 初始化客户端
    api_base_url = os.getenv("AGENTMEM_API_BASE_URL", "http://localhost:8080")
    api_key = os.getenv("AGENTMEM_API_KEY", "demo_key")

    config = Config(
        api_base_url=api_base_url,
        api_key=api_key,
    )

    async with AgentMemClient(config) as client:
        print("✅ 初始化完成")

        # 创建搜索引擎
        engine = MultimodalSearchEngine(
            client=client,
            user_id="multimodal_user",
        )

        # 演示各种功能
        await demo_image_indexing(engine)
        await demo_audio_indexing(engine)
        await demo_document_indexing(engine)
        await demo_cross_modal_search(engine)
        await demo_type_filtering(engine)
        await demo_semantic_understanding(engine)

        # 显示统计
        stats = await engine.get_stats()

        print("\n📊 内容统计:")
        for content_type, count in sorted(stats.items()):
            print(f"   {content_type}: {count} 条")


if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("\n\n👋 用户中断")
    except Exception as e:
        print(f"\n\n❌ 错误: {e}")
        exit(1)


# ============================================
# 高级功能: 实际图像处理
# ============================================
#
# 集成真实的图像处理 API:
#
# ```python
# import base64
# import httpx
#
# async def describe_image(image_path: str) -> str:
#     """使用 Vision API 描述图像"""
#     # 读取图像
#     with open(image_path, "rb") as f:
#         image_data = base64.b64encode(f.read()).decode()
#
#     # 调用 Vision API（示例）
#     async with httpx.AsyncClient() as client:
#         response = await client.post(
#             "https://api.openai.com/v1/chat/completions",
#             headers={
#                 "Authorization": f"Bearer {os.getenv('OPENAI_API_KEY')}",
#             },
#             json={
#                 "model": "gpt-4-vision-preview",
#                 "messages": [{
#                     "role": "user",
#                     "content": [
#                         {"type": "text", "text": "描述这张图片"},
#                         {
#                             "type": "image_url",
#                             "image_url": {
#                                 "url": f"data:image/jpeg;base64,{image_data}"
#                             }
#                         }
#                     ]
#                 }]
#             }
#         )
#
#     result = response.json()
#     return result["choices"][0]["message"]["content"]
#
# # 使用
# description = await describe_image("photo.jpg")
# await engine.index_content(MultimodalContent(
#     content=description,
#     content_type=ContentType.IMAGE,
#     source="photo.jpg"
# ))
# ```
#
# ============================================
# 高级功能: 实际音频处理
# ============================================
#
# ```python
# async def transcribe_audio(audio_path: str) -> str:
#     """使用 Whisper API 转录音频"""
#     async with httpx.AsyncClient() as client:
#         with open(audio_path, "rb") as f:
#             response = await client.post(
#                 "https://api.openai.com/v1/audio/transcriptions",
#                 headers={
#                     "Authorization": f"Bearer {os.getenv('OPENAI_API_KEY')}",
#                 },
#                 files={"file": f},
#                 data={"model": "whisper-1"}
#             )
#
#     result = response.json()
#     return result["text"]
# ```
#
# ============================================
# 高级功能: 视频处理
# ============================================
#
# ```python
# async def process_video(video_path: str) -> List[MultimodalContent]:
#     """处理视频文件"""
#     contents = []
#
#     # 1. 提取关键帧
#     frames = extract_key_frames(video_path)
#     for i, frame in enumerate(frames):
#         description = await describe_image(frame)
#         contents.append(MultimodalContent(
#             content=f"视频帧 {i+1}: {description}",
#             content_type=ContentType.VIDEO,
#             source=video_path,
#             metadata={"frame": str(i)}
#         ))
#
#     # 2. 提取音频并转录
#     audio_path = extract_audio(video_path)
#     transcription = await transcribe_audio(audio_path)
#     contents.append(MultimodalContent(
#         content=f"视频音频: {transcription}",
#         content_type=ContentType.VIDEO,
#         source=video_path,
#         metadata={"type": "audio"}
#     ))
#
#     return contents
# ```
