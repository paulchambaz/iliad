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

You can also add it to your NixOS configuration.

### Using Docker

A Docker image is available for easy deployment:

```
docker run -v /path/to/your/audiobooks:/app/instance -e ILIAD_ADMIN_PASSWORD=your-password -p 9090:9090 paulchambaz/iliad:latest
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

This project is licensed under the GPLv3. See the [LICENSE](LICENSE) file for details.

## Authors

Written by Paul Chambaz in 2024.
