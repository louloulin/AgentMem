//! 记忆路由工具函数测试
//! 
//! 测试中文检测和自适应阈值计算功能

#[cfg(test)]
mod tests {
    use super::super::memory::*;

    /// 测试中文检测函数
    #[test]
    fn test_contains_chinese() {
        // 测试中文字符
        assert!(contains_chinese("仓颉"));
        assert!(contains_chinese("中文测试"));
        assert!(contains_chinese("Hello 世界"));
        assert!(contains_chinese("测试123"));
        
        // 测试非中文字符
        assert!(!contains_chinese("Hello World"));
        assert!(!contains_chinese("123456"));
        assert!(!contains_chinese("product_id"));
        assert!(!contains_chinese(""));
        
        // 测试混合内容
        assert!(contains_chinese("商品ID: P123456"));
        assert!(!contains_chinese("Product ID: P123456"));
    }

    /// 测试自适应阈值计算 - 中文查询
    #[test]
    fn test_get_adaptive_threshold_chinese() {
        // 中文短查询应该使用较低阈值
        let threshold1 = get_adaptive_threshold("仓颉");
        assert!(threshold1 < 0.3, "中文短查询阈值应该 < 0.3, 实际: {}", threshold1);
        assert!(threshold1 >= 0.1, "阈值应该 >= 0.1, 实际: {}", threshold1);
        
        // 中文中等长度查询
        let threshold2 = get_adaptive_threshold("仓颉是造字圣人");
        assert!(threshold2 < 0.5, "中文中等查询阈值应该 < 0.5, 实际: {}", threshold2);
        
        // 中文长查询
        let threshold3 = get_adaptive_threshold("仓颉是中国古代传说中的人物，被尊为造字圣人");
        assert!(threshold3 < 0.7, "中文长查询阈值应该 < 0.7, 实际: {}", threshold3);
    }

    /// 测试自适应阈值计算 - 英文查询
    #[test]
    fn test_get_adaptive_threshold_english() {
        // 英文短查询
        let threshold1 = get_adaptive_threshold("test");
        assert!(threshold1 >= 0.3, "英文短查询阈值应该 >= 0.3, 实际: {}", threshold1);
        
        // 英文中等长度查询
        let threshold2 = get_adaptive_threshold("This is a test query");
        assert!(threshold2 >= 0.5, "英文中等查询阈值应该 >= 0.5, 实际: {}", threshold2);
        
        // 英文长查询
        let threshold3 = get_adaptive_threshold("This is a very long test query that should have a higher threshold");
        assert!(threshold3 >= 0.7, "英文长查询阈值应该 >= 0.7, 实际: {}", threshold3);
    }

    /// 测试自适应阈值计算 - 精确ID查询
    #[test]
    fn test_get_adaptive_threshold_exact_id() {
        // 商品ID格式
        let threshold1 = get_adaptive_threshold("P123456");
        assert_eq!(threshold1, 0.1, "商品ID阈值应该为0.1");
        
        // UUID格式
        let threshold2 = get_adaptive_threshold("550e8400-e29b-41d4-a716-446655440000");
        assert_eq!(threshold2, 0.1, "UUID阈值应该为0.1");
        
        // 其他精确ID
        let threshold3 = get_adaptive_threshold("abc123def");
        assert_eq!(threshold3, 0.2, "精确ID阈值应该为0.2");
    }

    /// 测试自适应阈值计算 - 混合中英文
    #[test]
    fn test_get_adaptive_threshold_mixed() {
        // 包含中文的查询应该降低阈值
        let threshold1 = get_adaptive_threshold("商品ID");
        assert!(threshold1 < 0.4, "包含中文的商品查询阈值应该 < 0.4, 实际: {}", threshold1);
        
        // 纯英文的商品查询
        let threshold2 = get_adaptive_threshold("product id");
        assert_eq!(threshold2, 0.4, "纯英文商品查询阈值应该为0.4");
    }

    /// 测试阈值边界情况
    #[test]
    fn test_get_adaptive_threshold_boundaries() {
        // 空字符串
        let threshold1 = get_adaptive_threshold("");
        assert!(threshold1 >= 0.1 && threshold1 <= 0.9);
        
        // 单个中文字符
        let threshold2 = get_adaptive_threshold("中");
        assert!(threshold2 >= 0.1 && threshold2 < 0.3);
        
        // 单个英文字符
        let threshold3 = get_adaptive_threshold("a");
        assert!(threshold3 >= 0.1 && threshold3 <= 0.9);
    }

    /// 测试Recency评分计算
    #[test]
    fn test_calculate_recency_score() {
        use chrono::{DateTime, Utc};
        
        // 测试最近访问（应该接近1.0）
        let now = Utc::now();
        let recent_time = now.to_rfc3339();
        let recency1 = calculate_recency_score(&recent_time, 0.1);
        assert!(recency1 > 0.9, "最近访问的recency应该 > 0.9, 实际: {}", recency1);
        
        // 测试1小时前访问（decay=0.1时应该约0.9）
        let one_hour_ago = (now - chrono::Duration::hours(1)).to_rfc3339();
        let recency2 = calculate_recency_score(&one_hour_ago, 0.1);
        assert!(recency2 > 0.85 && recency2 < 0.95, "1小时前访问的recency应该在0.85-0.95之间, 实际: {}", recency2);
        
        // 测试24小时前访问（decay=0.1时应该约0.08）
        let one_day_ago = (now - chrono::Duration::hours(24)).to_rfc3339();
        let recency3 = calculate_recency_score(&one_day_ago, 0.1);
        assert!(recency3 > 0.05 && recency3 < 0.15, "24小时前访问的recency应该在0.05-0.15之间, 实际: {}", recency3);
        
        // 测试无效时间格式（应该返回1.0作为默认值）
        let invalid_time = "invalid-time";
        let recency4 = calculate_recency_score(invalid_time, 0.1);
        assert_eq!(recency4, 1.0, "无效时间格式应该返回1.0");
    }

    /// 测试三维检索综合评分计算
    #[test]
    fn test_calculate_3d_score() {
        use chrono::{DateTime, Utc};
        
        // 测试完美记忆（高relevance、高importance、最近访问）
        let now = Utc::now();
        let recent_time = now.to_rfc3339();
        let score1 = calculate_3d_score(0.9, 0.9, &recent_time, 0.1);
        assert!(score1 > 0.7, "完美记忆的综合评分应该 > 0.7, 实际: {}", score1);
        
        // 测试低relevance记忆（即使importance和recency高，综合评分也应该低）
        let score2 = calculate_3d_score(0.1, 0.9, &recent_time, 0.1);
        assert!(score2 < 0.2, "低relevance记忆的综合评分应该 < 0.2, 实际: {}", score2);
        
        // 测试低importance记忆
        let score3 = calculate_3d_score(0.9, 0.1, &recent_time, 0.1);
        assert!(score3 < 0.2, "低importance记忆的综合评分应该 < 0.2, 实际: {}", score3);
        
        // 测试旧记忆（低recency）
        let old_time = (now - chrono::Duration::hours(48)).to_rfc3339();
        let score4 = calculate_3d_score(0.9, 0.9, &old_time, 0.1);
        assert!(score4 < 0.5, "旧记忆的综合评分应该 < 0.5, 实际: {}", score4);
        
        // 测试边界值（所有维度都是0）
        let score5 = calculate_3d_score(0.0, 0.0, &recent_time, 0.1);
        assert_eq!(score5, 0.0, "所有维度为0的综合评分应该为0.0");
        
        // 测试边界值（所有维度都是1.0）
        let score6 = calculate_3d_score(1.0, 1.0, &recent_time, 0.1);
        assert!(score6 > 0.9, "所有维度为1.0的综合评分应该 > 0.9, 实际: {}", score6);
    }

    /// 测试三维检索评分边界情况
    #[test]
    fn test_calculate_3d_score_boundaries() {
        use chrono::{DateTime, Utc};
        
        let now = Utc::now();
        let recent_time = now.to_rfc3339();
        
        // 测试超出范围的relevance（应该被clamp到[0.0, 1.0]）
        let score1 = calculate_3d_score(1.5, 0.5, &recent_time, 0.1);
        assert!(score1 <= 1.0, "超出范围的relevance应该被clamp, 实际: {}", score1);
        
        let score2 = calculate_3d_score(-0.5, 0.5, &recent_time, 0.1);
        assert!(score2 >= 0.0, "负值的relevance应该被clamp, 实际: {}", score2);
        
        // 测试超出范围的importance（应该被clamp到[0.0, 1.0]）
        let score3 = calculate_3d_score(0.5, 1.5, &recent_time, 0.1);
        assert!(score3 <= 1.0, "超出范围的importance应该被clamp, 实际: {}", score3);
        
        let score4 = calculate_3d_score(0.5, -0.5, &recent_time, 0.1);
        assert!(score4 >= 0.0, "负值的importance应该被clamp, 实际: {}", score4);
    }
}

