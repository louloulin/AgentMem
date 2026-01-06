#!/bin/bash

# AgentMem Docker 启动脚本
# 用于快速启动和管理 AgentMem 向量数据库生态系统

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 日志函数
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 检查 Docker 和 Docker Compose
check_dependencies() {
    log_info "检查依赖..."
    
    if ! command -v docker &> /dev/null; then
        log_error "Docker 未安装，请先安装 Docker"
        exit 1
    fi
    
    if ! command -v docker-compose &> /dev/null; then
        log_error "Docker Compose 未安装，请先安装 Docker Compose"
        exit 1
    fi
    
    log_success "依赖检查通过"
}

# 创建必要的目录
create_directories() {
    log_info "创建数据目录..."
    
    mkdir -p data/{milvus,chroma,qdrant,elasticsearch,postgres,redis,prometheus,grafana}
    mkdir -p monitoring/grafana/{dashboards,datasources}
    mkdir -p logs
    
    log_success "目录创建完成"
}

# 复制环境配置
setup_environment() {
    log_info "设置环境配置..."
    
    if [ ! -f .env ]; then
        if [ -f .env.example ]; then
            cp .env.example .env
            log_success "已复制 .env.example 到 .env"
            log_warning "请编辑 .env 文件以配置您的环境"
        else
            log_error ".env.example 文件不存在"
            exit 1
        fi
    else
        log_info ".env 文件已存在，跳过复制"
    fi
}

# 启动服务
start_services() {
    local profile=${1:-"all"}
    
    log_info "启动服务 (profile: $profile)..."
    
    case $profile in
        "minimal")
            log_info "启动最小配置 (Chroma + Redis)..."
            docker-compose up -d chroma redis
            ;;
        "vector-only")
            log_info "启动向量数据库..."
            docker-compose up -d chroma qdrant
            ;;
        "milvus")
            log_info "启动 Milvus 生态系统..."
            docker-compose up -d etcd minio milvus
            ;;
        "monitoring")
            log_info "启动监控服务..."
            docker-compose up -d prometheus grafana jaeger
            ;;
        "all")
            log_info "启动所有服务..."
            docker-compose up -d
            ;;
        *)
            log_error "未知的 profile: $profile"
            log_info "可用的 profiles: minimal, vector-only, milvus, monitoring, all"
            exit 1
            ;;
    esac
    
    log_success "服务启动完成"
}

# 检查服务状态
check_services() {
    log_info "检查服务状态..."
    
    docker-compose ps
    
    log_info "检查服务健康状态..."
    
    # 等待服务启动
    sleep 10
    
    # 检查各个服务
    services=(
        "http://localhost:8000/api/v1/heartbeat|Chroma"
        "http://localhost:6333/health|Qdrant"
        "http://localhost:9200/_cluster/health|Elasticsearch"
        "http://localhost:9091/healthz|Milvus"
        "http://localhost:9090/-/healthy|Prometheus"
        "http://localhost:3000/api/health|Grafana"
    )
    
    for service in "${services[@]}"; do
        IFS='|' read -r url name <<< "$service"
        if curl -f -s "$url" > /dev/null 2>&1; then
            log_success "$name 服务健康"
        else
            log_warning "$name 服务可能未就绪或未启动"
        fi
    done
}

# 显示服务信息
show_services() {
    log_info "服务访问信息:"
    echo ""
    echo "📊 向量数据库:"
    echo "  • Chroma:        http://localhost:8000"
    echo "  • Qdrant:        http://localhost:6333"
    echo "  • Weaviate:      http://localhost:8080"
    echo "  • Milvus:        http://localhost:19530 (gRPC), http://localhost:9091 (HTTP)"
    echo "  • Elasticsearch: http://localhost:9200"
    echo ""
    echo "🗄️ 传统数据库:"
    echo "  • PostgreSQL:    localhost:5432"
    echo "  • Redis:         localhost:6379"
    echo ""
    echo "📈 监控服务:"
    echo "  • Prometheus:    http://localhost:9090"
    echo "  • Grafana:       http://localhost:3000 (admin/admin)"
    echo "  • Jaeger:        http://localhost:16686"
    echo ""
    echo "💾 对象存储:"
    echo "  • MinIO:         http://localhost:9000 (minioadmin/minioadmin)"
    echo "  • MinIO Console: http://localhost:9001"
    echo ""
}

# 显示日志
show_logs() {
    local service=${1:-""}
    
    if [ -z "$service" ]; then
        log_info "显示所有服务日志..."
        docker-compose logs -f
    else
        log_info "显示 $service 服务日志..."
        docker-compose logs -f "$service"
    fi
}

# 停止服务
stop_services() {
    log_info "停止服务..."
    docker-compose down
    log_success "服务已停止"
}

# 清理数据
cleanup_data() {
    log_warning "这将删除所有数据，确定要继续吗? (y/N)"
    read -r response
    if [[ "$response" =~ ^([yY][eE][sS]|[yY])$ ]]; then
        log_info "清理数据..."
        docker-compose down -v
        sudo rm -rf data/*
        log_success "数据清理完成"
    else
        log_info "取消清理操作"
    fi
}

# 显示帮助信息
show_help() {
    echo "AgentMem Docker 管理脚本"
    echo ""
    echo "用法: $0 [命令] [选项]"
    echo ""
    echo "命令:"
    echo "  start [profile]    启动服务 (profiles: minimal, vector-only, milvus, monitoring, all)"
    echo "  stop              停止服务"
    echo "  restart [profile] 重启服务"
    echo "  status            显示服务状态"
    echo "  logs [service]    显示日志"
    echo "  info              显示服务访问信息"
    echo "  cleanup           清理所有数据"
    echo "  help              显示此帮助信息"
    echo ""
    echo "示例:"
    echo "  $0 start minimal     # 启动最小配置"
    echo "  $0 start milvus      # 启动 Milvus"
    echo "  $0 logs chroma       # 显示 Chroma 日志"
    echo "  $0 status            # 检查服务状态"
    echo ""
}

# 主函数
main() {
    local command=${1:-"help"}
    
    case $command in
        "start")
            check_dependencies
            create_directories
            setup_environment
            start_services "$2"
            sleep 5
            check_services
            show_services
            ;;
        "stop")
            stop_services
            ;;
        "restart")
            stop_services
            sleep 2
            start_services "$2"
            sleep 5
            check_services
            ;;
        "status")
            check_services
            ;;
        "logs")
            show_logs "$2"
            ;;
        "info")
            show_services
            ;;
        "cleanup")
            cleanup_data
            ;;
        "help"|*)
            show_help
            ;;
    esac
}

# 切换到脚本目录
cd "$(dirname "$0")/.."

# 执行主函数
main "$@"
