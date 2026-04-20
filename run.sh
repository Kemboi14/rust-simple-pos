#!/bin/bash
# Run Kipko POS Server and UI

# Kill existing processes on exit
trap 'kill $(jobs -p) 2>/dev/null; exit' EXIT INT TERM

# Start server in background
echo "Starting server on port 3000..."
SERVER_PORT=3000 cargo run --bin kipko-server &
SERVER_PID=$!

# Wait for server to start
sleep 3

# Start UI
echo "Starting UI..."
cd kipko-ui
API_BASE_URL=http://localhost:3000 $HOME/.cargo/bin/dx serve --no-default-features &
UI_PID=$!

echo "Both services running!"
echo "- Server: http://localhost:3000"
echo "- UI: http://localhost:8080"
echo "Press Ctrl+C to stop both"

# Wait for both processes
wait $SERVER_PID $UI_PID
