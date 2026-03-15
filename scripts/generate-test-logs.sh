#!/usr/bin/env bash
# Generates live JSONL log entries to magma-debug.log.jsonl
# Run this while Magma is open and select the file in the logr pane
# to see live log tailing in action.

FILE="magma-debug.log.jsonl"
COMPONENTS=("auth" "api" "db" "cache" "worker" "scheduler")
LEVELS=("debug" "info" "info" "info" "warn" "error")
MESSAGES_INFO=(
    "request handled successfully"
    "cache hit for session lookup"
    "connection pool healthy"
    "background job completed"
    "health check passed"
    "config reloaded"
    "new client connected"
    "task dispatched to worker"
)
MESSAGES_WARN=(
    "slow query detected"
    "retry attempt on transient failure"
    "connection pool nearing capacity"
    "rate limit threshold approaching"
    "deprecated endpoint accessed"
)
MESSAGES_ERROR=(
    "failed to parse request body"
    "database connection timeout"
    "authentication token expired"
    "upstream service unavailable"
    "disk usage above threshold"
)
MESSAGES_DEBUG=(
    "entering request handler"
    "cache key generated"
    "serializing response payload"
    "checking feature flag"
    "resolving dependency graph"
)

echo "writing live logs to $FILE (Ctrl+C to stop)"
echo "open magma, select this file in the logr pane, and watch them stream in"
echo ""

while true; do
    ts=$(date +%s)
    component=${COMPONENTS[$RANDOM % ${#COMPONENTS[@]}]}
    level=${LEVELS[$RANDOM % ${#LEVELS[@]}]}

    case $level in
        info)  msg=${MESSAGES_INFO[$RANDOM % ${#MESSAGES_INFO[@]}]} ;;
        warn)  msg=${MESSAGES_WARN[$RANDOM % ${#MESSAGES_WARN[@]}]} ;;
        error) msg=${MESSAGES_ERROR[$RANDOM % ${#MESSAGES_ERROR[@]}]} ;;
        debug) msg=${MESSAGES_DEBUG[$RANDOM % ${#MESSAGES_DEBUG[@]}]} ;;
    esac

    request_id=$(printf '%04x%04x' $RANDOM $RANDOM)
    duration_ms=$(( RANDOM % 500 + 1 ))

    echo "{\"timestamp\":$ts,\"level\":\"$level\",\"message\":\"$msg\",\"component\":\"$component\",\"request_id\":\"$request_id\",\"duration_ms\":$duration_ms}" >> "$FILE"

    # Random delay between 200ms and 2s
    sleep_ms=$(( RANDOM % 1800 + 200 ))
    sleep "$(echo "scale=3; $sleep_ms/1000" | bc)"
done
