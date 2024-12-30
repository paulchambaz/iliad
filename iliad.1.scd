iliad(1) "https://github.com/username/iliad" "General Commands Manual"

# NAME

*iliad* - Audio book server and library manager

# SYNOPSIS

*iliad* [OPTIONS]

# DESCRIPTION

*iliad* is a lightweight, self-hosted audio book server and library manager. It provides a RESTful API for managing and accessing your audio book collection, with features including user authentication, library scanning, and playback position tracking.

Key features include:
- User authentication with regular and admin accounts
- Automatic library scanning and metadata extraction
- Audio book playback position tracking
- RESTful API for client applications

# CONFIGURATION

*iliad* uses environment variables for configuration. These can be set in a *.env* file in the same directory as the executable, or as system environment variables.

*ILIAD_DB_PATH*
	Path to the SQLite database file [default: /app/instance/iliad.db]

*ILIAD_LIBRARY_PATH*
	Path to the audio book library directory [default: /app/instance/library]

*ILIAD_SERVER_ADDRESS*
	IP address for the server to bind to [default: 0.0.0.0]

*ILIAD_SERVER_PORT*
	Port number for the server to listen on [default: 9090]

*ILIAD_PUBLIC_REGISTER*
	Allow public user registration without admin authentication [default: false]

*ILIAD_ADMIN_PASSWORD*
	Password for admin authentication (required)

# API ENDPOINTS

All authenticated requests should include an "Authorization" header with the format "Bearer {token}", where {token} is the token received from the login or register endpoints.

## Authentication

*POST /auth/login*
	Log in as a regular user
	Input:

	```
	{
	  "username": "string",
	  "password": "string"
	}
	```

	Output:

	```
	{
	  "token": "string"
	}
	```

*POST /auth/admin*
	Log in as an admin user
	Input:

	```
	{
	  "password": "string"
	}
	```

	Output:

	```
	{
	  "token": "string"
	}
	```

*POST /auth/register*
	Register a new user (requires admin authentication if ILIAD_PUBLIC_REGISTER is false)
	Input:

	```
	{
	  "username": "string",
	  "password": "string"
	}
	```

	Output:
	```
	{
	  "token": "string"
	}
	```

## Audio Books

*GET /audiobooks*
	List all audio books (requires authentication)
	Input: None
	Output:

	```
	[
	  {
	    "hash": "string",
	    "title": "string",
	    "author": "string"
	  },
	  ...
	]
	```

*GET /audiobooks/{hash}*
	Get details for a specific audio book (requires authentication)
	Input: None
	Output:

	```
	{
	  "hash": "string",
	  "title": "string",
	  "author": "string",
	  "date": "integer",
	  "description": "string",
	  "genres": ["string", ...],
	  "duration": "integer",
	  "size": "integer"
	}
	```

*GET /audiobooks/{hash}/download*
	Download an audio book archive (requires authentication)
	Input: None
	Output:
	- Binary file (application/octet-stream)
	- Filename format: "{author-slug}-{title-slug}-{date}.tar.gz"

## Playback Positions

*GET /positions/{hash}*
	Get the current playback position for an audio book (requires authentication)
	Input: None
	Output:

	```
	{
	  "chapter_index": "integer",
	  "chapter_position": "integer"
	}
	```

*PUT /positions/{hash}*
	Update the playback position for an audio book (requires authentication)
	Input:

	```
	{
	  "chapter_index": "integer",
	  "chapter_position": "integer",
	  "client_timestamp": "integer"
	}
	```

	Output: 200 OK (no content)

## Library Management

*PUT /library/scan*
	Trigger a library scan (requires admin authentication)
	Input: None
	Output: 200 OK (no content)

*PUT /library/cleanup*
	Clean up old playback positions (requires admin authentication)
	Input: None
	Output: 200 OK (no content)

# AUDIO BOOK DIRECTORY STRUCTURE

Each audio book in the library should be in its own directory with the following structure:

```
/path/to/library/
└── Audio Book Title/
    ├── info.yaml
    ├── chapter1.mp3
    ├── chapter2.mp3
    └── ...
```

The *info.yaml* file should contain metadata about the audio book in the following format:

```
title: "Audio Book Title"
author: "Author Name"
date: 2023
description: "A brief description of the audio book"
genres:
  - "Genre 1"
  - "Genre 2"
chapters:
  - title: "Chapter 1 Title"
    path: "chapter1.mp3"
  - title: "Chapter 2 Title"
    path: "chapter2.mp3"
```

# NOTES

- *iliad* uses SQLite for its database, which is created automatically if it doesn't exist.
- The server automatically scans the library directory on startup.
- Audio book files are not served directly; instead, they are packaged into a .tar.gz archive for download.
- Playback positions older than 3 years are automatically cleaned up.

# AUTHORS

This program was created by [Your Name] in [Year]. It is licensed under [License], for more information, please read the *LICENSE* file in the project repository.

# SEE ALSO

sqlite3(1), tar(1), gzip(1)
