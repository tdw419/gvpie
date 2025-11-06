#!/bin/bash
# Startup script for Zero Human Daemon

set -e

cd "$(dirname "$0")"

echo "🚀 Starting Zero Human Daemon"
echo ""

# Check if ai_runtime is running
if ! curl -s http://localhost:8081/health > /dev/null 2>&1; then
    echo "⚠️  AI Runtime API is not running!"
    echo ""
    echo "Please start it in another terminal:"
    echo "  cd ../ai_runtime_rust"
    echo "  cargo run --release"
    echo ""
    exit 1
fi

echo "✅ AI Runtime API is healthy"
echo ""

# Check if database exists and has scripts
if [ ! -f "db/daemon.db" ]; then
    echo "📦 Initializing database..."
    python3 zero_human_daemon.py &
    DAEMON_PID=$!
    sleep 2
    kill $DAEMON_PID 2>/dev/null || true
    wait $DAEMON_PID 2>/dev/null || true
    echo "✅ Database created"
    echo ""
fi

# Load initial scripts if needed
SCRIPT_COUNT=$(sqlite3 db/daemon.db "SELECT COUNT(*) FROM improvement_scripts" 2>/dev/null || echo "0")

if [ "$SCRIPT_COUNT" -eq "0" ]; then
    echo "📝 Loading initial improvement scripts..."
    for sql_file in improvement_scripts/*.sql; do
        if [ -f "$sql_file" ]; then
            echo "   Loading $(basename $sql_file)..."
            sqlite3 db/daemon.db < "$sql_file"
        fi
    done
    echo "✅ Scripts loaded"
    echo ""
fi

# Create logs directory
mkdir -p logs

# Start the daemon
echo "🤖 Starting autonomous development loop..."
echo ""
python3 zero_human_daemon.py 2>&1 | tee logs/daemon.log
