#!/usr/bin/env bash
set -euo pipefail

# Start mock RPC in background
python3 /app/tools/mock_rpc.py &
RPC_PID=$!

# Allow mock RPC time to start
sleep 1

# Run tarpaulin to produce HTML report
cargo tarpaulin --all --out Html

# Wait for background process (will keep running until tarpaulin ends)
wait $RPC_PID
