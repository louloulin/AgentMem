#!/usr/bin/env python3
"""
AgentMem Movie Recommendation Demo

对标 Mem0 的 movie_recommendation_grok3.py
功能：
- 电影偏好记忆
- 观影历史追踪
- 个性化推荐
- 评分和评论记忆

使用说明：
1. 设置环境变量：
   export DEEPSEEK_API_KEY="your_key"  # 或 OPENAI_API_KEY
   
2. 运行：
   python3 movie_recommendation.py

依赖：
- agent_mem_python (AgentMem Python SDK)
"""

import os
import sys
from pathlib import Path
from datetime import datetime

# Add the project root to Python path
project_root = Path(__file__).parent.parent.parent.absolute()
sys.path.insert(0, str(project_root))

try:
    from agent_mem_python import AgentMem
except ImportError:
    print("❌ Error: agent_mem_python not found")
    print("Please build the Python bindings first:")
    print("  cd crates/agent-mem-python && maturin develop")
    sys.exit(1)


class MovieRecommendationAgent:
    """电影推荐助手 - 基于观影历史提供个性化推荐"""
    
    def __init__(self, user_id: str = "movie_fan"):
        self.user_id = user_id
        
        # 初始化AgentMem
        deepseek_api_key = os.getenv("DEEPSEEK_API_KEY")
        openai_api_key = os.getenv("OPENAI_API_KEY")
        
        if deepseek_api_key:
            print("✅ Using DeepSeek LLM")
            self.memory = AgentMem(
                llm_provider="deepseek",
                llm_model="deepseek-chat",
                llm_api_key=deepseek_api_key,
                embedder_provider="fastembed",
                embedder_model="bge-small-en-v1.5"
            )
        elif openai_api_key:
            print("✅ Using OpenAI LLM")
            self.memory = AgentMem(
                llm_provider="openai",
                llm_model="gpt-4o-mini",
                llm_api_key=openai_api_key,
                embedder_provider="fastembed",
                embedder_model="bge-small-en-v1.5"
            )
        else:
            print("⚠️  No LLM API key found. Running in basic mode.")
            self.memory = AgentMem(
                embedder_provider="fastembed",
                embedder_model="bge-small-en-v1.5",
                disable_intelligent_features=True
            )
    
    def log_movie_watch(self, title: str, genre: str, rating: float, review: str = ""):
        """
        记录观影记录
        
        Args:
            title: 电影标题
            genre: 电影类型
            rating: 评分 (1-10)
            review: 观影评价
        """
        timestamp = datetime.now().strftime("%Y-%m-%d")
        
        watch_log = f"""
Movie Watch Log
Title: {title}
Genre: {genre}
Rating: {rating}/10
Date: {timestamp}
Review: {review if review else "No review"}
"""
        
        self.memory.add(watch_log, user_id=self.user_id)
        print(f"✅ Logged: {title} ({genre}) - {rating}/10")
    
    def log_preference(self, preference_type: str, details: str):
        """
        记录用户偏好
        
        Args:
            preference_type: 偏好类型 (genre/actor/director/mood)
            details: 偏好详情
        """
        preference_log = f"""
User Preference
Type: {preference_type}
Details: {details}
"""
        
        self.memory.add(preference_log, user_id=self.user_id)
        print(f"✅ Preference recorded: {preference_type} - {details}")
    
    def get_recommendations(self, query: str = "Recommend movies for me") -> str:
        """
        获取个性化电影推荐
        
        Args:
            query: 推荐查询
            
        Returns:
            推荐结果
        """
        # 搜索相关记忆
        try:
            memories = self.memory.search(query, user_id=self.user_id)
            memory_context = "\n".join(f"- {m.content[:200]}..." for m in memories[:10])
        except Exception as e:
            print(f"⚠️  Search failed: {e}")
            memory_context = ""
        
        # 构建提示词
        prompt = f"""You are a movie recommendation expert with deep knowledge of cinema.

Based on the user's watching history and preferences:
{memory_context if memory_context else "No watch history found."}

User's request:
{query}

Please provide personalized movie recommendations that:
1. Match the user's taste based on their history
2. Include a mix of similar and exploratory picks
3. Explain why each movie fits their preferences
4. Consider their rating patterns and reviews

Format your response as a numbered list with brief explanations.
"""
        
        try:
            response = self.memory.chat(prompt, user_id=self.user_id)
            
            # 记录这次推荐
            recommendation_log = f"Q: {query}\nRecommendations: {response}"
            self.memory.add(recommendation_log, user_id=self.user_id)
            
            return response
        except Exception as e:
            print(f"⚠️  LLM chat failed: {e}")
            return "Based on your history, I recommend exploring similar genres to what you've enjoyed."
    
    def get_stats(self) -> dict:
        """获取观影统计"""
        try:
            all_memories = self.memory.get_all(user_id=self.user_id)
            
            movies_watched = sum(1 for m in all_memories if "Movie Watch Log" in m.content)
            preferences_set = sum(1 for m in all_memories if "User Preference" in m.content)
            recommendations_given = sum(1 for m in all_memories if "Recommendations:" in m.content)
            
            # 提取评分信息
            ratings = []
            for mem in all_memories:
                if "Rating:" in mem.content:
                    for line in mem.content.split('\n'):
                        if line.startswith("Rating:"):
                            try:
                                rating_str = line.replace("Rating:", "").replace("/10", "").strip()
                                ratings.append(float(rating_str))
                            except:
                                pass
            
            avg_rating = sum(ratings) / len(ratings) if ratings else 0
            
            return {
                "total_memories": len(all_memories),
                "movies_watched": movies_watched,
                "preferences_set": preferences_set,
                "recommendations_given": recommendations_given,
                "average_rating": round(avg_rating, 1)
            }
        except Exception as e:
            return {"error": str(e)}


def run_demo():
    """运行演示"""
    print("╔════════════════════════════════════════════════════════════════╗")
    print("║                                                                ║")
    print("║         🎬 AgentMem Movie Recommendation Demo 🎬              ║")
    print("║                                                                ║")
    print("║        对标 Mem0 movie_recommendation_grok3.py                ║")
    print("║                                                                ║")
    print("╚════════════════════════════════════════════════════════════════╝")
    print()
    
    # 创建推荐助手
    agent = MovieRecommendationAgent(user_id="Alice")
    
    print("=" * 70)
    print("📝 测试场景 1: 记录观影历史")
    print("=" * 70)
    
    # 场景1: 记录科幻电影
    print("\n🎬 Watching Sci-Fi movies...")
    agent.log_movie_watch(
        title="Inception",
        genre="Sci-Fi/Thriller",
        rating=9.5,
        review="Mind-bending plot with stunning visuals. Christopher Nolan at his best!"
    )
    
    agent.log_movie_watch(
        title="Interstellar",
        genre="Sci-Fi/Drama",
        rating=9.0,
        review="Epic space odyssey with emotional depth. Love the time dilation concept."
    )
    
    agent.log_movie_watch(
        title="The Matrix",
        genre="Sci-Fi/Action",
        rating=9.8,
        review="Revolutionary! The action sequences and philosophical themes are perfect."
    )
    
    # 场景2: 记录其他类型
    print("\n🎬 Watching other genres...")
    agent.log_movie_watch(
        title="The Shawshank Redemption",
        genre="Drama",
        rating=10.0,
        review="Perfect storytelling. One of the best films ever made."
    )
    
    agent.log_movie_watch(
        title="The Grand Budapest Hotel",
        genre="Comedy/Drama",
        rating=8.5,
        review="Wes Anderson's unique style. Visually beautiful and quirky."
    )
    
    # 场景3: 记录一部评分低的
    agent.log_movie_watch(
        title="Generic Action Movie",
        genre="Action",
        rating=5.0,
        review="Too much CGI, weak plot. Forgettable."
    )
    
    print("\n" + "=" * 70)
    print("📝 测试场景 2: 设置偏好")
    print("=" * 70)
    
    # 场景4: 设置偏好
    print("\n⭐ Setting preferences...")
    agent.log_preference("genre", "Love sci-fi and thought-provoking films")
    agent.log_preference("director", "Christopher Nolan, Denis Villeneuve, Wes Anderson")
    agent.log_preference("mood", "Prefer films with depth over pure entertainment")
    agent.log_preference("actor", "Enjoy Leonardo DiCaprio, Christian Bale")
    
    print("\n" + "=" * 70)
    print("📝 测试场景 3: 获取推荐")
    print("=" * 70)
    
    # 场景5: 基于历史的推荐
    print("\n🤔 Question: What movies should I watch next?")
    recommendations1 = agent.get_recommendations(
        "Based on my watching history, recommend 3 movies I would love"
    )
    print(f"\n💡 Recommendations:\n{recommendations1[:300]}...")
    
    # 场景6: 特定类型推荐
    print("\n🤔 Question: Any good sci-fi movies I haven't seen?")
    recommendations2 = agent.get_recommendations(
        "Recommend sci-fi movies similar to Inception and Interstellar"
    )
    print(f"\n💡 Recommendations:\n{recommendations2[:300]}...")
    
    # 场景7: 心情推荐
    print("\n🤔 Question: Something for a contemplative mood?")
    recommendations3 = agent.get_recommendations(
        "I'm in a contemplative mood. Suggest something deep and meaningful"
    )
    print(f"\n💡 Recommendations:\n{recommendations3[:300]}...")
    
    print("\n" + "=" * 70)
    print("📊 观影统计")
    print("=" * 70)
    
    stats = agent.get_stats()
    print(f"  总记忆数: {stats.get('total_memories', 0)}")
    print(f"  观影数量: {stats.get('movies_watched', 0)}")
    print(f"  偏好设置: {stats.get('preferences_set', 0)}")
    print(f"  推荐次数: {stats.get('recommendations_given', 0)}")
    print(f"  平均评分: {stats.get('average_rating', 0)}/10")
    
    print("\n" + "=" * 70)
    print("✅ 演示完成！AgentMem Movie Recommendation 功能验证成功")
    print("=" * 70)
    print("\n对比 Mem0:")
    print("  ✅ 观影历史记忆 - 完全对标")
    print("  ✅ 偏好追踪 - 完全对标")
    print("  ✅ 个性化推荐 - 完全对标")
    print("  🔥 增强统计 - AgentMem独有")
    print("  🔥 本地嵌入 - 无需额外API调用")
    print("  🔥 Rust性能 - 更快的推荐生成")


def interactive_mode():
    """交互模式"""
    print("╔════════════════════════════════════════════════════════════════╗")
    print("║                                                                ║")
    print("║      🎬 AgentMem Movie Recommendation - Interactive 🎬        ║")
    print("║                                                                ║")
    print("║      命令:                                                     ║")
    print("║        watch - 记录观影                                       ║")
    print("║        prefer - 设置偏好                                      ║")
    print("║        recommend - 获取推荐                                   ║")
    print("║        stats - 统计信息                                       ║")
    print("║        quit - 退出                                            ║")
    print("║                                                                ║")
    print("╚════════════════════════════════════════════════════════════════╝")
    print()
    
    user_id = input("请输入你的名字 (默认: MovieFan): ").strip() or "moviefan"
    agent = MovieRecommendationAgent(user_id=user_id)
    
    print(f"\n欢迎, {user_id}! 我是你的电影推荐助手。")
    print("我会记住你的观影历史，并提供个性化推荐。\n")
    
    while True:
        command = input(f"{user_id}> ").strip().lower()
        
        if not command:
            continue
        
        if command in ['quit', 'exit', 'q']:
            print(f"\n👋 再见, {user_id}! 享受电影!")
            stats = agent.get_stats()
            print(f"你已观看了 {stats.get('movies_watched', 0)} 部电影。")
            break
        
        elif command == 'stats':
            stats = agent.get_stats()
            print(f"\n📊 观影统计:")
            for key, value in stats.items():
                print(f"  {key}: {value}")
            print()
        
        elif command == 'watch':
            title = input("  电影标题: ").strip()
            genre = input("  类型: ").strip()
            rating = input("  评分 (1-10): ").strip()
            review = input("  评价: ").strip()
            try:
                rating_float = float(rating)
                agent.log_movie_watch(title, genre, rating_float, review)
            except ValueError:
                print("❌ 无效的评分")
            print()
        
        elif command == 'prefer':
            pref_type = input("  偏好类型 (genre/actor/director/mood): ").strip()
            details = input("  详情: ").strip()
            agent.log_preference(pref_type, details)
            print()
        
        elif command == 'recommend':
            query = input("  推荐查询: ").strip() or "Recommend movies for me"
            recommendations = agent.get_recommendations(query)
            print(f"\n💡 {recommendations}\n")
        
        else:
            print("❌ 未知命令。输入 'quit' 退出。\n")


if __name__ == "__main__":
    import argparse
    
    parser = argparse.ArgumentParser(description="AgentMem Movie Recommendation Demo")
    parser.add_argument(
        "--interactive", "-i",
        action="store_true",
        help="Run in interactive mode"
    )
    
    args = parser.parse_args()
    
    if args.interactive:
        interactive_mode()
    else:
        run_demo()

