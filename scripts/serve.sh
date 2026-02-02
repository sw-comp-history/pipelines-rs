#!/bin/bash
set -euo pipefail

# Serve the WASM UI on port 9952
# Creates a symlink structure to match GitHub Pages path /pipelines-rs/

cd "$(dirname "$0")/.."

# Create a temporary serve directory with the right path structure
SERVE_DIR="$(mktemp -d)"
trap "rm -rf $SERVE_DIR" EXIT

# Create symlink: $SERVE_DIR/pipelines-rs -> pages/
ln -s "$(pwd)/pages" "$SERVE_DIR/pipelines-rs"

# Create root index.html that redirects to /pipelines-rs/
cat > "$SERVE_DIR/index.html" << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta http-equiv="refresh" content="0; url=/pipelines-rs/">
    <title>Redirecting...</title>
</head>
<body>
    <p>Redirecting to <a href="/pipelines-rs/">pipelines-rs</a>...</p>
</body>
</html>
EOF

echo "Serving pipelines-rs UI at http://localhost:9952/"
echo "(redirects to http://localhost:9952/pipelines-rs/)"
echo "Press Ctrl+C to stop"
basic-http-server -a 0.0.0.0:9952 "$SERVE_DIR"
