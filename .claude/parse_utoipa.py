#!/usr/bin/env python3
import re
import json

# Read the raw clippy output
with open('.claude/utoipa-clippy-raw.txt', 'r') as f:
    content = f.read()

# Parse the ToSchema errors
parsed_errors = []

# Pattern to match:
# error: cannot find derive macro `ToSchema` in this scope
#   --> hoop-daemon/src/api_draft_queue.rs:34:28
pattern = r"error: cannot find derive macro `ToSchema` in this scope\s*\-->\s+(\S+):(\d+):\d+"

matches = re.finditer(pattern, content)

for match in matches:
    file_path = match.group(1)
    line_number = int(match.group(2))
    warning_text = f"cannot find derive macro `ToSchema` in this scope at {file_path}:{line_number}"

    parsed_errors.append({
        "file_path": file_path,
        "line_number": line_number,
        "warning_text": warning_text
    })

# Sort by line number
parsed_errors.sort(key=lambda x: x["line_number"])

# Write to JSON
output = {
    "total_count": len(parsed_errors),
    "errors": parsed_errors
}

with open('.claude/utoipa-parsed.json', 'w') as f:
    json.dump(output, f, indent=2)

print(f"Parsed {len(parsed_errors)} unused utoipa::ToSchema imports")
print(f"Output written to .claude/utoipa-parsed.json")
