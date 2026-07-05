#!/bin/bash
# Quick verification that backup config deserialization works correctly
set -e

cd /home/coding/HOOP/test_backup_deser

echo "=== Testing Backup Config Deserialization ==="
echo

cargo run --quiet 2>&1 | grep -A 10 "Minimal Config Test Results"

echo
echo "=== Verification Complete ==="
echo "If defaults are shown correctly above (schedule: 0 4 * * *, retention_days: 30, encryption: false),"
echo "then the deserialization logic is working and the test in backup.rs is correct."
