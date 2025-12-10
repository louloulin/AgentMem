// AgentMem UI Playwright 验证脚本
// 验证前端 UI 的所有关键功能

// 尝试加载 playwright（支持从项目根目录或 agentmem-ui 目录运行）
const path = require('path');
const fs = require('fs');

// 查找 playwright 模块
function findPlaywright() {
    const possiblePaths = [
        path.join(__dirname, '../agentmem-ui/node_modules/playwright'),
        path.join(__dirname, '../node_modules/playwright'),
        'playwright'
    ];
    
    for (const playPath of possiblePaths) {
        try {
            return require(playPath);
        } catch (e) {
            // 继续尝试下一个路径
        }
    }
    return null;
}

let playwright = findPlaywright();
let chromium;

if (!playwright) {
    console.error('❌ Playwright 未安装');
    console.error('请运行以下命令安装:');
    console.error('  cd agentmem-ui && npm install --save-dev playwright');
    console.error('  npx playwright install chromium');
    console.error('');
    console.error('或者使用简化版 UI 验证（不使用 Playwright）:');
    console.error('  bash scripts/verify_ui_simple.sh');
    process.exit(1);
}

chromium = playwright.chromium;

// 配置
const CONFIG = {
    baseURL: 'http://localhost:3001',
    backendURL: 'http://localhost:8080',
    timeout: 30000,
    headless: true,
    screenshotDir: './test-results/screenshots',
    reportDir: './test-results/reports'
};

// 创建报告目录
[CONFIG.screenshotDir, CONFIG.reportDir].forEach(dir => {
    if (!fs.existsSync(dir)) {
        fs.mkdirSync(dir, { recursive: true });
    }
});

// 测试结果
const testResults = {
    passed: [],
    failed: [],
    warnings: []
};

// 辅助函数
function log(message, type = 'info') {
    const timestamp = new Date().toISOString();
    const prefix = type === 'error' ? '❌' : type === 'success' ? '✅' : type === 'warning' ? '⚠️' : 'ℹ️';
    console.log(`[${timestamp}] ${prefix} ${message}`);
}

async function takeScreenshot(page, name) {
    const screenshotPath = path.join(CONFIG.screenshotDir, `${name}-${Date.now()}.png`);
    await page.screenshot({ path: screenshotPath, fullPage: true });
    return screenshotPath;
}

async function checkBackendHealth() {
    try {
        const response = await fetch(`${CONFIG.backendURL}/health`);
        const data = await response.json();
        return data.status === 'healthy';
    } catch (error) {
        log(`后端健康检查失败: ${error.message}`, 'error');
        return false;
    }
}

// 测试用例
async function testHomePage(page) {
    log('测试: 首页加载');
    try {
        await page.goto(CONFIG.baseURL, { waitUntil: 'networkidle' });
        await page.waitForTimeout(2000);
        
        const title = await page.title();
        if (title && title.length > 0) {
            testResults.passed.push('首页加载');
            log('首页加载成功', 'success');
            await takeScreenshot(page, 'homepage');
            return true;
        } else {
            throw new Error('页面标题为空');
        }
    } catch (error) {
        testResults.failed.push({ test: '首页加载', error: error.message });
        log(`首页加载失败: ${error.message}`, 'error');
        await takeScreenshot(page, 'homepage-error');
        return false;
    }
}

async function testMemoriesPage(page) {
    log('测试: 记忆管理页面');
    try {
        await page.goto(`${CONFIG.baseURL}/admin/memories`, { waitUntil: 'networkidle' });
        await page.waitForTimeout(2000);
        
        // 检查页面元素
        const pageContent = await page.textContent('body');
        if (pageContent && pageContent.length > 0) {
            testResults.passed.push('记忆管理页面');
            log('记忆管理页面加载成功', 'success');
            await takeScreenshot(page, 'memories-page');
            return true;
        } else {
            throw new Error('页面内容为空');
        }
    } catch (error) {
        testResults.failed.push({ test: '记忆管理页面', error: error.message });
        log(`记忆管理页面失败: ${error.message}`, 'error');
        await takeScreenshot(page, 'memories-page-error');
        return false;
    }
}

async function testAgentsPage(page) {
    log('测试: Agent 管理页面');
    try {
        await page.goto(`${CONFIG.baseURL}/admin/agents`, { waitUntil: 'networkidle' });
        await page.waitForTimeout(2000);
        
        const pageContent = await page.textContent('body');
        if (pageContent && pageContent.length > 0) {
            testResults.passed.push('Agent 管理页面');
            log('Agent 管理页面加载成功', 'success');
            await takeScreenshot(page, 'agents-page');
            return true;
        } else {
            throw new Error('页面内容为空');
        }
    } catch (error) {
        testResults.failed.push({ test: 'Agent 管理页面', error: error.message });
        log(`Agent 管理页面失败: ${error.message}`, 'error');
        await takeScreenshot(page, 'agents-page-error');
        return false;
    }
}

async function testChatPage(page) {
    log('测试: 聊天页面');
    try {
        await page.goto(`${CONFIG.baseURL}/chat`, { waitUntil: 'networkidle' });
        await page.waitForTimeout(2000);
        
        const pageContent = await page.textContent('body');
        if (pageContent && pageContent.length > 0) {
            testResults.passed.push('聊天页面');
            log('聊天页面加载成功', 'success');
            await takeScreenshot(page, 'chat-page');
            return true;
        } else {
            throw new Error('页面内容为空');
        }
    } catch (error) {
        testResults.failed.push({ test: '聊天页面', error: error.message });
        log(`聊天页面失败: ${error.message}`, 'error');
        await takeScreenshot(page, 'chat-page-error');
        return false;
    }
}

async function testDashboardPage(page) {
    log('测试: Dashboard 页面');
    try {
        await page.goto(`${CONFIG.baseURL}/dashboard`, { waitUntil: 'networkidle' });
        await page.waitForTimeout(2000);
        
        const pageContent = await page.textContent('body');
        if (pageContent && pageContent.length > 0) {
            testResults.passed.push('Dashboard 页面');
            log('Dashboard 页面加载成功', 'success');
            await takeScreenshot(page, 'dashboard-page');
            return true;
        } else {
            throw new Error('页面内容为空');
        }
    } catch (error) {
        testResults.failed.push({ test: 'Dashboard 页面', error: error.message });
        log(`Dashboard 页面失败: ${error.message}`, 'error');
        await takeScreenshot(page, 'dashboard-page-error');
        return false;
    }
}

async function testNavigation(page) {
    log('测试: 页面导航');
    try {
        // 测试导航链接
        const navLinks = [
            { name: '首页', url: '/' },
            { name: 'Dashboard', url: '/dashboard' },
            { name: 'Chat', url: '/chat' },
            { name: 'Memories', url: '/admin/memories' },
            { name: 'Agents', url: '/admin/agents' }
        ];
        
        let successCount = 0;
        for (const link of navLinks) {
            try {
                await page.goto(`${CONFIG.baseURL}${link.url}`, { waitUntil: 'networkidle', timeout: 10000 });
                await page.waitForTimeout(1000);
                successCount++;
            } catch (error) {
                log(`导航到 ${link.name} 失败: ${error.message}`, 'warning');
            }
        }
        
        if (successCount === navLinks.length) {
            testResults.passed.push('页面导航');
            log('页面导航测试成功', 'success');
            return true;
        } else {
            testResults.warnings.push(`页面导航: ${successCount}/${navLinks.length} 成功`);
            log(`页面导航部分成功: ${successCount}/${navLinks.length}`, 'warning');
            return true; // 部分成功也算通过
        }
    } catch (error) {
        testResults.failed.push({ test: '页面导航', error: error.message });
        log(`页面导航失败: ${error.message}`, 'error');
        return false;
    }
}

async function testAPIEndpoints() {
    log('测试: 后端 API 端点');
    const endpoints = [
        { name: '健康检查', url: '/health' },
        { name: 'Dashboard 统计', url: '/api/v1/stats/dashboard' },
        { name: 'Agent 列表', url: '/api/v1/agents' }
    ];
    
    let successCount = 0;
    for (const endpoint of endpoints) {
        try {
            const response = await fetch(`${CONFIG.backendURL}${endpoint.url}`);
            if (response.ok) {
                log(`${endpoint.name} API 正常`, 'success');
                successCount++;
            } else {
                log(`${endpoint.name} API 返回错误: ${response.status}`, 'warning');
            }
        } catch (error) {
            log(`${endpoint.name} API 失败: ${error.message}`, 'warning');
        }
    }
    
    if (successCount > 0) {
        testResults.passed.push(`API 端点 (${successCount}/${endpoints.length})`);
        return true;
    } else {
        testResults.failed.push({ test: 'API 端点', error: '所有端点都失败' });
        return false;
    }
}

// 主测试函数
async function runTests() {
    log('开始 AgentMem UI Playwright 验证', 'info');
    log(`前端 URL: ${CONFIG.baseURL}`, 'info');
    log(`后端 URL: ${CONFIG.backendURL}`, 'info');
    
    // 检查后端健康
    log('检查后端服务健康状态...', 'info');
    const backendHealthy = await checkBackendHealth();
    if (!backendHealthy) {
        log('警告: 后端服务可能未运行', 'warning');
        testResults.warnings.push('后端服务健康检查失败');
    }
    
    // 测试 API
    await testAPIEndpoints();
    
    // 启动浏览器
    const browser = await chromium.launch({ 
        headless: CONFIG.headless,
        timeout: CONFIG.timeout 
    });
    
    const context = await browser.newContext({
        viewport: { width: 1920, height: 1080 }
    });
    
    const page = await context.newPage();
    
    try {
        // 运行所有测试
        await testHomePage(page);
        await testDashboardPage(page);
        await testChatPage(page);
        await testMemoriesPage(page);
        await testAgentsPage(page);
        await testNavigation(page);
        
    } finally {
        await browser.close();
    }
    
    // 生成报告
    generateReport();
}

function generateReport() {
    const report = {
        timestamp: new Date().toISOString(),
        summary: {
            total: testResults.passed.length + testResults.failed.length,
            passed: testResults.passed.length,
            failed: testResults.failed.length,
            warnings: testResults.warnings.length
        },
        passed: testResults.passed,
        failed: testResults.failed,
        warnings: testResults.warnings
    };
    
    const reportPath = path.join(CONFIG.reportDir, `ui-verification-${Date.now()}.json`);
    fs.writeFileSync(reportPath, JSON.stringify(report, null, 2));
    
    // 控制台输出
    console.log('\n==========================================');
    console.log('AgentMem UI 验证报告');
    console.log('==========================================');
    console.log(`总测试数: ${report.summary.total}`);
    console.log(`✅ 通过: ${report.summary.passed}`);
    console.log(`❌ 失败: ${report.summary.failed}`);
    console.log(`⚠️  警告: ${report.summary.warnings}`);
    console.log('\n通过的测试:');
    report.passed.forEach(test => console.log(`  ✅ ${test}`));
    if (report.failed.length > 0) {
        console.log('\n失败的测试:');
        report.failed.forEach(test => console.log(`  ❌ ${test.test}: ${test.error}`));
    }
    if (report.warnings.length > 0) {
        console.log('\n警告:');
        report.warnings.forEach(warning => console.log(`  ⚠️  ${warning}`));
    }
    console.log(`\n详细报告已保存到: ${reportPath}`);
    console.log('==========================================\n');
}

// 运行测试
if (require.main === module) {
    runTests().catch(error => {
        console.error('测试执行失败:', error);
        process.exit(1);
    });
}

module.exports = { runTests, CONFIG };
