# 实体和关系提取演示

此示例演示 AgentMem 的实体和关系提取功能，用于从文本中自动提取实体和关系，构建知识图谱。

## 功能特性

### 实体提取

支持以下实体类型：

- **Person** (人物): 姓名、称呼等
- **Organization** (组织): 公司、学校、政府机构等
- **Location** (地点): 城市、国家、地址等
- **Email** (邮箱): 电子邮件地址
- **Phone** (电话): 电话号码
- **Url** (网址): 网站链接
- **Date** (日期): 年月日
- **Time** (时间): 时分秒
- **Money** (金额): 货币金额
- **Number** (数字): 数量、编号等
- **Percentage** (百分比)
- **Event** (事件): 会议、活动等
- **Concept** (概念): 抽象概念、术语等
- **Product** (产品): 商品、服务等
- **Skill** (技能): 编程语言、专业技能等
- **Language** (语言): 自然语言
- **Technology** (技术): 技术栈、工具等

### 关系提取

支持以下关系类型：

- **WorksAt** (工作于): 人物-组织
- **LocatedAt** (位于): 实体-地点
- **FamilyOf** (家庭关系): 人物-人物
- **FriendOf** (朋友关系): 人物-人物
- **Likes** (喜欢): 人物-事物
- **Dislikes** (不喜欢): 人物-事物
- **Owns** (拥有): 人物-事物
- **ParticipatesIn** (参与): 人物-事件
- **OccursAt** (发生于): 事件-时间/地点
- **Causes** (导致): 事件-事件
- **BelongsTo** (属于): 实体-类别
- **Uses** (使用): 人物-技术/工具
- **Creates** (创建): 人物-产品/内容
- **Learns** (学习): 人物-技能/知识
- **Teaches** (教授): 人物-人物-技能

## 运行示例

```bash
cd agentmen
cargo run --package entity-relation-extraction-demo
```

## 示例输出

```
╔══════════════════════════════════════════════════════════════════════╗
║          AgentMem 实体和关系提取演示                                  ║
╚══════════════════════════════════════════════════════════════════════╝

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
1️⃣  基础实体提取
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📝 输入文本:
   我叫张三，在北京的谷歌公司工作。我的邮箱是zhangsan@google.com，电话是13800138000。

✅ 提取到 5 个实体（耗时: 2ms）:
   - 张三 [Person] (置信度: 0.85)
   - 北京 [Location] (置信度: 0.75)
   - 谷歌公司 [Organization] (置信度: 0.80)
   - zhangsan@google.com [Email] (置信度: 0.95)
   - 13800138000 [Phone] (置信度: 0.90)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
2️⃣  关系提取
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

✅ 提取到 2 个关系（耗时: 1ms）:
   - 张三 [工作于] 谷歌公司 (置信度: 0.75)
   - 谷歌公司 [位于] 北京 (置信度: 0.70)
```

## 代码示例

### 基础用法

```rust
use agent_mem_core::extraction::{
    EntityExtractor, RelationExtractor, 
    RuleBasedExtractor, RuleBasedRelationExtractor,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 创建提取器
    let entity_extractor = RuleBasedExtractor::new();
    let relation_extractor = RuleBasedRelationExtractor::new();

    // 提取实体
    let text = "我叫张三，在北京的谷歌公司工作。";
    let entities = entity_extractor.extract_entities(text).await?;

    // 提取关系
    let relations = relation_extractor.extract_relations(text, &entities).await?;

    println!("提取到 {} 个实体", entities.len());
    println!("提取到 {} 个关系", relations.len());

    Ok(())
}
```

### 高级用法

```rust
use agent_mem_core::extraction::{Entity, EntityType, Relation, RelationType};

// 遍历实体
for entity in &entities {
    println!("实体: {} [{}] (置信度: {:.2})", 
        entity.name, 
        entity.entity_type.as_str(), 
        entity.confidence
    );
    
    // 访问实体属性
    for (key, value) in &entity.attributes {
        println!("  属性: {} = {}", key, value);
    }
}

// 遍历关系
for relation in &relations {
    println!("关系: {} --[{}]--> {}", 
        relation.subject, 
        relation.predicate,
        relation.object
    );
}
```

## 实现细节

### 提取器架构

```
┌─────────────────────────────────────────┐
│         EntityExtractor Trait           │
│  - extract_entities(text) -> Vec<Entity>│
└─────────────────────────────────────────┘
                    ▲
                    │
        ┌───────────┴───────────┐
        │                       │
┌───────────────────┐  ┌────────────────────┐
│ RuleBasedExtractor│  │ LLMBasedExtractor  │
│  (基于规则)        │  │  (基于 LLM)        │
└───────────────────┘  └────────────────────┘
```

### 提取流程

1. **文本预处理**: 清理和标准化输入文本
2. **模式匹配**: 使用正则表达式匹配实体
3. **实体识别**: 识别实体类型和边界
4. **置信度计算**: 为每个实体计算置信度分数
5. **去重**: 移除重复的实体
6. **关系提取**: 基于实体和上下文提取关系
7. **结果返回**: 返回实体和关系列表

### 性能优化

- **正则表达式缓存**: 预编译正则表达式
- **并行处理**: 支持批量文本并行提取
- **增量提取**: 支持增量更新知识图谱
- **内存优化**: 使用引用避免不必要的复制

## 扩展性

### 自定义实体类型

```rust
use agent_mem_core::extraction::EntityType;

let custom_entity = Entity::new(
    "entity_001".to_string(),
    "Rust".to_string(),
    EntityType::Technology,
    0.90,
);
```

### 自定义关系类型

```rust
use agent_mem_core::extraction::RelationType;

let custom_relation = Relation::new(
    "rel_001".to_string(),
    "person_001".to_string(),
    "张三".to_string(),
    "学习".to_string(),
    "tech_001".to_string(),
    "Rust".to_string(),
    RelationType::Learns,
    0.85,
);
```

## 应用场景

1. **知识图谱构建**: 从文本中自动构建知识图谱
2. **信息抽取**: 从非结构化文本中提取结构化信息
3. **关系挖掘**: 发现实体之间的隐含关系
4. **问答系统**: 支持基于知识图谱的问答
5. **推荐系统**: 基于实体关系进行推荐

## 未来改进

- [ ] 支持基于 LLM 的实体提取
- [ ] 支持实体链接和消歧
- [ ] 支持多语言实体提取
- [ ] 支持实体属性提取
- [ ] 支持关系强度计算
- [ ] 支持时序关系提取
- [ ] 集成到图数据库

## 参考资料

- [实体识别 (NER)](https://en.wikipedia.org/wiki/Named-entity_recognition)
- [关系抽取](https://en.wikipedia.org/wiki/Relationship_extraction)
- [知识图谱](https://en.wikipedia.org/wiki/Knowledge_graph)

