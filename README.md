# Unhidra – Distributed Messaging System

Unhidra is a multi-service Rust messaging platform composed of:

• gateway-service – WebSocket entrypoint  
• auth-api – authentication and token issuance  
• chat-service – message routing  
• presence-service – online/offline tracking  
• history-service – message history retrieval  
• event-hub-service – internal pub/sub bridge  
• client – CLI test client  
• startup-check – health diagnostics tool  

==================================================
CURRENT STATUS (v0.1.3)
==================================================

GATEWAY SERVICE
• Running on: ws://127.0.0.1:9000/ws  
• WebSocket echo verified  
• Real broadcast messaging now functional  
• Utf8Bytes compatibility implemented  
• Stream forwarding and session isolation confirmed  

AUTH API
• Running on http://127.0.0.1:9200  
• Login route (POST) active  
• Health check validated through client  
• DB opens cleanly at /opt/unhidra/auth.db  

CLIENT
• Performs Auth API health test  
• Connects to Gateway WebSocket  
• Confirms message path availability  
• Ready for next step: auth tokens and identity  

STARTUP CHECK
• Validates all ports:  
  9000 gateway  
  9200 auth  
  9300 chat  
  9400 presence  
  9500 history  
• Confirms DOWN/UP globally before boot  

==================================================
RECENT UPDATES (v0.1.3)
==================================================

SERVICE BOOT
• All services now successfully run together under tmux  
• startup-check confirms service ports before launch  
• Event-hub fixes ensure forwarding doesn’t panic  

WEBSOCKET FUNCTIONALITY
• Gateway now fully processes text messages  
• Transmission uses correct UTF-8 type conversions  
• Verified with websocat and our real client  

CLIENT IMPROVEMENT
• Now prints both Auth API status and WS connectivity  
• Gateway connection path validated  
• Prepped for token-based login  

DOCUMENTATION
• README rewritten and expanded  
• Added versioned changelog sections  
• GitHub repo updated with clean history  

==================================================
INSTALLATION & BUILD
==================================================

Clone and build:

git clone git@github.com:BronBron-Commits/Unhidra.git
cd Unhidra
cargo build --workspace --release

==================================================
NEXT STEPS
==================================================

• Implement authenticated WebSocket sessions  
• Introduce message envelopes using JWTs  
• Create a real-time chat client UI (web + native)  
• Expand presence-service for idle/active/typing  
• Add Windows & Android builds  
• Federation protocol design for multi-node networks  

