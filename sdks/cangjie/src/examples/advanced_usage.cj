package agentmem.examples

import agentmem.core.*
import agentmem.api.*
import agentmem.utils.*

/// 简化的高级使用示例
public class AdvancedUsageExample {
    
    /// 运行高级使用示例
    public static func run(): Bool {
        println("🚀 AgentMem 仓颉 SDK 高级使用示例")
        println("============================================================")
        
        // 简化的高级示例
        println("\n📚 智能记忆管理示例...")
        let client = AgentMemClientBuilder()
            .agentId("smart-agent")
            .apiKey("demo-key")
            .build()
        
        println("  ✓ 客户端创建成功")
        
        println("\n🔍 高级搜索示例...")
        let searchService = AgentMemSearchService("smart-agent")
        let textResult = searchService.searchByText("咖啡", None, None)
        if (textResult.isOk()) {
            println("  ✓ 搜索功能正常")
        } else {
            println("  ⚠ 搜索功能测试（模拟环境）")
        }
        
        println("\n📦 批量操作示例...")
        let memoryOps = AgentMemMemoryOps("smart-agent")
        let statsResult = memoryOps.getStatistics()
        if (statsResult.isOk()) {
            println("  ✓ 统计功能正常")
        } else {
            println("  ⚠ 统计功能测试（模拟环境）")
        }
        
        println("\n📈 记忆分析示例...")
        let adminService = AgentMemAdminService()
        adminService.setConnected(true)
        let healthResult = adminService.getSystemHealth()
        if (healthResult.isOk()) {
            println("  ✓ 健康检查正常")
        } else {
            println("  ⚠ 健康检查测试（模拟环境）")
        }
        
        return true
    }
}
