//! 个性化记忆管理演示程序
//!
//! 展示 AgentMem 的个性化功能，包括：
//! - 用户行为记录和学习
//! - 个性化搜索和推荐
//! - 用户偏好管理
//! - 用户档案分析

use agent_mem_compat::{
    BehaviorType, Mem0Client, PersonalizationConfig, PersonalizationManager,
    PersonalizedSearchRequest, PreferenceType, UserBehavior, UserPreference,
};
use agent_mem_traits::Session;
use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    println!("🚀 启动个性化记忆管理演示");

    // 创建 Mem0 客户端
    let client = Mem0Client::new().await?;

    // 创建测试用户和会话
    let user_id = "personalization_demo_user";
    let session = Session {
        id: "personalization_demo_session".to_string(),
        user_id: Some(user_id.to_string()),
        agent_id: Some("demo_agent".to_string()),
        run_id: Some("demo_run".to_string()),
        actor_id: Some("demo_actor".to_string()),
        created_at: chrono::Utc::now(),
        metadata: HashMap::new(),
    };

    // 演示 1: 添加一些记忆
    println!("\n🎯 演示 1: 添加测试记忆");
    let memories = vec![
        "我喜欢学习 Rust 编程语言",
        "今天早上我在咖啡厅工作",
        "我对机器学习很感兴趣",
        "我经常在晚上阅读技术文档",
        "我喜欢使用 VSCode 编辑器",
        "我在学习分布式系统",
        "我喜欢喝咖啡",
        "我对区块链技术很好奇",
    ];

    let mut memory_ids = Vec::new();
    for (i, memory_content) in memories.iter().enumerate() {
        let memory_id = client.add(user_id, memory_content, None).await?;
        memory_ids.push(memory_id.clone());
        println!(
            "  ✅ 添加记忆 {}: {} (ID: {})",
            i + 1,
            memory_content,
            memory_id
        );
    }

    // 演示 2: 记录用户行为
    println!("\n🎯 演示 2: 记录用户行为");
    let behaviors = vec![
        UserBehavior {
            id: Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            behavior_type: BehaviorType::Search,
            memory_id: None,
            search_query: Some("Rust 编程".to_string()),
            context: HashMap::new(),
            timestamp: Utc::now(),
            session_id: session.id.clone(),
            duration: Some(30),
            result: Some("success".to_string()),
        },
        UserBehavior {
            id: Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            behavior_type: BehaviorType::Access,
            memory_id: Some(memory_ids[0].clone()),
            search_query: None,
            context: HashMap::new(),
            timestamp: Utc::now(),
            session_id: session.id.clone(),
            duration: Some(45),
            result: Some("success".to_string()),
        },
        UserBehavior {
            id: Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            behavior_type: BehaviorType::Search,
            memory_id: None,
            search_query: Some("机器学习".to_string()),
            context: HashMap::new(),
            timestamp: Utc::now(),
            session_id: session.id.clone(),
            duration: Some(25),
            result: Some("success".to_string()),
        },
        UserBehavior {
            id: Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            behavior_type: BehaviorType::Favorite,
            memory_id: Some(memory_ids[2].clone()),
            search_query: None,
            context: HashMap::new(),
            timestamp: Utc::now(),
            session_id: session.id.clone(),
            duration: None,
            result: Some("success".to_string()),
        },
    ];

    for (i, behavior) in behaviors.iter().enumerate() {
        client.record_user_behavior(behavior.clone()).await?;
        println!("  ✅ 记录行为 {}: {:?}", i + 1, behavior.behavior_type);
        if let Some(query) = &behavior.search_query {
            println!("    🔍 搜索查询: {}", query);
        }
        if let Some(memory_id) = &behavior.memory_id {
            println!("    📝 相关记忆: {}", memory_id);
        }
    }

    // 演示 3: 个性化搜索
    println!("\n🎯 演示 3: 个性化搜索");
    let search_request = PersonalizedSearchRequest {
        query: "编程学习".to_string(),
        user_id: user_id.to_string(),
        session: session.clone(),
        limit: Some(5),
        filters: None,
        enable_personalization: true,
        personalization_weight: 0.3,
    };

    println!("  🔍 搜索查询: {}", search_request.query);
    println!("  👤 用户ID: {}", search_request.user_id);
    println!("  ⚖️ 个性化权重: {}", search_request.personalization_weight);

    match client.personalized_search(search_request).await {
        Ok(results) => {
            println!("  ✅ 个性化搜索完成，找到 {} 条结果", results.len());
            for (i, result) in results.iter().enumerate() {
                println!("    {}. 内容: {}", i + 1, result.memory.memory);
                println!("       基础分数: {:.2}", result.base_score);
                println!("       个性化分数: {:.2}", result.personalization_score);
                println!("       最终分数: {:.2}", result.final_score);
                if !result.recommendation_reasons.is_empty() {
                    println!("       推荐原因: {:?}", result.recommendation_reasons);
                }
            }
        }
        Err(e) => {
            println!("  ❌ 个性化搜索失败: {}", e);
        }
    }

    // 演示 4: 获取用户偏好
    println!("\n🎯 演示 4: 用户偏好管理");
    match client.get_user_preferences(user_id).await {
        Ok(preferences) => {
            println!("  📊 用户偏好 ({} 个):", preferences.len());
            for (i, pref) in preferences.iter().enumerate() {
                println!("    {}. 类型: {:?}", i + 1, pref.preference_type);
                println!("       值: {}", pref.value);
                println!("       权重: {:.2}", pref.weight);
                println!("       置信度: {:.2}", pref.confidence);
                println!("       使用频率: {}", pref.usage_frequency);
            }
        }
        Err(e) => {
            println!("  ❌ 获取用户偏好失败: {}", e);
        }
    }

    // 演示 5: 手动添加用户偏好
    println!("\n🎯 演示 5: 手动添加用户偏好");
    let manual_preference = UserPreference {
        id: Uuid::new_v4().to_string(),
        user_id: user_id.to_string(),
        preference_type: PreferenceType::Topic,
        value: "分布式系统".to_string(),
        weight: 0.9,
        confidence: 0.8,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        usage_frequency: 1,
        metadata: HashMap::new(),
    };

    match client
        .update_user_preference(manual_preference.clone())
        .await
    {
        Ok(_) => {
            println!("  ✅ 成功添加用户偏好:");
            println!("    类型: {:?}", manual_preference.preference_type);
            println!("    值: {}", manual_preference.value);
            println!("    权重: {:.2}", manual_preference.weight);
        }
        Err(e) => {
            println!("  ❌ 添加用户偏好失败: {}", e);
        }
    }

    // 演示 6: 生成记忆推荐
    println!("\n🎯 演示 6: 记忆推荐");
    match client.generate_recommendations(user_id, 3).await {
        Ok(recommendations) => {
            if recommendations.is_empty() {
                println!("  📝 暂无推荐记忆（需要更多用户行为数据）");
            } else {
                println!("  🎯 推荐记忆 ({} 个):", recommendations.len());
                for (i, rec) in recommendations.iter().enumerate() {
                    println!("    {}. 内容: {}", i + 1, rec.memory.memory);
                    println!("       推荐分数: {:.2}", rec.score);
                    println!("       推荐类型: {}", rec.recommendation_type);
                    println!("       推荐原因: {:?}", rec.reasons);
                }
            }
        }
        Err(e) => {
            println!("  ❌ 生成推荐失败: {}", e);
        }
    }

    // 演示 7: 获取用户档案
    println!("\n🎯 演示 7: 用户档案分析");
    match client.get_user_profile(user_id).await {
        Ok(profile_opt) => {
            if let Some(profile) = profile_opt {
                println!("  👤 用户档案:");
                println!("    用户ID: {}", profile.user_id);
                println!("    偏好数量: {}", profile.preferences.len());
                println!("    行为历史: {} 条", profile.behavior_history.len());
                println!("    兴趣标签: {} 个", profile.interest_tags.len());
                println!("    活跃时间段: {:?}", profile.active_hours);

                println!("  📈 用户统计:");
                println!("    总搜索次数: {}", profile.stats.total_searches);
                println!("    总访问次数: {}", profile.stats.total_accesses);
                println!(
                    "    平均会话时长: {:.1} 秒",
                    profile.stats.avg_session_duration
                );
                println!("    最活跃时间: {}:00", profile.stats.most_active_hour);
                println!("    偏好多样性: {:.2}", profile.stats.preference_diversity);

                if !profile.stats.top_search_terms.is_empty() {
                    println!("    热门搜索词: {:?}", profile.stats.top_search_terms);
                }
            } else {
                println!("  📝 用户档案不存在，正在创建...");
                match client.update_user_profile(user_id).await {
                    Ok(new_profile) => {
                        println!("  ✅ 用户档案创建成功:");
                        println!("    用户ID: {}", new_profile.user_id);
                        println!("    偏好数量: {}", new_profile.preferences.len());
                        println!("    行为历史: {} 条", new_profile.behavior_history.len());
                    }
                    Err(e) => {
                        println!("  ❌ 创建用户档案失败: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            println!("  ❌ 获取用户档案失败: {}", e);
        }
    }

    println!("\n✅ 所有个性化记忆演示完成！");
    println!("\n🎉 个性化功能特点:");
    println!("  - 🧠 智能用户行为学习");
    println!("  - 🎯 个性化搜索和推荐");
    println!("  - 📊 用户偏好自动发现");
    println!("  - 👤 完整的用户档案分析");
    println!("  - 🔄 自适应学习和优化");

    Ok(())
}
