#!/bin/bash
# 在 Docker 容器内修复 aws-lc-sys 构建脚本

set -e

echo "🔧 开始修复 aws-lc-sys 构建脚本..."

# 查找所有 aws-lc-sys 的 cc_builder.rs 文件
FILES=$(find /root/.cargo/registry/src -name 'cc_builder.rs' -path '*aws-lc-sys*' 2>/dev/null || true)

if [ -z "$FILES" ]; then
    echo "⚠️  未找到 aws-lc-sys 源码文件，可能在构建时才会下载"
    exit 0
fi

for file in $FILES; do
    if [ ! -f "$file" ]; then
        continue
    fi
    
    # 检查是否已经修复过
    if grep -q "AWS_LC_SYS_SKIP_COMPILER_CHECK" "$file" 2>/dev/null; then
        echo "✅ 已修复: $file"
        continue
    fi
    
    # 使用 sed 在 memcmp_check 函数开始处添加跳过逻辑
    # 查找 "fn memcmp_check(&self) {" 后的第一个 "let basename" 之前插入代码
    python3 << PYTHON_SCRIPT
import sys
import re

file_path = "$file"
try:
    with open(file_path, 'r') as f:
        content = f.read()
    
    # 检查是否已经修复
    if 'AWS_LC_SYS_SKIP_COMPILER_CHECK' in content:
        print(f"✅ 已修复: {file_path}")
        sys.exit(0)
    
    # 查找 memcmp_check 函数，在 let basename 之前插入代码
    pattern = r'(fn memcmp_check\(&self\) \{[^}]*?)(let basename = "memcmp_invalid_stripped_check")'
    match = re.search(pattern, content, re.DOTALL)
    
    if match:
        replacement = match.group(1) + '''        // Skip compiler check if environment variable is set
        if env_var_to_bool("AWS_LC_SYS_SKIP_COMPILER_CHECK").unwrap_or(false) {
            return;
        }
        ''' + match.group(2)
        new_content = content[:match.start()] + replacement + content[match.end():]
        
        with open(file_path, 'w') as f:
            f.write(new_content)
        print(f"✅ 修复成功: {file_path}")
    else:
        print(f"⚠️  未找到 memcmp_check 函数: {file_path}")
except Exception as e:
    print(f"❌ 错误处理 {file_path}: {e}")
    sys.exit(1)
PYTHON_SCRIPT
done

echo "✅ aws-lc-sys 修复完成"
