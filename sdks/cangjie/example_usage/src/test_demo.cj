/*
 * Copyright (c) AgentMem Team 2024. All rights reserved.
 */

/**
 * @file
 * AgentMem仓颉SDK测试演示 - 运行完整的测试套件
 */

package test_demo

import agentmem.tests.*

/// 测试演示主函数
main() {
    println("🧪 AgentMem 仓颉 SDK 测试演示")
    println("=" * 50)
    
    // 运行快速验证测试
    println("\n🚀 第一步：快速验证测试")
    println("-" * 30)
    if (TestRunner.runQuickTests()) {
        println("✅ 快速验证测试通过")
    } else {
        println("❌ 快速验证测试失败")
        return
    }
    
    // 运行完整测试套件
    println("\n🔬 第二步：完整测试套件")
    println("-" * 30)
    let allTestsPassed = TestRunner.runAllTests()
    
    if (allTestsPassed) {
        println("\n🎉 所有测试通过！")
        println("AgentMem仓颉SDK已准备就绪！")
        
        // 生成性能基准报告
        println("\n📊 第三步：性能基准报告")
        println("-" * 30)
        TestRunner.generatePerformanceBenchmark()
        
    } else {
        println("\n⚠️  部分测试失败")
        println("请检查上述错误信息并进行修复")
    }
    
    println("\n" + "=" * 50)
    println("测试演示完成")
}
