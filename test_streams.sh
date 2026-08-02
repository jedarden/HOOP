#!/bin/bash
# Simple test script that outputs to both stdout and stderr
# This should help verify if streams are distinguishable in log files

echo "=== STREAM DISTINCTION TEST ==="
echo "This is STDOUT line 1"
echo "This is STDOUT line 2"
>&2 echo "This is STDERR line 1"
>&2 echo "This is STDERR line 2"
echo "This is STDOUT line 3"
>&2 echo "This is STDERR line 3"
echo "=== END TEST ==="
