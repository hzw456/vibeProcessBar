#!/bin/bash

echo "=== Vibe Process Bar API Test ==="
echo ""

API_URL="http://localhost:31415"
TASK_ID="test-$(date +%s)"

echo "Task ID: $TASK_ID"
echo ""

# Test 1: Get initial status
echo "1. Getting initial status..."
curl -s "$API_URL/api/status"
echo ""
echo ""

# Test 2: Start a new task
echo "2. Starting a new task..."
curl -s -X POST "$API_URL/api/task/start" \
  -H "Content-Type: application/json" \
  -d "{
    \"task_id\": \"$TASK_ID\",
    \"name\": \"Generate user module\",
    \"ide\": \"cursor\",
    \"window_title\": \"user.ts - Cursor\"
  }"
echo ""
echo ""

# Test 3: Get status after starting task
echo "3. Getting status after starting task..."
curl -s "$API_URL/api/status"
echo ""
echo ""

# Test 4: Update progress
echo "4. Updating progress to 50%..."
curl -s -X POST "$API_URL/api/task/progress" \
  -H "Content-Type: application/json" \
  -d "{
    \"task_id\": \"$TASK_ID\",
    \"progress\": 50
  }"
echo ""
echo ""

# Test 5: Add tokens (increment mode)
echo "5. Adding 1500 tokens (increment mode)..."
curl -s -X POST "$API_URL/api/task/token" \
  -H "Content-Type: application/json" \
  -d "{
    \"task_id\": \"$TASK_ID\",
    \"tokens\": 1500,
    \"increment\": true
  }"
echo ""
echo ""

# Test 6: Add more tokens
echo "6. Adding 500 more tokens..."
curl -s -X POST "$API_URL/api/task/token" \
  -H "Content-Type: application/json" \
  -d "{
    \"task_id\": \"$TASK_ID\",
    \"tokens\": 500,
    \"increment\": true
  }"
echo ""
echo ""

# Test 7: Get status after updates
echo "7. Getting status after updates..."
curl -s "$API_URL/api/status"
echo ""
echo ""

# Test 8: Complete task
echo "8. Completing task with 2500 total tokens..."
curl -s -X POST "$API_URL/api/task/complete" \
  -H "Content-Type: application/json" \
  -d "{
    \"task_id\": \"$TASK_ID\",
    \"total_tokens\": 2500
  }"
echo ""
echo ""

# Test 9: Get final status
echo "9. Getting final status..."
curl -s "$API_URL/api/status"
echo ""
echo ""

# Test 10: Start another task for error testing
ERROR_TASK_ID="error-$(date +%s)"
echo "10. Starting error test task (ID: $ERROR_TASK_ID)..."
curl -s -X POST "$API_URL/api/task/start" \
  -H "Content-Type: application/json" \
  -d "{
    \"task_id\": \"$ERROR_TASK_ID\",
    \"name\": \"Error Test Task\",
    \"ide\": \"vscode\",
    \"window_title\": \"error.ts\"
  }"
echo ""
echo ""

# Test 11: Set task to error status
echo "11. Setting task to error status..."
curl -s -X POST "$API_URL/api/task/error" \
  -H "Content-Type: application/json" \
  -d "{
    \"task_id\": \"$ERROR_TASK_ID\",
    \"message\": \"Something went wrong\"
  }"
echo ""
echo ""

# Test 12: Get status after error
echo "12. Getting status after error..."
curl -s "$API_URL/api/status"
echo ""
echo ""

# Test 13: Cancel a task
echo "13. Canceling error task..."
curl -s -X POST "$API_URL/api/task/cancel" \
  -H "Content-Type: application/json" \
  -d "{
    \"task_id\": \"$ERROR_TASK_ID\"
  }"
echo ""
echo ""

# Test 14: Reset all tasks
echo "14. Resetting all tasks..."
curl -s -X POST "$API_URL/api/reset" \
  -H "Content-Type: application/json" \
  -d '{}'
echo ""
echo ""

# Test 15: Verify reset
echo "15. Verifying reset..."
curl -s "$API_URL/api/status"
echo ""
echo ""

echo "=== Test Complete ==="
