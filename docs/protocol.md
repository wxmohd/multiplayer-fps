# Network Protocol

## Overview
UDP-based client-server protocol for multiplayer FPS game.

## Message Types

### Client to Server
- `CONNECT:<username>` - Initial connection request
- `INPUT:<x>,<y>,<angle>,<action>` - Player input and position
- `SHOOT:<angle>` - Shooting action
- `DISCONNECT` - Clean disconnect

### Server to Client  
- `ACCEPT:<player_id>` - Connection accepted
- `SNAPSHOT:<players_data>` - World state update
- `HIT:<player_id>` - Player was hit
- `LEVEL_COMPLETE` - Level completed
- `GAME_OVER` - Game ended

## Protocol Details
- Port: 34254
- Tick Rate: 60 Hz
- MTU: 1024 bytes
- Sequence numbers for reliability