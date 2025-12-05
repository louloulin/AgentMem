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
}

