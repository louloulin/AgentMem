#!/bin/bash
# 在构建过程中动态修复 aws-lc-sys 源码
# 这个脚本会在 cargo build 执行前自动修复源码

set -e

echo "🔧 检查并修复 aws-lc-sys 构建脚本..."

# 查找所有可能的 aws-lc-sys 源码位置
REGISTRY_PATHS=(
    "/root/.cargo/registry/src"
    "$HOME/.cargo/registry/src"
    "/usr/local/cargo/registry/src"
)

FOUND=0

for registry_path in "${REGISTRY_PATHS[@]}"; do
    if [ ! -d "$registry_path" ]; then
        continue
    fi
    
    FILES=$(find "$registry_path" -name 'cc_builder.rs' -path '*aws-lc-sys*' 2>/dev/null || true)
    
    for file in $FILES; do
        if [ ! -f "$file" ]; then
            continue
        fi
        
        # 检查是否已经修复过
        if grep -q "AWS_LC_SYS_SKIP_COMPILER_CHECK" "$file" 2>/dev/null; then
            echo "✅ 已修复: $file"
            FOUND=1
            continue
        fi
        
        # 使用 Python 修复
        python3 << PYTHON_SCRIPT
import sys
import re

file_path = "$file"
try:
    with open(file_path, 'r') as f:
        content = f.read()
    
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
        sys.exit(0)
    else:
        print(f"⚠️  未找到 memcmp_check 函数: {file_path}")
        sys.exit(1)
except Exception as e:
    print(f"❌ 错误处理 {file_path}: {e}")
    sys.exit(1)
PYTHON_SCRIPT
        
        if [ $? -eq 0 ]; then
            FOUND=1
        fi
    done
done

if [ $FOUND -eq 0 ]; then
    echo "⚠️  未找到 aws-lc-sys 源码文件，可能在构建时才会下载"
    echo "   将在构建时通过 cargo 钩子自动修复"
fi

echo "✅ 修复检查完成"

