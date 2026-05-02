# Writing a Client for Iliad

This document is the authoritative guide for building a player or client that
talks to an iliad server. It covers the full API contract, the position sync
algorithm, and the operational patterns that make a client reliable.

---

## Base URL and Content Type

All requests go to `http://<host>:<port>` (default port 9090). All request and
response bodies are JSON. Set `Content-Type: application/json` on every
request that has a body.

---

## Authentication

### Token model

Iliad has two token scopes: **regular** and **admin**. Players only need
regular tokens. Admin tokens are for library management (scan, cleanup) and are
not required for playback.

Tokens are opaque strings. They expire server-side after a configurable TTL
(default 24 hours). The server wipes all tokens on restart — clients must
handle re-authentication transparently.

### Login

```
POST /auth/login
Content-Type: application/json

{ "username": "string", "password": "string" }
```

Response `200`:

```json
{ "token": "string" }
```

Store this token. Attach it to every subsequent request:

```
Authorization: Bearer <token>
```

### Registration

```
POST /auth/register
Content-Type: application/json

{ "username": "string", "password": "string" }
```

If the server has `ILIAD_PUBLIC_REGISTER=false` (the default), this endpoint
requires an admin token. If you are building a self-service client, inform the
user that an admin must create their account first.

On success returns `{ "token": "string" }` — the same shape as login.

### Token expiry and 401 handling

Any endpoint can return `401 Unauthorized` when:
- The token is missing or malformed.
- The token has expired (server-side TTL).
- The server restarted and wiped its token maps.

**Clients must re-authenticate automatically.** The correct pattern:

1. Send request with stored token.
2. On `401`, call `POST /auth/login` with stored credentials.
3. Store the new token.
4. Retry the original request once.
5. If the retry also returns `401`, surface an error to the user (wrong
   password or account deleted).

Never store admin credentials in a player. Never show the user a raw 401 —
retry silently first.

---

## Library

### List audiobooks

```
GET /audiobooks
Authorization: Bearer <token>
```

Response `200`:

```json
[
  {
    "hash": "string",
    "title": "string",
    "author": "string",
    "date": 2021,
    "genres": ["fiction", "thriller"],
    "duration": 123456,
    "archive_ready": true
  }
]
```

`hash` is the stable identifier for a book. It is derived from the book's
content and does not change between scans as long as the source files are
unchanged.

`date` is the publication year as an integer.

`genres` is the list of genres from the book's `info.yml`.

`duration` is the total playback duration in milliseconds.

`archive_ready` indicates whether the downloadable `.tar.gz` archive has been
built. A newly scanned book may not be ready immediately — see the download
section.

### Get audiobook details

```
GET /audiobooks/<hash>
Authorization: Bearer <token>
```

Response `200`:

```json
{
  "hash": "string",
  "title": "string",
  "author": "string",
  "date": 2023,
  "description": "string",
  "genres": ["string"],
  "duration": 123456789,
  "size": 987654321,
  "archive_ready": true
}
```

`duration` is total playback duration in **milliseconds**.

`size` is archive size in **bytes**.

`date` is a publication year (integer, not a timestamp).

---

## Downloading a Book

```
GET /audiobooks/<hash>/download
Authorization: Bearer <token>
```

Response `200`: binary stream, `Content-Type: application/octet-stream`.
The filename header follows the pattern `author-slug-title-slug-year.tar.gz`.

Response `503 Service Unavailable`: the archive is not yet built. Sending this
request automatically promotes the book to the front of the archive build
queue. The client should:

1. Display a "preparing download" state.
2. Poll `GET /audiobooks/<hash>` (not the download endpoint) until
   `archive_ready` is `true`.
3. Then retry the download.

Do not hammer the download endpoint in a loop on 503. Poll the detail endpoint
instead — it is cheap and does not consume queue priority.

### Archive format

The archive is a `.tar.gz` containing the chapter audio files and the
`info.yaml` metadata file. Extract with standard tar:

```sh
tar -xzf book.tar.gz
```

Chapter order in the archive matches the `chapters` list in `info.yaml`. File
names are as declared in the `path` fields of that YAML.

---

## Playback Position

Position lets the server remember where a user stopped. It is per-user,
per-book. A client that never calls the position endpoints still works; the
user just loses progress across devices or reinstalls.

### Units

- `chapter_index`: zero-based integer index into the book's chapter list.
- `chapter_position`: position within the chapter in **milliseconds**.
- `timestamp`: Unix timestamp in **seconds** (not milliseconds).

### Get position

```
GET /positions/<hash>
Authorization: Bearer <token>
```

Response `200`:

```json
{
  "chapter_index": 0,
  "chapter_position": 0,
  "timestamp": 1714521600
}
```

`timestamp` is the Unix timestamp (seconds) of the last position update. It is
`0` if the user has never listened to this book.

If the user has never listened to this book, the server returns `0, 0, 0` — there
is no 404 for a missing position. Always call this when opening a book to
resume from the last known position.

### Set position

```
PUT /positions/<hash>
Authorization: Bearer <token>
Content-Type: application/json

{
  "chapter_index": 2,
  "chapter_position": 180000,
  "timestamp": 1714521600
}
```

Response `200` with no body.

`timestamp` must be the wall-clock time at which the user was at this
position — not the time of the HTTP request, though in practice they are
close. Use the current Unix time in seconds.

### The position merge algorithm

The rule is simple: **latest `timestamp` wins.**

The server accepts the incoming position if `timestamp` is strictly
greater than the stored timestamp. Otherwise it discards the update silently
and returns `200` either way.

Every position change — playing forward, skipping, rewinding, scrubbing — is
a deliberate user action that happened at a real wall-clock time. The most
recent action is authoritative. There is no forward-progress bias and no
special case for rewinds. A user who scrubs back to an earlier chapter
generates a new timestamp for that action; that timestamp wins over any older
position on any other device.

One guard: the server rejects `timestamp` values more than 300 seconds
in the future (compared to server wall clock) with `400 Bad Request`. This
prevents a device with a misconfigured fast clock from permanently locking a
position.

**What this means for clients:**

- Send an accurate wall-clock Unix timestamp in seconds. Do not manipulate it.
- A `200` does not mean the server stored your position. It may have been
  discarded as older. This is correct; do not retry aggressively.
- All user actions (skip, rewind, scrub) are handled identically — they carry
  their own timestamp and win or lose on that basis alone.

### The fetch-before-play protocol

This is the most important client-side rule:

**Always fetch the server position before allowing playback to start.**

```
open book
  → GET /positions/<hash>
  → seek to returned position
  → allow play
```

Never start generating position timestamps from a locally cached state without
first loading the server's current position. If you play from a stale local
cache and generate new (current) timestamps for it, those timestamps will beat
any progress made on other devices — silently and incorrectly.

The correct mental model: local cached position is a hint for the UI while the
fetch is in flight, never an input to playback itself.

### Offline handling

If the server is unreachable when the user opens a book:

1. Display the locally cached position. Allow playback from it.
2. Track that this is an **offline session** started from a potentially stale
   position.
3. When connectivity returns, before syncing any progress: fetch the server
   position and compare it to where your offline session started.
   - If the server position is ahead of your session's starting point, another
     device has been used more recently. Offer the user a choice: jump to the
     server's position, or keep their offline progress.
   - If the server position is at or behind your session's starting point,
     your offline progress is ahead. Sync normally.

Do not blindly sync offline progress when reconnecting. Fetch first, then
decide.

### Position sync cadence

- On book open: fetch position (mandatory, before play).
- During playback: send every 30 seconds.
- On pause, chapter change, app backgrounded, app closed: send immediately.

Do not send on every frame or every second.

---

## Error Reference

| Status | Meaning |
|--------|---------|
| `200` | Success |
| `401` | Missing, invalid, or expired token. Re-authenticate. |
| `404` | Resource not found (bad hash, etc.). |
| `409` | Conflict (e.g. username already registered). |
| `500` | Server-side error. Log it, surface a generic message. |
| `503` | Archive not ready. Poll `archive_ready`, then retry. |

Error bodies are plain text strings (not JSON). Do not parse them as JSON.

---

## Multi-Device Behavior

Iliad's position sync is designed for multi-device use. The merge algorithm
means:

- The furthest-ahead recent session wins.
- A fresh listen on one device will not clobber progress on another device
  that is far ahead, as long as the timestamps are honest.
- Two devices listening simultaneously will converge to whichever sends the
  later timestamp — which will usually be the one that is actually further
  along.

Clients do not need to implement their own conflict resolution. Trust the
server's merge and always send accurate timestamps.

---

## Practical Checklist

A well-written client does all of the following:

- [ ] Stores the token and reuses it across sessions.
- [ ] Automatically re-authenticates on `401` and retries once.
- [ ] Fetches position on book open and resumes from `chapter_index` /
      `chapter_position`.
- [ ] Syncs position every 30 seconds during playback.
- [ ] Syncs position immediately on pause, chapter change, background, and
      close.
- [ ] Sends accurate wall-clock Unix timestamps (seconds) with every position
      update.
- [ ] Checks `archive_ready` before attempting download; polls it until true
      rather than retrying the download endpoint.
- [ ] Treats `503` on download as a queue request, not an error — shows
      "preparing" UI state.
- [ ] Handles network unavailability gracefully — buffers last position, sends
      on reconnect with original timestamp.
- [ ] Does not expose raw HTTP errors to the user.
