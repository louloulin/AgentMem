#!/bin/bash
# SQLx Setup Script
# This script sets up PostgreSQL and generates SQLx offline data

set -e

echo "🚀 AgentMem SQLx Setup Script"
echo "=============================="
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if PostgreSQL is installed
echo "📝 Step 1: Check PostgreSQL Installation"
echo "-----------------------------------------"

if command -v psql &> /dev/null; then
    echo -e "${GREEN}✅ PostgreSQL is installed${NC}"
    psql --version
else
    echo -e "${RED}❌ PostgreSQL is not installed${NC}"
    echo ""
    echo "Please install PostgreSQL:"
    echo "  macOS:   brew install postgresql@15"
    echo "  Ubuntu:  sudo apt-get install postgresql-15"
    echo "  Windows: Download from https://www.postgresql.org/download/"
    exit 1
fi

echo ""

# Check if PostgreSQL is running
echo "📝 Step 2: Check PostgreSQL Status"
echo "-----------------------------------"

if pg_isready &> /dev/null; then
    echo -e "${GREEN}✅ PostgreSQL is running${NC}"
else
    echo -e "${YELLOW}⚠️  PostgreSQL is not running${NC}"
    echo ""
    echo "Starting PostgreSQL..."
    
    # Try to start PostgreSQL (macOS with Homebrew)
    if command -v brew &> /dev/null; then
        brew services start postgresql@15 || brew services start postgresql
        sleep 2
        
        if pg_isready &> /dev/null; then
            echo -e "${GREEN}✅ PostgreSQL started successfully${NC}"
        else
            echo -e "${RED}❌ Failed to start PostgreSQL${NC}"
            echo "Please start PostgreSQL manually"
            exit 1
        fi
    else
        echo "Please start PostgreSQL manually:"
        echo "  macOS:   brew services start postgresql@15"
        echo "  Ubuntu:  sudo systemctl start postgresql"
        echo "  Windows: Start PostgreSQL service"
        exit 1
    fi
fi

echo ""

# Create database
echo "📝 Step 3: Create Database"
echo "--------------------------"

DB_NAME="agentmem_dev"
DB_USER="${USER}"

# Check if database exists
if psql -lqt | cut -d \| -f 1 | grep -qw "$DB_NAME"; then
    echo -e "${YELLOW}⚠️  Database '$DB_NAME' already exists${NC}"
    read -p "Do you want to drop and recreate it? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo "Dropping database..."
        dropdb "$DB_NAME" || true
        echo "Creating database..."
        createdb "$DB_NAME"
        echo -e "${GREEN}✅ Database recreated${NC}"
    else
        echo "Using existing database"
    fi
else
    echo "Creating database '$DB_NAME'..."
    createdb "$DB_NAME"
    echo -e "${GREEN}✅ Database created${NC}"
fi

echo ""

# Set DATABASE_URL
echo "📝 Step 4: Set DATABASE_URL"
echo "---------------------------"

export DATABASE_URL="postgresql://${DB_USER}@localhost/${DB_NAME}"
echo "DATABASE_URL=$DATABASE_URL"

# Save to .env file
echo "DATABASE_URL=$DATABASE_URL" > .env
echo -e "${GREEN}✅ DATABASE_URL saved to .env${NC}"

echo ""

# Run migrations
echo "📝 Step 5: Run Database Migrations"
echo "-----------------------------------"

# Check if sqlx-cli is installed
if ! command -v sqlx &> /dev/null; then
    echo -e "${YELLOW}⚠️  sqlx-cli is not installed${NC}"
    echo "Installing sqlx-cli..."
    cargo install sqlx-cli --no-default-features --features postgres
fi

# Check if migrations directory exists
if [ -d "migrations" ]; then
    echo "Running migrations..."
    sqlx database create || true
    sqlx migrate run
    echo -e "${GREEN}✅ Migrations completed${NC}"
else
    echo -e "${YELLOW}⚠️  No migrations directory found${NC}"
    echo "Creating basic schema..."
    
    # Create basic tables
    psql "$DATABASE_URL" <<EOF
-- Create basic tables for SQLx to work
CREATE TABLE IF NOT EXISTS memories (
    id TEXT PRIMARY KEY,
    agent_id TEXT NOT NULL,
    user_id TEXT,
    content TEXT NOT NULL,
    memory_type TEXT NOT NULL,
    importance REAL NOT NULL DEFAULT 0.5,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    metadata JSONB DEFAULT '{}'::jsonb,
    search_vector tsvector
);

CREATE TABLE IF NOT EXISTS memory_lifecycle (
    id TEXT PRIMARY KEY,
    memory_id TEXT NOT NULL,
    state TEXT NOT NULL,
    event_type TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    metadata JSONB DEFAULT '{}'::jsonb
);

CREATE TABLE IF NOT EXISTS memory_associations (
    id TEXT PRIMARY KEY,
    organization_id TEXT,
    user_id TEXT,
    agent_id TEXT,
    memory_id_1 TEXT NOT NULL,
    memory_id_2 TEXT NOT NULL,
    association_type TEXT NOT NULL,
    strength REAL NOT NULL DEFAULT 0.5,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS episodic_memory (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    event TEXT NOT NULL,
    context TEXT,
    timestamp TIMESTAMPTZ NOT NULL,
    importance REAL NOT NULL DEFAULT 0.5,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS semantic_memory (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    summary TEXT,
    details TEXT,
    source TEXT,
    tree_path TEXT[],
    importance REAL NOT NULL DEFAULT 0.5,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS procedural_memory (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    steps TEXT[],
    conditions TEXT,
    expected_outcome TEXT,
    success_rate REAL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indices
CREATE INDEX IF NOT EXISTS idx_memories_agent_id ON memories(agent_id);
CREATE INDEX IF NOT EXISTS idx_memories_user_id ON memories(user_id);
CREATE INDEX IF NOT EXISTS idx_memories_type ON memories(memory_type);
CREATE INDEX IF NOT EXISTS idx_memories_search ON memories USING gin(search_vector);

EOF
    
    echo -e "${GREEN}✅ Basic schema created${NC}"
fi

echo ""

# Generate SQLx offline data
echo "📝 Step 6: Generate SQLx Offline Data"
echo "--------------------------------------"

echo "Preparing SQLx queries..."
cd crates/agent-mem-core
cargo sqlx prepare --workspace -- --lib

if [ -f ".sqlx/query-*.json" ] || [ "$(ls -A .sqlx 2>/dev/null)" ]; then
    echo -e "${GREEN}✅ SQLx offline data generated${NC}"
    echo "Files in .sqlx/:"
    ls -la .sqlx/
else
    echo -e "${RED}❌ Failed to generate SQLx offline data${NC}"
    exit 1
fi

cd ../..

echo ""

# Test compilation
echo "📝 Step 7: Test Compilation"
echo "----------------------------"

echo "Testing compilation with SQLX_OFFLINE=true..."
SQLX_OFFLINE=true cargo check --package agent-mem-core

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Compilation successful${NC}"
else
    echo -e "${RED}❌ Compilation failed${NC}"
    exit 1
fi

echo ""

# Summary
echo "🎉 SQLx Setup Complete!"
echo "======================="
echo ""
echo "✅ PostgreSQL is running"
echo "✅ Database '$DB_NAME' is ready"
echo "✅ DATABASE_URL is set in .env"
echo "✅ Migrations completed"
echo "✅ SQLx offline data generated"
echo "✅ Compilation successful"
echo ""
echo "📝 Next Steps:"
echo "  1. Run tests: cargo test --package agent-mem-core"
echo "  2. Run examples: cargo run --package real-agentmem-test"
echo ""
echo "💡 Tips:"
echo "  - Use SQLX_OFFLINE=true for offline compilation"
echo "  - DATABASE_URL is saved in .env file"
echo "  - Re-run this script if schema changes"
echo ""

