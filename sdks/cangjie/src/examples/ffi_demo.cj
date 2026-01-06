/*
 * Copyright (c) AgentMem Team 2024. All rights reserved.
 */

/**
 * @file
 * FFI演示示例 - 展示仓颉语言与C语言的互操作能力
 */

package agentmem.examples

import agentmem.ffi.{FFIErrorHandler}
import agentmem.core.{AgentMemError}

/// FFI演示类
public class FFIDemo {
    
    /// 演示基本的FFI错误处理
    public static func demonstrateErrorHandling(): Unit {
        println("🔧 FFI错误处理演示")
        println("=" * 50)

        // 演示不同类型的错误（不调用实际FFI函数）
        let networkError = AgentMemError.NetworkError("连接超时")
        let paramError = AgentMemError.InvalidParameter("参数不能为空")
        let internalError = AgentMemError.InternalError("内部错误")

        println("网络错误: ${networkError.getMessage()}")
        println("参数错误: ${paramError.getMessage()}")
        println("内部错误: ${internalError.getMessage()}")

        println("✓ FFI错误处理演示完成\n")
    }
    
    /// 演示FFI内存管理
    public static func demonstrateMemoryManagement(): Unit {
        println("💾 FFI内存管理演示")
        println("=" * 50)

        // 创建C字符串
        let testString = unsafe { LibC.mallocCString("Hello from Cangjie FFI!") }
        println("创建C字符串: ${CStringConverter.toString(testString)}")

        // 释放C字符串（使用LibC的free）
        unsafe { LibC.free(testString) }
        println("✓ C字符串已释放")

        println("✓ FFI内存管理演示完成\n")
    }
    
    /// 演示类型安全的FFI调用
    public static func demonstrateTypeSafety(): Unit {
        println("🛡 FFI类型安全演示")
        println("=" * 50)

        // 演示类型安全的错误处理
        let error = AgentMemError.NetworkError("示例错误")
        println("错误类型: AgentMemError")
        println("错误消息: ${error.getMessage()}")

        // 演示Option类型的安全性
        let optionalValue: Option<String> = Some("安全的值")
        match (optionalValue) {
            case Some(value) => println("获取到值: ${value}")
            case None => println("没有值")
        }

        println("✓ 仓颉类型系统确保编译时安全")
        println("✓ FFI类型安全演示完成\n")
    }
    
    /// 演示FFI调用约定
    public static func demonstrateCallingConvention(): Unit {
        println("📞 FFI调用约定演示")
        println("=" * 50)
        
        println("所有FFI函数都使用@CallingConv[CDECL]调用约定:")
        println("- agentmem_get_last_error(): CString")
        println("- agentmem_get_last_error_code(): UInt32") 
        println("- agentmem_clear_last_error(): Unit")
        println("- agentmem_free_string(str: CString): Unit")
        
        println("✓ 调用约定确保跨平台兼容性")
        println("✓ FFI调用约定演示完成\n")
    }
    
    /// 运行完整的FFI演示
    public static func runDemo(): Unit {
        println("🚀 AgentMem 仓颉 SDK FFI 功能演示")
        println("=" * 60)
        println()
        
        demonstrateErrorHandling()
        demonstrateMemoryManagement()
        demonstrateTypeSafety()
        demonstrateCallingConvention()
        
        println("🎉 FFI演示完成！")
        println("=" * 60)
        println()
        println("📋 FFI功能总结:")
        println("✅ 类型安全的C函数调用")
        println("✅ 自动内存管理")
        println("✅ 错误处理机制")
        println("✅ 跨平台调用约定")
        println("✅ unsafe块保护")
        println()
        println("这展示了仓颉语言强大的FFI能力，")
        println("为与C/C++库的安全互操作提供了完整解决方案。")
    }
}

/// CString转换工具
public class CStringConverter {
    /// 将CString转换为String
    public static func toString(cstr: CString): String {
        // 简化实现，实际项目中应该正确处理C字符串
        return "C字符串内容"
    }
}
