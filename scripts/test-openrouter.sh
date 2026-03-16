#!/usr/bin/env bash
set -euo pipefail

ENV_FILE="$(dirname "$0")/../.env.agent"
API_KEY=$(grep -E '^MAGMA_OPENROUTER_API_KEY=' "$ENV_FILE" | head -1 | cut -d'=' -f2- | tr -d "\"' ")

if [[ -z "$API_KEY" ]]; then
    echo "ERROR: MAGMA_OPENROUTER_API_KEY not found in .env.agent"
    exit 1
fi

ENDPOINT="https://openrouter.ai/api/v1/chat/completions"
PRIMARY="z-ai/glm-4.5-air:free"
FALLBACK="nvidia/nemotron-3-super-120b-a12b:free"

SYSTEM_PROMPT='You are Magma, an AI workspace agent in a terminal emulator. Observe context, respond with JSON actions.
RULES: Return ONLY a JSON array of actions. No markdown, no explanation. Each action needs `kind` and `confidence` (0.0-1.0). Use confidence >= 0.90 for clear, safe actions.
ACTIONS: surface_message, run_command, open_pane, filter_logr, stage_hunk, write_annotation.'

USER_PROMPT='## Intent
Respond to workspace events: command_failed

## Workspace Context
{"terminal":{"cwd":"/home/user/project","last_lines":["error[E0308]: mismatched types","  --> src/main.rs:42:5"],"last_exit_code":1,"last_command":"cargo check"},"git":{"branch":"feature/agent","unstaged_summary":["src/main.rs"]},"logs":{},"active_pane":{},"annotations":[],"memory":[]}'

call_model() {
    local model="$1"
    local start=$SECONDS
    local response
    response=$(curl -sS -w "\n%{http_code}" \
        -X POST "$ENDPOINT" \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer $API_KEY" \
        -d "$(python3 -c "
import json, sys
print(json.dumps({
    'model': '$model',
    'messages': [
        {'role': 'system', 'content': '''$SYSTEM_PROMPT'''},
        {'role': 'user', 'content': '''$USER_PROMPT'''},
    ],
    'temperature': 0.2,
}))
")" 2>&1)
    local elapsed=$((SECONDS - start))
    local http_code
    http_code=$(echo "$response" | tail -1)
    local body
    body=$(echo "$response" | sed '$d')

    echo "  Model:    $model"
    echo "  HTTP:     $http_code"
    echo "  Time:     ${elapsed}s"

    if [[ "$http_code" -ge 200 && "$http_code" -lt 300 ]]; then
        local content
        content=$(echo "$body" | python3 -c "
import json, sys
data = json.load(sys.stdin)
print(data['choices'][0]['message']['content'])
" 2>/dev/null)
        echo "  Content:  $content"
        echo ""
        echo "$content" | python3 -c "
import json, sys
raw = sys.stdin.read().strip()
if raw.startswith('\`\`\`'):
    lines = raw.split('\n')
    lines = [l for l in lines if not l.strip().startswith('\`\`\`')]
    raw = '\n'.join(lines)
try:
    parsed = json.loads(raw)
    if isinstance(parsed, dict) and 'actions' in parsed:
        parsed = parsed['actions']
    if not isinstance(parsed, list):
        parsed = [parsed]
    valid = 0
    for a in parsed:
        kind = a.get('kind', '?')
        conf = a.get('confidence', 0)
        auto = 'auto-exec' if kind in ('surface_message','open_pane','filter_logr') and conf >= 0.90 else 'needs-confirm'
        print(f'    [{kind}] confidence={conf} -> {auto}')
        if kind == 'surface_message': print(f'      msg: {a.get(\"message\",\"\")}')
        elif kind == 'run_command': print(f'      cmd: {a.get(\"command\",\"\")}')
        valid += 1
    print(f'  Result:   {valid} valid action(s)')
except json.JSONDecodeError as e:
    print(f'  PARSE FAILED: {e}')
    sys.exit(1)
"
        return 0
    else
        echo "  FAILED (will try fallback)"
        return 1
    fi
}

echo "=== Magma Agent Integration Test ==="
echo ""

echo "--- Primary model ---"
if ! call_model "$PRIMARY"; then
    echo ""
    echo "--- Fallback model ---"
    call_model "$FALLBACK" || { echo "Both models failed"; exit 1; }
fi

echo ""
echo "=== PASS ==="
