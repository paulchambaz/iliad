# Iliad

Iliad is a self-hosted audiobook server designed for those who wish to have personal control over their audiobook collections.

Iliad is an audiobook server designed to help users efficiently manage and listen to their audiobooks across various devices. It's built with a two goals in mind : to ensure that when you move from one device to another, you can easily pick up from where you left off and that you can always listen to your audiobooks, no matter if you are online or offline. It uses Rust and Sqlite3, so it is performant and stable. Additionally, Iliad works in tandem with its client : [odyssey](https://github.com/paulchambaz/odyssey.git) for android/ios and [odyssey-tui](https://github.com/paulchambaz/odyssey-tui) to offer a consistent audiobook experience accross multiple devices. Feel free to contribute more clients, since the protocol is very simple to implement.

## Installation

### Prerequistes

This program uses rust, so you will need to install rust nightly in order to compile it. It also uses libsqlite3 for managing the database, so this library needs to be installed at runtime on the system.

### Docker installation

Although you can run this project straight from the terminal, it is really intended to be installed through docker.

```sh
docker run -d \
    --name iliad \
    -p 8080:8080 \
    -v $(pwd)/database:/data/database \
    -v $(pwd)/library:/data/library \
    -e ILIAD_DB_PATH=/data/database/iliad.db \
    -e ILIAD_LIBRARY_PATH=/data/library \
    -e ILIAD_SCAN_INTERVAL=60 \
    -e ILIAD_CLEANUP_INTERVAL=30 \
    -e ILIAD_SERVER_ADDRESS=0.0.0.0 \
    -e ILIAD_SERVER_PORT=8080 \
    -e ILIAD_ALLOW_REGISTER=true \
    paulchambaz/iliad:latest
```

Or use docker-compose.

```yml
version: '3'

services:
  iliad:
    image: paulchambaz/iliad:latest
    ports:
      - "8080:8080"
    volumes:
      - /data/iliad/database:/data/database
      - /data/iliad/library:/data/library
    environment:
      ILIAD_DB_PATH: /data/database/iliad.db
      ILIAD_LIBRARY_PATH: /data/library
      ILIAD_SCAN_INTERVAL: 60
      ILIAD_CLEANUP_INTERVAL: 365
      ILIAD_SERVER_ADDRESS: "0.0.0.0"
      ILIAD_SERVER_PORT: 8080
      ILIAD_ALLOW_REGISTER: true
```

The environment variable are used to configure the program.

- **`ILIAD_DB_PATH`**: path to the database used
- **`ILIAD_LIBRARY_PATH`**: path to the library directory
- **`ILIAD_SCAN_INTERVAL`**: interval between each library scan (in minutes)
- **`ILIAD_CLEANUP_INTERVAL`**: interval between each position cleanup (in days) after this delay, non updated position will be deleted from the database to make space
- **`ILIAD_SERVER_ADDRESS`**: address server binds to
- **`ILIAD_SERVER_PORT`**: port server binds to
- **`ILIAD_ALLOW_REGISTER`**: whether or not to allow register endpoint, if true, then anyone will be able to make an account on your server, if false, then you will have to manually add new accounts

## Library

To ensure iliad recognizes and manages your audiobooks seamlessly, a particular library structure is requried. First, at the top level of your library folder, you need to put a directory for each audiobook, here's an example of the structure :

```
library
├── the-ancient-city-fustel-de-coulanges
│   ├── info.yaml
│   ├── introduction.opus
│   ├── book1chapter1.opus
│   ├── book1chapter2.opus
│   └── cover.jpg
└── lives-plutarch
```

Every audiobook must have an accompanying `info.yaml` file to guide Iliad. Here is an example : 

```yml
title: "The Ancient City"
author: "Fustel de Coulanges"
date: "1864"

chapters:
  - name: "Introduction"
    file: "introduction.opus"

  - name: "Book First: Ancient Belief - Chapter I: Notions about the Soul and Death"
    file: "book1-chapter1.opus"
```

Though this behaviour is implemented by the client, the rest of the audiobook directory is also important. Each audiobook directory should contain a `cover.{jpg,jpeg,png}` for the cover and files should be at format `opus`, `mp3`, `ogg`, `flac` or `wav`.

## Manual register

There is no fancy user interface to add new user, at least for now. To manually add a new user, we will simply add it to the database directly.

```sh
NEW_USERNAME="your-new-username"
NEW_PASSWORD="your-new-password"
sqlite3 /path/to/database.db "INSERT INTO accounts (username, password, key) VALUES ('${NEW_USERNAME}', '$(mkpasswd \"${NEW_PASSWORD}\")', '$(openssl rand -hex 16)');"
```

Ensure that no two keys are the same, though the probability of that happening is extremely low.

## API Endpoints

In case you want to develop your own client for iliad, here are the API Endpoints that you should use.

- **`Auth`** - While not an api endpoint, many requests require a `Auth: your-api-key` in order to work.
- **`GET /audiobooks`** - Returns the list of all the audiobooks present on the server *(Requires Auth)*.
- **`GET /audiobook/<hash>`** - Returns the binary data of a compressed Gzip Tar archive *(Requires Auth)*.
- **`GET /audiobook/<hash>/position`** - Returns the position information for a given audiobook *(Requires Auth)*.
- **`PUT /audiobooks<hash>/position`** - Updates the position information for a given audiobook *(Requires Auth)* `{ "file"="filename", "position"=42 }`.
- **`POST /audiobooks/login`** - Login using a username and password to get an **api key** `{ "username"="your-username", "password"="your-password-hash" }`.
- **`POST /audiobooks/register`** - Register using a username and password and get an **api key** `{ "username"="your-username", "password"="your-password-hash" }`.

## FAQ

**Will this project support streaming directly ?** No not for now, though i am open to contribution if you want to implement this feature.

## License

This project is licensed under the GPLv3. For more information, check the license file.
