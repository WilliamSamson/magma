#!/usr/bin/env bash
set -euo pipefail

ENV_FILE="$(dirname "$0")/../.env.agent"

if [[ ! -f "$ENV_FILE" ]]; then
    echo "ERROR: .env.agent not found at $ENV_FILE"
    exit 1
fi

API_KEY=$(grep -E '^deepseek_apiKey=' "$ENV_FILE" | head -1 | cut -d'=' -f2- | tr -d "\"' ")

if [[ -z "$API_KEY" ]]; then
    echo "ERROR: deepseek_apiKey not found in .env.agent"
    exit 1
fi

echo "Testing DeepSeek API..."
echo "Endpoint: https://api.deepseek.com/v1/chat/completions"
echo ""

RESPONSE=$(curl -sS -w "\n%{http_code}" \
    -X POST "https://api.deepseek.com/v1/chat/completions" \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer $API_KEY" \
    -d '{
        "model": "deepseek-chat",
        "messages": [
            {"role": "system", "content": "You are a helpful assistant. Respond in JSON only."},
            {"role": "user", "content": "Return a JSON array with one object: {\"kind\": \"surface_message\", \"message\": \"hello from deepseek\", \"confidence\": 0.99}"}
        ],
        "temperature": 0.2,
        "response_format": {"type": "json_object"}
    }')

HTTP_CODE=$(echo "$RESPONSE" | tail -1)
BODY=$(echo "$RESPONSE" | sed '$d')

echo "HTTP Status: $HTTP_CODE"
echo ""

if [[ "$HTTP_CODE" -ge 200 && "$HTTP_CODE" -lt 300 ]]; then
    echo "SUCCESS - Response body:"
    echo "$BODY" | python3 -m json.tool 2>/dev/null || echo "$BODY"
else
    echo "FAILED - Response:"
    echo "$BODY"
    exit 1
fi
