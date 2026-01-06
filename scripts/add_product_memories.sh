#!/bin/bash

##############################################################################
# 商品记忆批量写入脚本
# 功能: 批量写入10,000种商品的记忆数据
# 日期: 2025-11-07
##############################################################################

set -e

# 配置
API_BASE="${API_BASE:-http://localhost:8080}"
TOTAL_PRODUCTS=1000
BATCH_SIZE=100
TOTAL_BATCHES=$((TOTAL_PRODUCTS / BATCH_SIZE))

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 统计变量
SUCCESS_COUNT=0
FAIL_COUNT=0
START_TIME=$(date +%s)

echo -e "${BLUE}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║       AgentMem - 商品记忆批量写入系统                       ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${GREEN}📦 总商品数: ${TOTAL_PRODUCTS}${NC}"
echo -e "${GREEN}📊 批次大小: ${BATCH_SIZE}${NC}"
echo -e "${GREEN}🔢 总批次数: ${TOTAL_BATCHES}${NC}"
echo -e "${GREEN}🌐 API地址: ${API_BASE}${NC}"
echo ""

# 商品分类定义
declare -a CATEGORIES=(
    "电子产品:手机" "电子产品:电脑" "电子产品:平板" "电子产品:耳机" "电子产品:相机"
    "服装鞋帽:男装" "服装鞋帽:女装" "服装鞋帽:童装" "服装鞋帽:运动装" "服装鞋帽:鞋"
    "食品饮料:零食" "食品饮料:饮料" "食品饮料:生鲜" "食品饮料:酒水" "食品饮料:茶叶"
    "家居用品:家具" "家居用品:厨具" "家居用品:装饰" "家居用品:床品" "家居用品:收纳"
    "图书文娱:图书" "图书文娱:文具" "图书文娱:乐器" "图书文娱:玩具" "图书文娱:游戏"
    "运动户外:运动装备" "运动户外:户外用品" "运动户外:健身器材" "运动户外:球类" "运动户外:自行车"
    "美妆个护:护肤" "美妆个护:彩妆" "美妆个护:香水" "美妆个护:洗护" "美妆个护:美容工具"
    "母婴用品:奶粉" "母婴用品:尿不湿" "母婴用品:玩具" "母婴用品:童车" "母婴用品:童装"
    "汽车用品:汽车配件" "汽车用品:车载电器" "汽车用品:美容清洁" "汽车用品:座垫脚垫" "汽车用品:装饰"
    "数码配件:充电器" "数码配件:数据线" "数码配件:移动电源" "数码配件:保护壳" "数码配件:存储卡"
)

# 品牌列表
declare -a BRANDS=(
    "Apple" "Samsung" "Huawei" "Xiaomi" "OPPO" "vivo" "Lenovo" "Dell" "HP" "Asus"
    "Nike" "Adidas" "Puma" "Li-Ning" "Anta" "Uniqlo" "ZARA" "H&M" "MetersBonwe" "Semir"
    "Coca-Cola" "Pepsi" "Nestle" "Yili" "Mengniu" "Nongfu" "Wahaha" "Want-Want" "Master-Kong" "Uni-President"
    "IKEA" "MUJI" "Haier" "Midea" "Gree" "Supor" "Joyoung" "Bear" "Povos" "Philips"
)

# 价格区间
declare -a PRICE_RANGES=(
    "10-50" "50-100" "100-200" "200-500" "500-1000" 
    "1000-2000" "2000-5000" "5000-10000" "10000-20000" "20000-50000"
)

# 生成随机商品名称
generate_product_name() {
    local category=$1
    local brand=$2
    local id=$3
    
    local main_cat=$(echo $category | cut -d: -f1)
    local sub_cat=$(echo $category | cut -d: -f2)
    
    # 根据分类生成特定的商品名称
    case "$main_cat" in
        "电子产品")
            echo "${brand} ${sub_cat} 旗舰版 P${id}"
            ;;
        "服装鞋帽")
            echo "${brand} ${sub_cat} 时尚款 S${id}"
            ;;
        "食品饮料")
            echo "${brand} ${sub_cat} 精选装 F${id}"
            ;;
        "家居用品")
            echo "${brand} ${sub_cat} 豪华款 H${id}"
            ;;
        "图书文娱")
            echo "${brand} ${sub_cat} 经典版 B${id}"
            ;;
        "运动户外")
            echo "${brand} ${sub_cat} 专业款 R${id}"
            ;;
        "美妆个护")
            echo "${brand} ${sub_cat} 奢华系列 C${id}"
            ;;
        "母婴用品")
            echo "${brand} ${sub_cat} 安全款 M${id}"
            ;;
        "汽车用品")
            echo "${brand} ${sub_cat} 高端版 A${id}"
            ;;
        "数码配件")
            echo "${brand} ${sub_cat} 快充款 D${id}"
            ;;
        *)
            echo "${brand} 商品 G${id}"
            ;;
    esac
}

# 生成随机价格
generate_price() {
    local range=$1
    local min=$(echo $range | cut -d- -f1)
    local max=$(echo $range | cut -d- -f2)
    echo $((RANDOM % (max - min + 1) + min))
}

# 生成随机库存
generate_stock() {
    echo $((RANDOM % 1000 + 50))
}

# 添加单个商品记忆
add_product_memory() {
    local product_id=$1
    local product_name=$2
    local category=$3
    local brand=$4
    local price=$5
    local stock=$6
    local memory_type=$7
    local scope=$8
    
    local main_cat=$(echo $category | cut -d: -f1)
    local sub_cat=$(echo $category | cut -d: -f2)
    
    # 构建商品描述
    local content="商品ID: ${product_id}, 名称: ${product_name}, 分类: ${main_cat}>${sub_cat}, 品牌: ${brand}, 价格: ¥${price}, 库存: ${stock}件, 状态: 在售"
    
    # 构建metadata
    local metadata="{\"product_id\":\"${product_id}\",\"category\":\"${main_cat}\",\"subcategory\":\"${sub_cat}\",\"brand\":\"${brand}\",\"price\":\"${price}\",\"stock\":\"${stock}\",\"status\":\"active\",\"scope_type\":\"${scope}\"}"
    
    # 构建请求body
    local body=$(cat <<EOF
{
  "content": "${content}",
  "memory_type": "${memory_type}",
  "importance": 0.8,
  "metadata": ${metadata}
}
EOF
)
    
    # 发送请求
    local response=$(curl -s -w "\n%{http_code}" -X POST \
        "${API_BASE}/api/v1/memories" \
        -H "Content-Type: application/json" \
        -d "${body}")
    
    local http_code=$(echo "$response" | tail -n1)
    
    if [ "$http_code" = "200" ] || [ "$http_code" = "201" ]; then
        return 0
    else
        echo -e "${RED}✗${NC} 失败: ${product_name} (HTTP ${http_code})" >&2
        return 1
    fi
}

# 添加客户浏览记忆
add_customer_view_memory() {
    local product_id=$1
    local user_id=$2
    
    local content="用户${user_id}浏览了商品${product_id}，停留时间$((RANDOM % 300 + 30))秒，查看了产品详情"
    
    local metadata="{\"product_id\":\"${product_id}\",\"action\":\"view\",\"duration\":\"$((RANDOM % 300 + 30))\",\"scope_type\":\"user\",\"user_id\":\"${user_id}\"}"
    
    local body=$(cat <<EOF
{
  "content": "${content}",
  "memory_type": "Episodic",
  "importance": 0.6,
  "user_id": "${user_id}",
  "metadata": ${metadata}
}
EOF
)
    
    curl -s -X POST "${API_BASE}/api/v1/memories" \
        -H "Content-Type: application/json" \
        -d "${body}" > /dev/null
}

# 添加销售分析记忆
add_sales_analysis_memory() {
    local product_id=$1
    local agent_id=$2
    
    local sales=$((RANDOM % 500 + 10))
    local growth=$((RANDOM % 100 - 20))
    
    local content="商品${product_id}在过去7天销售${sales}件，环比增长${growth}%"
    
    local metadata="{\"product_id\":\"${product_id}\",\"analysis_type\":\"sales_trend\",\"period\":\"7days\",\"sales_count\":\"${sales}\",\"growth_rate\":\"${growth}\",\"scope_type\":\"agent\"}"
    
    local body=$(cat <<EOF
{
  "content": "${content}",
  "memory_type": "Episodic",
  "importance": 0.7,
  "agent_id": "${agent_id}",
  "metadata": ${metadata}
}
EOF
)
    
    curl -s -X POST "${API_BASE}/api/v1/memories" \
        -H "Content-Type: application/json" \
        -d "${body}" > /dev/null
}

# 批量处理
echo -e "${YELLOW}🚀 开始批量写入...${NC}"
echo ""

for batch in $(seq 1 $TOTAL_BATCHES); do
    echo -ne "${BLUE}批次 ${batch}/${TOTAL_BATCHES}:${NC} "
    
    batch_success=0
    batch_fail=0
    
    for i in $(seq 1 $BATCH_SIZE); do
        # 计算全局商品ID
        product_num=$(((batch - 1) * BATCH_SIZE + i))
        product_id=$(printf "P%06d" $product_num)
        
        # 随机选择分类、品牌、价格
        category_idx=$((RANDOM % ${#CATEGORIES[@]}))
        brand_idx=$((RANDOM % ${#BRANDS[@]}))
        price_range_idx=$((RANDOM % ${#PRICE_RANGES[@]}))
        
        category="${CATEGORIES[$category_idx]}"
        brand="${BRANDS[$brand_idx]}"
        price_range="${PRICE_RANGES[$price_range_idx]}"
        
        # 生成商品数据
        product_name=$(generate_product_name "$category" "$brand" "$product_num")
        price=$(generate_price "$price_range")
        stock=$(generate_stock)
        
        # 添加商品基础信息（Global Scope）
        if add_product_memory "$product_id" "$product_name" "$category" "$brand" "$price" "$stock" "Semantic" "global"; then
            ((batch_success++))
            ((SUCCESS_COUNT++))
        else
            ((batch_fail++))
            ((FAIL_COUNT++))
        fi
        
        # 10%的商品添加客户浏览记忆（User Scope）
        if [ $((RANDOM % 10)) -eq 0 ]; then
            user_num=$((RANDOM % 100 + 1))
            user_id=$(printf "user-%03d" $user_num)
            add_customer_view_memory "$product_id" "$user_id"
        fi
        
        # 5%的商品添加销售分析记忆（Agent Scope）
        if [ $((RANDOM % 20)) -eq 0 ]; then
            agent_id="agent-sales-analyst"
            add_sales_analysis_memory "$product_id" "$agent_id"
        fi
        
        # 进度显示
        if [ $((i % 10)) -eq 0 ]; then
            echo -n "."
        fi
    done
    
    echo -e " ${GREEN}✓${NC} 成功: ${batch_success}, ${RED}✗${NC} 失败: ${batch_fail}"
    
    # 每10批次显示统计
    if [ $((batch % 10)) -eq 0 ]; then
        elapsed=$(($(date +%s) - START_TIME))
        rate=$((SUCCESS_COUNT / elapsed))
        echo -e "${YELLOW}  ⏱ 已耗时: ${elapsed}秒, 速率: ${rate}条/秒, 总成功: ${SUCCESS_COUNT}${NC}"
    fi
    
    # 短暂延迟，避免API限流
    sleep 0.1
done

# 最终统计
END_TIME=$(date +%s)
ELAPSED=$((END_TIME - START_TIME))
RATE=$((SUCCESS_COUNT / ELAPSED))

echo ""
echo -e "${BLUE}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                   批量写入完成                               ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${GREEN}✅ 成功写入: ${SUCCESS_COUNT} 条${NC}"
echo -e "${RED}❌ 写入失败: ${FAIL_COUNT} 条${NC}"
echo -e "${YELLOW}📊 成功率: $(awk "BEGIN {printf \"%.2f\", ${SUCCESS_COUNT}*100/(${SUCCESS_COUNT}+${FAIL_COUNT})}")%${NC}"
echo -e "${YELLOW}⏱ 总耗时: ${ELAPSED} 秒${NC}"
echo -e "${YELLOW}⚡ 写入速率: ${RATE} 条/秒${NC}"
echo ""

# 验证数据
echo -e "${BLUE}🔍 验证数据...${NC}"
total_memories=$(curl -s "${API_BASE}/api/v1/memories/search?query=商品&limit=1" | jq -r '.total // 0' 2>/dev/null || echo "0")
echo -e "${GREEN}📦 数据库中商品记忆总数: ${total_memories}${NC}"
echo ""

# 搜索测试
echo -e "${BLUE}🧪 搜索测试...${NC}"
echo -e "${YELLOW}测试1: 搜索'Apple'品牌${NC}"
apple_count=$(curl -s "${API_BASE}/api/v1/memories/search?query=Apple&limit=100" | jq -r '.memories | length' 2>/dev/null || echo "0")
echo -e "  结果: ${apple_count} 条记忆"

echo -e "${YELLOW}测试2: 搜索'电子产品'分类${NC}"
electronics_count=$(curl -s "${API_BASE}/api/v1/memories/search?query=电子产品&limit=100" | jq -r '.memories | length' 2>/dev/null || echo "0")
echo -e "  结果: ${electronics_count} 条记忆"

echo -e "${YELLOW}测试3: 搜索'手机'子分类${NC}"
phone_count=$(curl -s "${API_BASE}/api/v1/memories/search?query=手机&limit=100" | jq -r '.memories | length' 2>/dev/null || echo "0")
echo -e "  结果: ${phone_count} 条记忆"

echo ""
echo -e "${GREEN}✅ 批量写入完成！${NC}"
echo ""
echo -e "${BLUE}📄 查看详细设计: PRODUCT_MEMORY_DESIGN.md${NC}"
echo ""

