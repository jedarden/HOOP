#!/bin/bash
# Generate placeholder screenshots for HOOP documentation
# These are simple SVG placeholders until real screenshots can be captured

SCREENSHOTS_DIR="$(dirname "$0")"

echo "🎨 Generating placeholder screenshots..."

# Create a simple SVG placeholder function
create_placeholder() {
    local title="$1"
    local subtitle="$2"
    local filename="$3"
    local color="$4"

    cat > "${SCREENSHOTS_DIR}/${filename}" << EOF
<svg width="1920" height="1080" xmlns="http://www.w3.org/2000/svg">
  <rect width="1920" height="1080" fill="#1a1a2e"/>
  <rect x="960" y="540" width="800" height="400" rx="20" fill="#16213e" stroke="#0f3460" stroke-width="2"/>
  <text x="960" y="400" font-family="Arial, sans-serif" font-size="48" fill="#e94560" text-anchor="middle" font-weight="bold">${title}</text>
  <text x="960" y="480" font-family="Arial, sans-serif" font-size="32" fill="#a0a0a0" text-anchor="middle">${subtitle}</text>
  <text x="960" y="550" font-family="Arial, sans-serif" font-size="24" fill="#666" text-anchor="middle">HOOP v1.0.0</text>
  <text x="960" y="600" font-family="Arial, sans-serif" font-size="20" fill="#555" text-anchor="middle">Run 'hoop init' to see the live interface</text>
  <rect x="760" y="650" width="400" height="50" rx="10" fill="#e94560"/>
  <text x="960" y="685" font-family="Arial, sans-serif" font-size="24" fill="white" text-anchor="middle">Live Demo Available</text>
</svg>
EOF

    echo "✅ Created ${filename}"
}

# Create placeholders
create_placeholder "Project Dashboard" "One card per project, aggregating active work, cost today, and alerts." "dashboard.png" "#e94560"
create_placeholder "Stitch Timeline" "All conversations in a project — worker sessions, operator chats, dictated notes." "project-detail.png" "#0f3460"
create_placeholder "Agent Chat" "Ask questions, draft work, get summaries — your primary interface to HOOP." "agent-chat.png" "#16213e"
create_placeholder "File Browser" "Navigate project files with code syntax highlighting and Stitch-aware change tracking." "file-browser.png" "#533483"

echo ""
echo "✨ Placeholder screenshots generated!"
echo "📁 Location: ${SCREENSHOTS_DIR}"
echo ""
echo "To generate real screenshots, run:"
echo "  cd hoop-ui/web && pnpm tsx scripts/take-screenshots.ts"
