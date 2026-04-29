#!/usr/bin/env bash
# Generate TypeScript client from OpenAPI spec
#
# This script generates TypeScript types and API client code from the OpenAPI spec.
# The output is written to hoop-ui/web/src/api.gen.ts

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

echo "=== TypeScript Client Generation from OpenAPI Spec ==="
echo ""

# Check if openapi-typescript is installed
if ! command -v openapi-typescript &> /dev/null; then
    echo -e "${YELLOW}openapi-typescript not found${NC}"
    echo "Installing via npm..."
    npm install -g openapi-typescript
fi

# Paths
OPENAPI_SPEC="hoop-schema/openapi.yaml"
OUTPUT_DIR="hoop-ui/web/src"
OUTPUT_FILE="${OUTPUT_DIR}/api.gen.ts"

# Check if spec exists
if [ ! -f "$OPENAPI_SPEC" ]; then
    echo -e "${RED}✗ OpenAPI spec not found at ${OPENAPI_SPEC}${NC}"
    echo "Run 'cargo run --bin generate_openapi --features openapi > ${OPENAPI_SPEC}' first"
    exit 1
fi

echo "Generating TypeScript client from ${OPENAPI_SPEC}..."

# Create output directory if it doesn't exist
mkdir -p "$OUTPUT_DIR"

# Generate TypeScript types
if ! openapi-typescript "$OPENAPI_SPEC" -o "$OUTPUT_FILE"; then
    echo -e "${RED}✗ Failed to generate TypeScript client${NC}"
    exit 1
fi

echo -e "${GREEN}✓ TypeScript client generated to ${OUTPUT_FILE}${NC}"
echo ""
echo "Generated exports include:"
echo "  - paths: API endpoint types and request/response schemas"
echo "  - components: Shared schemas referenced by the API"
echo "  - operations: Operation-level types for each endpoint"
echo ""
echo "Usage in code:"
echo '  import type { paths, components } from "./api.gen";'
echo '  type AgentStatus = paths("/api/agent/status")["get"]["responses"]["200"]["content"]["application/json"];'
echo ""
exit 0
