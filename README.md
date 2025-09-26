# multiplayer-fps

Maze Wars–style multiplayer FPS with a client–server architecture over UDP. This repository currently contains an empty scaffold (folders and files only) to organize the implementation.

## Overview

- Authoritative server over UDP
- Client prompts for server IP and username at startup
- 3+ maze levels with increasing difficulty
- Real-time rendering with camera updates
- Mini-map showing player position and entire maze
- On-screen FPS counter (target: > 50 FPS)

## Repository Structure

- [Cargo.toml] — Rust workspace manifest (empty for now)
- [README.md] — Project overview, goals, and roadmap
- `docs/`
  - [architecture.md] — System architecture (to be filled)
  - [protocol.md] — Networking protocol (to be filled)
- `scripts/`
  - [run_server.ps1] — Helper to run server (to be filled)
  - [run_client.ps1] — Helper to run client (to be filled)
- `levels/`
  - [level1.ron] — Easy level placeholder
  - [level2.ron] — Medium level placeholder
  - [level3.ron] — Hard level placeholder
- `crates/`
  - `shared/`
    - [Cargo.toml] — Shared crate manifest (empty)
    - [src/lib.rs] — Shared library root (empty)
  - `server/`
    - [Cargo.toml] — Server crate manifest (empty)
    - [src/main.rs] — Server entry point (empty)
  - `client/`
    - [Cargo.toml] — Client crate manifest (empty)
    - [src/main.rs] — Client entry point (empty)

## Requirements Mapping

- Client–server over UDP: `crates/server`, `crates/client`, `crates/shared`
- Prompt for IP/username: `crates/client` (startup flow)
- Mini-map and camera: `crates/client` (rendering + UI)
- FPS display: `crates/client` (overlay)
- 3+ levels: `levels/` + loaders in client and server
- 10+ connections: server session management, config-driven cap
- Performance target (> 50 FPS): renderer choice + profiling

## Getting Started (planned)

1. Populate minimal Cargo manifests (name/version/edition) in the workspace and crates.
2. Choose the client rendering stack:
   - Bevy (plugin-based, ECS, batteries-included), or
   - macroquad (lightweight, quick to implement).
3. Define the UDP protocol in [docs/protocol.md](cci:7://file:///c:/Users/walaa/Downloads/multiplayer-fps/docs/protocol.md:0:0-0:0) and implement types in `crates/shared`.
4. Implement server networking and fixed-tick loop.
5. Implement client networking, prediction/interpolation, rendering, mini-map, and FPS overlay.
6. Fill the `levels/*.ron` format and loaders.

## Roadmap

- Workspace and crate manifests
- Protocol specification and serialization
- Server: sockets, sessions, world state, tick loop
- Client: init flow (IP/username), input, snapshots, rendering
- Levels: 3+ handcrafted mazes with increasing difficulty
- Optional bonus:
  - Level editor
  - Procedural maze generator
  - AI bots
  - Host history and aliases

## Contributing

This is a coursework project. Contributions and code style will be documented once the initial implementation lands.

## License

TBD (MIT or Apache-2.0 recommended)