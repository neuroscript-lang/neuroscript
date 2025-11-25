#!/bin/bash
cd "$(dirname "$0")"

echo "Testing all example files..."
echo

passed=0
failed=0

for file in examples/01-*.ns examples/02-*.ns examples/03-*.ns examples/04-*.ns examples/05-*.ns examples/06-*.ns examples/07-*.ns examples/08-*.ns examples/09-*.ns examples/10-*.ns examples/11-*.ns examples/12-*.ns examples/13-*.ns examples/14-*.ns examples/15-*.ns; do
  if [ ! -f "$file" ]; then
    continue
  fi
  
  # Run the parser and capture both stdout and stderr
  output=$(./target/release/neuroscript "$file")
  exitcode=$?
  
  # Check if it contains "Parsed" (success) rather than "Parse error"
  if echo "$output" | grep -q "^Parsed"; then
    echo "✓ $file"
    ((passed++))
  else
    echo "✗ $file"
    # Show the error
    echo "$output" | grep "Parse error"
    ((failed++))
  fi
done

echo
echo "Passed: $passed"
echo "Failed: $failed"
