#!/bin/bash

# Test multi-task functionality
echo "Creating multiple tasks..."

# Task 1
curl -s -X POST http://localhost:31415/api/task/start \
  -H "Content-Type: application/json" \
  -d '{"task_id":"task-1","name":"Building UI","ide":"kiro","window_title":"vibeProcessBar"}'
echo ""

# Task 2
curl -s -X POST http://localhost:31415/api/task/start \
  -H "Content-Type: application/json" \
  -d '{"task_id":"task-2","name":"API Work","ide":"cursor","window_title":"backend"}'
echo ""

# Task 3
curl -s -X POST http://localhost:31415/api/task/start \
  -H "Content-Type: application/json" \
  -d '{"task_id":"task-3","name":"Testing","ide":"vscode","window_title":"tests"}'
echo ""

echo "Setting progress..."

curl -s -X POST http://localhost:31415/api/task/progress \
  -H "Content-Type: application/json" \
  -d '{"task_id":"task-1","progress":75}'
echo ""

curl -s -X POST http://localhost:31415/api/task/progress \
  -H "Content-Type: application/json" \
  -d '{"task_id":"task-2","progress":40}'
echo ""

curl -s -X POST http://localhost:31415/api/task/progress \
  -H "Content-Type: application/json" \
  -d '{"task_id":"task-3","progress":20}'
echo ""

echo "Done! Check the floating window."

# Wait a bit then complete task 1
sleep 3
echo "Completing task 1..."
curl -s -X POST http://localhost:31415/api/task/complete \
  -H "Content-Type: application/json" \
  -d '{"task_id":"task-1"}'
echo ""
echo "Task 1 completed! You should see a completion notification."
