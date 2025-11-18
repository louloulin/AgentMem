#!/bin/bash
set -e

echo "🔍 Testing LumosAI Chat Integration..."

BASE_URL="http://localhost:8080"

# 1. Health check
echo "1️⃣  Health check..."
curl -s $BASE_URL/health | jq -r '.status'

# 2. Create test agent
echo "2️⃣  Creating test agent..."
AGENT_RESPONSE=$(curl -s -X POST $BASE_URL/api/v1/agents \
  -H "Content-Type: application/json" \
  -d '{
    "name": "LumosAI Test Agent",
    "system": "You are a helpful AI assistant powered by LumosAI with AgentMem integration.",
    "llm_config": {
      "provider": "openai",
      "model": "gpt-3.5-turbo"
    },
    "organization_id": "test_org"
  }')

AGENT_ID=$(echo $AGENT_RESPONSE | jq -r '.data.id')
echo "Created agent: $AGENT_ID"

# 3. Test LumosAI Chat
echo "3️⃣  Testing LumosAI chat..."
CHAT_RESPONSE=$(curl -s -X POST $BASE_URL/api/v1/agents/$AGENT_ID/chat/lumosai \
  -H "Content-Type: application/json" \
  -d '{
    "message": "Hello! Tell me about LumosAI memory integration.",
    "user_id": "test_user_new"
  }')

echo "Chat response:"
echo $CHAT_RESPONSE | jq '.'

# 4. Verify response
SUCCESS=$(echo $CHAT_RESPONSE | jq -r '.success')
CONTENT=$(echo $CHAT_RESPONSE | jq -r '.data.content')
MEMORIES_UPDATED=$(echo $CHAT_RESPONSE | jq -r '.data.memories_updated')

if [ "$SUCCESS" = "true" ] && [ ! -z "$CONTENT" ] && [ "$MEMORIES_UPDATED" = "true" ]; then
    echo "✅ LumosAI Chat Integration Test PASSED"
else
    echo "❌ Test FAILED"
    exit 1
fi
