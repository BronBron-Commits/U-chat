#!/usr/bin/env bash

# Kill any leftover processes
pkill -f "auth-api|gateway-service|chat-service|presence-service|history-service|bot-service" 2>/dev/null

# Export the shared JWT secret
export JWT_SECRET="supersecret"

# Go to project dir
cd ~/unhidra-rust

# Create a logs directory if missing
mkdir -p logs

echo "Starting UNHIDRA stack..."

# Launch each service silently in background
./target/release/auth-api          > logs/auth.log       2>&1 &
./target/release/gateway-service   > logs/gateway.log    2>&1 &
./target/release/chat-service      > logs/chat.log       2>&1 &
./target/release/presence-service  > logs/presence.log   2>&1 &
./target/release/history-service   > logs/history.log    2>&1 &
./target/release/bot-service       > logs/bot.log        2>&1 &

sleep 1

echo "All services launched."
echo "Use 'tail -f logs/<file>.log' to view output."
