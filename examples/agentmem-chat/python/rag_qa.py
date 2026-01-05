#!/usr/bin/env python3
"""
AgentMem Python SDK - RAG 问答系统示例

这个示例演示了如何使用 AgentMem 构建 RAG (检索增强生成) 系统：
- 文档索引
- 语义检索
- 上下文增强生成
- 答案生成

运行方式:
```bash
export AGENTMEM_API_BASE_URL=http://localhost:8080
export AGENTMEM_API_KEY=your_api_key
export OPENAI_API_KEY=sk-...  # 如果使用 LLM

python rag_qa.py
```

预期输出:
```
📚 AgentMem RAG 问答系统示例

✅ 初始化完成

📖 步骤 1: 索引文档
   ✅ 索引: "Rust 是一门系统编程语言..."
   ✅ 索引: "Python 是一门高级编程语言..."
   ✅ 索引: "JavaScript 是一门脚本语言..."
   ✅ 已索引 3 份文档

🔍 步骤 2: 语义检索
   问题: "Rust 有什么特点？"
   ✅ 检索到 2 个相关片段:
      1. Rust 是一门系统编程语言，注重安全、并发和性能 (0.95)
      2. Rust 的所有权系统确保内存安全 (0.89)

💡 步骤 3: 生成答案
   ✅ 生成答案:
      根据检索到的文档，Rust 有以下特点:
      1. 系统编程语言，注重安全、并发和性能
      2. 所有权系统确保内存安全

🎯 步骤 4: 多轮问答
   Q: "Python 适合做什么？"
   A: Python 适合数据科学、机器学习、Web 开发...

   Q: "它有什么优势？"
   A: Python 的优势是简洁易读、生态丰富...

🎉 完成！
```
"""

import asyncio
import os
from typing import List, Dict, Optional
from dataclasses import dataclass

try:
    from agentmem import AgentMemClient, Config, SearchQuery, MemoryType
except ImportError:
    print("⚠️  AgentMem SDK 未安装")
    print("   安装方式: pip install agentmem")
    exit(1)


@dataclass
class Document:
    """文档"""
    title: str
    content: str
    source: str = ""


class RAGSystem:
    """RAG 问答系统"""

    def __init__(self, client: AgentMemClient, user_id: str):
        """初始化 RAG 系统"""
        self.client = client
        self.user_id = user_id
        self.agent_id = "rag_system"

    async def index_document(self, doc: Document) -> str:
        """索引文档"""
        # 将文档内容分段存储
        memory_id = await self.client.add_memory(
            content=f"{doc.title}: {doc.content}",
            agent_id=self.agent_id,
            user_id=self.user_id,
            memory_type=MemoryType.SEMANTIC,
            metadata={
                "type": "document",
                "title": doc.title,
                "source": doc.source,
                "indexed_at": str(asyncio.get_event_loop().time()),
            },
        )
        return memory_id

    async def batch_index(self, documents: List[Document]) -> List[str]:
        """批量索引文档"""
        memory_ids = []
        for doc in documents:
            memory_id = await self.index_document(doc)
            memory_ids.append(memory_id)
        return memory_ids

    async def retrieve(self, query: str, top_k: int = 3) -> List[dict]:
        """检索相关文档"""
        search_query = SearchQuery(
            query=query,
            user_id=self.user_id,
            limit=top_k,
            threshold=0.7,
        )
        results = await self.client.search_memories(search_query)
        return results

    async def generate_answer_simple(self, query: str, context: List[dict]) -> str:
        """生成简单答案（不使用 LLM）"""
        if not context:
            return "抱歉，我没有找到相关信息。"

        # 提取上下文内容
        contexts = [c.get("content", "") for c in context]

        # 构建答案
        answer = f"根据检索到的文档，{query}\n\n"

        for i, ctx in enumerate(contexts, 1):
            answer += f"{i}. {ctx}\n"

        return answer

    async def generate_answer_llm(self, query: str, context: List[dict]) -> str:
        """使用 LLM 生成答案（需要 OpenAI API）"""
        if not context:
            return "抱歉，我没有找到相关信息。"

        # 这里可以集成 OpenAI API
        # 示例代码（需要安装 openai 库）:
        #
        # import openai
        # openai.api_key = os.getenv("OPENAI_API_KEY")
        #
        # context_text = "\n".join([
        #     f"- {c.get('content', '')}"
        #     for c in context
        # ])
        #
        # prompt = f"""基于以下文档片段回答问题。如果文档中没有答案，请说"抱歉，文档中没有相关信息"。
        #
        # 文档片段:
        # {context_text}
        #
        # 问题: {query}
        #
        # 回答:"""
        #
        # response = await openai.ChatCompletion.acreate(
        #     model="gpt-4",
        #     messages=[
        #         {"role": "system", "content": "你是一个有帮助的问答助手。"},
        #         {"role": "user", "content": prompt}
        #     ]
        # )
        #
        # return response.choices[0].message.content

        # 简化版本
        return await self.generate_answer_simple(query, context)

    async def ask(self, query: str, use_llm: bool = False) -> str:
        """提问"""
        # 检索相关文档
        context = await self.retrieve(query)

        # 生成答案
        if use_llm:
            answer = await self.generate_answer_llm(query, context)
        else:
            answer = await self.generate_answer_simple(query, context)

        return answer


async def demo_document_indexing(rag: RAGSystem):
    """演示文档索引"""
    print("\n📖 步骤 1: 索引文档")
    print("---")

    documents = [
        Document(
            title="Rust 编程语言",
            content="Rust 是一门系统编程语言，注重安全、并发和性能。Rust 的所有权系统确保内存安全，无需垃圾回收。",
            source="rust_doc.md"
        ),
        Document(
            title="Python 编程语言",
            content="Python 是一门高级编程语言，以其简洁易读的语法著称。Python 广泛应用于数据科学、机器学习、Web 开发等领域。",
            source="python_doc.md"
        ),
        Document(
            title="JavaScript 编程语言",
            content="JavaScript 是一门脚本语言，主要用于 Web 开发。随着 Node.js 的出现，JavaScript 也可以用于服务端开发。",
            source="js_doc.md"
        ),
        Document(
            title="Go 编程语言",
            content="Go 是一门开源编程语言，专为构建简单、可靠和高效的软件而设计。Go 特别适合并发编程和网络服务。",
            source="go_doc.md"
        ),
    ]

    memory_ids = await rag.batch_index(documents)

    for doc, memory_id in zip(documents, memory_ids):
        print(f"   ✅ 索引: \"{doc.title}\" -> {memory_id}")

    print(f"\n   ✅ 已索引 {len(documents)} 份文档")


async def demo_semantic_retrieval(rag: RAGSystem):
    """演示语义检索"""
    print("\n🔍 步骤 2: 语义检索")
    print("---")

    queries = [
        "Rust 有什么特点？",
        "Python 适合做什么？",
        "JavaScript 有什么用途？",
    ]

    for query in queries:
        print(f"\n   问题: \"{query}\"")

        results = await rag.retrieve(query, top_k=2)

        print(f"   ✅ 检索到 {len(results)} 个相关片段:")

        for i, result in enumerate(results, 1):
            content = result.get("content", "")
            score = result.get("score", 0.0)
            print(f"      {i}. {content} ({score:.2f})")


async def demo_answer_generation(rag: RAGSystem):
    """演示答案生成"""
    print("\n💡 步骤 3: 生成答案")
    print("---")

    query = "Rust 有什么特点？"
    print(f"   问题: \"{query}\"")

    # 检索上下文
    context = await rag.retrieve(query, top_k=2)

    print(f"\n   检索到的上下文:")
    for i, ctx in enumerate(context, 1):
        content = ctx.get("content", "")
        print(f"      {i}. {content}")

    # 生成答案
    answer = await rag.generate_answer_simple(query, context)

    print(f"\n   ✅ 生成答案:")
    print(f"   {answer}")


async def demo_multi_turn_qa(rag: RAGSystem):
    """演示多轮问答"""
    print("\n🎯 步骤 4: 多轮问答")
    print("---")

    conversations = [
        ("Python 适合做什么？", "Python 适合数据科学、机器学习、Web 开发等领域"),
        ("它有什么优势？", "Python 的优势是简洁易读、生态丰富、易于学习"),
        ("有什么流行的框架？", "流行的 Python 框架包括 Django、Flask、FastAPI 等"),
    ]

    for question, expected_answer in conversations:
        print(f"\n   Q: \"{question}\"")

        answer = await rag.ask(question)

        # 显示前两行
        answer_lines = answer.split("\n")[:2]
        short_answer = "\n   ".join(answer_lines)

        print(f"   A: {short_answer}...")


async def demo_domain_specific_qa(rag: RAGSystem):
    """演示领域特定问答"""
    print("\n🎓 步骤 5: 领域特定问答")
    print("---")

    # 添加技术文档
    tech_docs = [
        Document(
            title="Rust 所有权系统",
            content="Rust 的所有权系统是其最独特的特性。每个值都有一个所有者，且同一时间只能有一个所有者。当所有者超出作用域，值将被丢弃。",
            source="ownership.md"
        ),
        Document(
            title="Rust 借用检查",
            content="Rust 的借用检查器确保引用总是有效的。你可以拥有不可变引用（&T）或可变引用（&mut T），但不能同时拥有两者。",
            source="borrowing.md"
        ),
    ]

    await rag.batch_index(tech_docs)
    print("   ✅ 已添加技术文档")

    # 技术问题
    questions = [
        "什么是 Rust 的所有权？",
        "Rust 如何保证内存安全？",
        "什么是借用检查？",
    ]

    for question in questions:
        print(f"\n   问题: \"{question}\"")

        answer = await rag.ask(question)

        # 显示第一行
        first_line = answer.split("\n")[0]
        print(f"   回答: {first_line}...")


async def main():
    """主函数"""
    print("📚 AgentMem RAG 问答系统示例\n")
    print("这个示例演示了:")
    print("  1. 文档索引")
    print("  2. 语义检索")
    print("  3. 上下文增强生成")
    print("  4. 多轮问答")
    print("  5. 领域特定知识")
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

        # 创建 RAG 系统
        rag = RAGSystem(
            client=client,
            user_id="rag_user",
        )

        # 演示各种功能
        await demo_document_indexing(rag)
        await demo_semantic_retrieval(rag)
        await demo_answer_generation(rag)
        await demo_multi_turn_qa(rag)
        await demo_domain_specific_qa(rag)

        # 显示统计
        all_memories = await client.get_all_memories(
            user_id="rag_user",
            limit=100,
        )

        print("\n📊 系统统计:")
        print(f"   已索引文档: {len(all_memories)}")


if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("\n\n👋 用户中断")
    except Exception as e:
        print(f"\n\n❌ 错误: {e}")
        exit(1)


# ============================================
# 高级功能: 文档分块
# ============================================
#
# 对于长文档，需要先分块再索引:
#
# ```python
# def chunk_document(text: str, chunk_size: int = 500) -> List[str]:
#     """将文档分成块"""
#     chunks = []
#     sentences = text.split("。")
#
#     current_chunk = ""
#     for sentence in sentences:
#         if len(current_chunk) + len(sentence) < chunk_size:
#             current_chunk += sentence + "。"
#         else:
#             chunks.append(current_chunk.strip())
#             current_chunk = sentence + "。"
#
#     if current_chunk:
#         chunks.append(current_chunk.strip())
#
#     return chunks
#
# async def index_long_document(rag: RAGSystem, doc: Document):
#     """索引长文档"""
#     chunks = chunk_document(doc.content)
#
#     for i, chunk in enumerate(chunks):
#         chunk_doc = Document(
#             title=f"{doc.title} (部分 {i+1}/{len(chunks)})",
#             content=chunk,
#             source=doc.source,
#         )
#         await rag.index_document(chunk_doc)
#
#     print(f"✅ 文档已分为 {len(chunks)} 个块并索引")
# ```
#
# ============================================
# 高级功能: 混合检索
# ============================================
#
# 结合语义检索和关键词检索:
#
# ```python
# async def hybrid_retrieve(rag: RAGSystem, query: str) -> List[dict]:
#     """混合检索"""
#     # 语义检索
#     semantic_results = await rag.retrieve(query, top_k=5)
#
#     # 关键词检索（假设有这个功能）
#     # keyword_results = await rag.keyword_search(query, top_k=5)
#
#     # 合并和重新排序
#     # 这里简化处理
#     return semantic_results
# ```
#
# ============================================
# 高级功能: 答案质量评估
# ============================================
#
# ```python
# def evaluate_answer(answer: str, context: List[dict]) -> float:
#     """评估答案质量"""
#     # 简单的评估指标
#     score = 0.0
#
#     # 1. 答案长度
#     if len(answer) > 50:
#         score += 0.3
#
#     # 2. 是否包含上下文信息
#     context_text = " ".join([c.get("content", "") for c in context])
#     words_in_context = sum(1 for word in answer.split() if word in context_text)
#     if words_in_context > 0:
#         score += 0.4 * (words_in_context / len(answer.split()))
#
#     # 3. 答案完整性
#     if "。" in answer or "." in answer:
#         score += 0.3
#
#     return min(score, 1.0)
# ```
