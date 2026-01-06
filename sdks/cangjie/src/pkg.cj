/*
 * Copyright (c) AgentMem Team 2024. All rights reserved.
 */

/**
 * AgentMem 仓颉 SDK 静态库定义
 *
 * 这个文件定义了AgentMem SDK静态库的包结构和导出的公共API。
 * 提供完整的企业级智能记忆管理平台的仓颉语言接口。
 *
 * @version 1.0.0
 * @author AgentMem Team
 * @license MIT
 */

package agentmem

// 导出核心类型和功能
public import agentmem.core.*
public import agentmem.utils.*

// 导出API功能
public import agentmem.api.*

// 导出FFI功能
public import agentmem.ffi.*

// 导出示例代码（仅用于演示，生产环境可选）
public import agentmem.examples.*

// 导出测试套件（用于验证和基准测试）
public import agentmem.tests.*
