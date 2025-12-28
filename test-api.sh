#!/bin/bash

echo "=== Vibe Process Bar API Test ==="
echo ""

API_URL="http://localhost:31415"

echo "1. Starting a new task..."
curl -s -X POST "$API_URL/api/task/start" \
  -H "Content-Type: application/json" \
  -d '{
    "taskId": "test-'"$(date +%s)"'",
    "name": "Generate user module",
    "ide": "cursor",
    "windowTitle": "user.ts - Cursor"
  }'
echo ""
echo ""

echo "2. Updating progress to 50%..."
curl -s -X POST "$API_URL/api/task/progress" \
  -H "Content-Type: application/json" \
  -d '{
    "taskId": "test-'"$(date +%s)"'",
    "progress": 50
  }'
echo ""
echo ""

echo "3. Adding 1500 tokens..."
curl -s -X POST "$API_URL/api/task/token" \
  -H "Content-Type: application/json" \
  -d '{
    "taskId": "test-'"$(date +%s)"'",
    "tokens": 1500,
    "increment": true
  }'
echo ""
echo ""

echo "4. Adding 500 more tokens..."
curl -s -X POST "$API_URL/api/task/token" \
  -H "Content-Type: application/json" \
  -d '{
    "taskId": "test-'"$(date +%s)"'",
    "tokens": 500,
    "increment": true
  }'
echo ""
echo ""

echo "5. Completing task with 2500 total tokens..."
curl -s -X POST "$API_URL/api/task/complete" \
  -H "Content-Type: application/json" \
  -d '{
    "taskId": "test-'"$(date +%s)"'",
    "totalTokens": 2500
  }'
echo ""
echo ""

echo "6. Getting current status..."
curl -s "$API_URL/api/status"
echo ""
echo ""

echo "7. Resetting all tasks..."
curl -s -X POST "$API_URL/api/reset" \
  -H "Content-Type: application/json" \
  -d '{}'
echo ""
echo ""

echo "=== Test Complete ==="
