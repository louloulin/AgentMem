# AgentMem AWS 部署 Terraform 配置
# 企业级 AWS 基础设施即代码

terraform {
  required_version = ">= 1.0"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "~> 2.20"
    }
    helm = {
      source  = "hashicorp/helm"
      version = "~> 2.10"
    }
  }
}

# ================================
# 变量定义
# ================================
variable "cluster_name" {
  description = "EKS cluster name"
  type        = string
  default     = "agentmem-cluster"
}

variable "region" {
  description = "AWS region"
  type        = string
  default     = "us-west-2"
}

variable "environment" {
  description = "Environment name"
  type        = string
  default     = "production"
}

variable "node_instance_type" {
  description = "EC2 instance type for worker nodes"
  type        = string
  default     = "m5.xlarge"
}

variable "min_nodes" {
  description = "Minimum number of worker nodes"
  type        = number
  default     = 3
}

variable "max_nodes" {
  description = "Maximum number of worker nodes"
  type        = number
  default     = 10
}

variable "desired_nodes" {
  description = "Desired number of worker nodes"
  type        = number
  default     = 3
}

# ================================
# 数据源
# ================================
data "aws_availability_zones" "available" {
  state = "available"
}

data "aws_caller_identity" "current" {}

# ================================
# VPC 配置
# ================================
module "vpc" {
  source = "terraform-aws-modules/vpc/aws"
  version = "~> 5.0"

  name = "${var.cluster_name}-vpc"
  cidr = "10.0.0.0/16"

  azs             = slice(data.aws_availability_zones.available.names, 0, 3)
  private_subnets = ["10.0.1.0/24", "10.0.2.0/24", "10.0.3.0/24"]
  public_subnets  = ["10.0.101.0/24", "10.0.102.0/24", "10.0.103.0/24"]

  enable_nat_gateway = true
  enable_vpn_gateway = false
  enable_dns_hostnames = true
  enable_dns_support = true

  public_subnet_tags = {
    "kubernetes.io/role/elb" = "1"
  }

  private_subnet_tags = {
    "kubernetes.io/role/internal-elb" = "1"
  }

  tags = {
    Environment = var.environment
    Project     = "AgentMem"
    Terraform   = "true"
  }
}

# ================================
# EKS 集群
# ================================
module "eks" {
  source = "terraform-aws-modules/eks/aws"
  version = "~> 19.0"

  cluster_name    = var.cluster_name
  cluster_version = "1.28"

  vpc_id                         = module.vpc.vpc_id
  subnet_ids                     = module.vpc.private_subnets
  cluster_endpoint_public_access = true

  # EKS 管理的节点组
  eks_managed_node_groups = {
    agentmem_nodes = {
      name = "agentmem-nodes"

      instance_types = [var.node_instance_type]
      capacity_type  = "ON_DEMAND"

      min_size     = var.min_nodes
      max_size     = var.max_nodes
      desired_size = var.desired_nodes

      # 启用集群自动扩缩容
      enable_cluster_autoscaler = true

      # 节点组标签
      labels = {
        Environment = var.environment
        NodeGroup   = "agentmem-nodes"
      }

      # 节点组污点
      taints = []

      # 用户数据脚本
      pre_bootstrap_user_data = <<-EOT
        #!/bin/bash
        /etc/eks/bootstrap.sh ${var.cluster_name}
      EOT

      tags = {
        Environment = var.environment
        Project     = "AgentMem"
        Terraform   = "true"
      }
    }
  }

  # 集群插件
  cluster_addons = {
    coredns = {
      most_recent = true
    }
    kube-proxy = {
      most_recent = true
    }
    vpc-cni = {
      most_recent = true
    }
    aws-ebs-csi-driver = {
      most_recent = true
    }
  }

  tags = {
    Environment = var.environment
    Project     = "AgentMem"
    Terraform   = "true"
  }
}

# ================================
# RDS PostgreSQL 数据库
# ================================
module "rds" {
  source = "terraform-aws-modules/rds/aws"
  version = "~> 6.0"

  identifier = "${var.cluster_name}-postgres"

  engine            = "postgres"
  engine_version    = "15.4"
  instance_class    = "db.r6g.xlarge"
  allocated_storage = 100
  storage_encrypted = true

  db_name  = "agentmem"
  username = "agentmem"
  port     = "5432"

  vpc_security_group_ids = [aws_security_group.rds.id]
  db_subnet_group_name   = module.vpc.database_subnet_group

  backup_retention_period = 7
  backup_window          = "03:00-04:00"
  maintenance_window     = "sun:04:00-sun:05:00"

  # 启用性能洞察
  performance_insights_enabled = true
  performance_insights_retention_period = 7

  # 启用监控
  monitoring_interval = 60
  monitoring_role_name = "${var.cluster_name}-rds-monitoring-role"
  create_monitoring_role = true

  tags = {
    Environment = var.environment
    Project     = "AgentMem"
    Terraform   = "true"
  }
}

# ================================
# ElastiCache Redis 集群
# ================================
resource "aws_elasticache_subnet_group" "redis" {
  name       = "${var.cluster_name}-redis-subnet-group"
  subnet_ids = module.vpc.private_subnets

  tags = {
    Environment = var.environment
    Project     = "AgentMem"
    Terraform   = "true"
  }
}

resource "aws_elasticache_replication_group" "redis" {
  replication_group_id       = "${var.cluster_name}-redis"
  description                = "AgentMem Redis cluster"

  node_type            = "cache.r6g.large"
  port                 = 6379
  parameter_group_name = "default.redis7"

  num_cache_clusters = 3
  
  subnet_group_name  = aws_elasticache_subnet_group.redis.name
  security_group_ids = [aws_security_group.redis.id]

  at_rest_encryption_enabled = true
  transit_encryption_enabled = true
  auth_token                 = random_password.redis_auth_token.result

  automatic_failover_enabled = true
  multi_az_enabled          = true

  tags = {
    Environment = var.environment
    Project     = "AgentMem"
    Terraform   = "true"
  }
}

# ================================
# 安全组
# ================================
resource "aws_security_group" "rds" {
  name_prefix = "${var.cluster_name}-rds-"
  vpc_id      = module.vpc.vpc_id

  ingress {
    from_port   = 5432
    to_port     = 5432
    protocol    = "tcp"
    cidr_blocks = [module.vpc.vpc_cidr_block]
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = {
    Name        = "${var.cluster_name}-rds-sg"
    Environment = var.environment
    Project     = "AgentMem"
    Terraform   = "true"
  }
}

resource "aws_security_group" "redis" {
  name_prefix = "${var.cluster_name}-redis-"
  vpc_id      = module.vpc.vpc_id

  ingress {
    from_port   = 6379
    to_port     = 6379
    protocol    = "tcp"
    cidr_blocks = [module.vpc.vpc_cidr_block]
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = {
    Name        = "${var.cluster_name}-redis-sg"
    Environment = var.environment
    Project     = "AgentMem"
    Terraform   = "true"
  }
}

# ================================
# 随机密码生成
# ================================
resource "random_password" "redis_auth_token" {
  length  = 32
  special = true
}

# ================================
# 输出
# ================================
output "cluster_endpoint" {
  description = "EKS cluster endpoint"
  value       = module.eks.cluster_endpoint
}

output "cluster_name" {
  description = "EKS cluster name"
  value       = module.eks.cluster_name
}

output "rds_endpoint" {
  description = "RDS instance endpoint"
  value       = module.rds.db_instance_endpoint
  sensitive   = true
}

output "redis_endpoint" {
  description = "Redis cluster endpoint"
  value       = aws_elasticache_replication_group.redis.primary_endpoint_address
  sensitive   = true
}
