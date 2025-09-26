# Architecture

## Overview
Client-server multiplayer FPS with authoritative server and client-side prediction.

## Components

### Server (`crates/server`)
- Authoritative game state
- UDP socket handling
- Player session management
- Game loop at 60 Hz
- Collision detection
- Level management

### Client (`crates/client`)
- 3D first-person rendering (raycasting)
- Input handling and prediction
- Network synchronization
- Mini-map display
- FPS counter
- macroquad rendering engine

### Shared (`crates/shared`)
- Protocol message types
- Common game types
- Serialization utilities
- Constants and configuration

## Game Flow
1. Client prompts for IP and username
2. Client connects to server via UDP
3. Server accepts connection and assigns player ID
4. Game loop: input → prediction → server sync → render
5. Player navigates maze to find exit
6. Level progression (3 levels total)
7. Win condition: complete all levels

## Performance
- Target: >50 FPS
- Raycasting for 3D rendering
- Efficient UDP messaging
- Client-side prediction for smooth movement