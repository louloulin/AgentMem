#!/bin/bash
echo "🔍 Testing LumosAI Chat - Simple Version..."

BASE_URL="http://localhost:8080"

# Create agent (without LLM, for now we just test the architecture)
AGENT_RESPONSE=$(curl -s -X POST $BASE_URL/api/v1/agents \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Test Agent",
    "system": "Test system prompt",
    "organization_id": "test_org"
  }')

echo "Agent created:"
echo $AGENT_RESPONSE | jq '.'

AGENT_ID=$(echo $AGENT_RESPONSE | jq -r '.data.id')
echo "Agent ID: $AGENT_ID"

# Check if chat endpoint exists
echo "Testing LumosAI chat endpoint availability..."
CHAT_RESULT=$(curl -s -w "\n%{http_code}" -X POST $BASE_URL/api/v1/agents/$AGENT_ID/chat/lumosai \
  -H "Content-Type: application/json" \
  -d '{
    "message": "test",
    "user_id": "test_user"
  }')

HTTP_CODE=$(echo "$CHAT_RESULT" | tail -1)
RESPONSE=$(echo "$CHAT_RESULT" | head -n-1)

echo "HTTP Status: $HTTP_CODE"
echo "Response: $RESPONSE" | jq '.' || echo "$RESPONSE"

if [ "$HTTP_CODE" = "200" ] || [ "$HTTP_CODE" = "500" ]; then
    echo "✅ LumosAI Chat endpoint is accessible"
    if echo "$RESPONSE" | grep -q "API key"; then
        echo "⚠️  Needs API key configuration (expected)"
    fi
else
    echo "❌ Endpoint not found or error"
fi
