# iliad

A lightweight, self-hosted audio book server and library manager.

## Description

Iliad is a server application that provides a RESTful API for managing and accessing your audio book collection. It features user authentication, automatic library scanning, metadata extraction, and playback position tracking.

## Installation

### Using Nix Flakes

If you have Nix with flakes enabled, you can run Iliad directly from the repository:

```
nix run github:paulchambaz/iliad
```

You can also add it to your NixOS configuration as a flake input.

### Using Docker

Build and run the Docker image via Nix:

```
nix build .#docker
docker load < result
docker compose up
```

Or with the provided justfile:

```
just docker
```

## Quick Start

1. Set up your audio book library following the structure described in the man page.

2. Start the Iliad server:

   ```
   iliad
   ```

3. Access the API at `http://localhost:9090` (default port).

## Usage

For detailed usage instructions, please refer to the [man page](iliad.1.scd).

For client implementation details and the full API contract, see [CLIENTS.md](CLIENTS.md).

Basic API endpoints:

- `/auth/*`: User authentication
- `/audiobooks/*`: Audio book management
- `/positions/*`: Playback position tracking
- `/library/*`: Library management (admin only)

## Configuration

Iliad uses environment variables for configuration. Key variables:

- `ILIAD_DB_PATH`: Path to the SQLite database file
- `ILIAD_LIBRARY_PATH`: Path to the audio book library directory
- `ILIAD_ADMIN_PASSWORD`: Password for admin authentication (required)

For a complete list of configuration options, consult the man page.

## License

<a href="https://www.gnu.org/licenses/gpl-3.0.en.html">
  <img src="https://www.gnu.org/graphics/gplv3-127x51.png" alt="GNU GPLv3" />
</a>

Iliad is free software. You can use, study, modify and distribute it under the
terms of the [GNU General Public License](https://www.gnu.org/licenses/gpl-3.0.en.html),
version 3 or later.
