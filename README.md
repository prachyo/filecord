# Filecord

A Git‑style, content‑addressed file hosting service that stores encrypted file chunks and manifests as **Discord messages** (no attachments). Each file/dir lives in its own **thread**; the latest state is discovered via small **HEAD pointer messages**. The local project keeps a **.filecord/** folder (like `.git/`) to cache refs, manifests, and recovery logs; Discord is the **source of truth**.

> **Constraints:** Content‑only (no attachments). Chunks are Base64‑encoded and fit under Discord’s 2,000‑char message limit. Threads may auto‑archive; we persist thread IDs and “bump” to unarchive.

---

## Table of contents
- [Goals](#goals)
- [Glossary](#glossary)
- [Repo layout](#repo-layout)
- [Manifests & message formats](#manifests--message-formats)
- [Commit protocol](#commit-protocol)
- [Discord adapter (rate limits)](#discord-adapter-rate-limits)
- [API & CLI surface](#api--cli-surface)
- [Phase plan](#phase-plan)
- [Testing checklist](#testing-checklist)
- [Security & key management](#security--key-management)
- [Recovery & reconciler](#recovery--reconciler)
- [GC & dedup](#gc--dedup)
- [Build & deps](#build--deps)
- [Roadmap / nice‑to‑haves](#roadmap--nice-to-haves)

---

## Goals
- **Content‑addressed storage** on Discord using message content only.
- **Immutable manifests** with Git‑like history; mutate state by **editing HEAD pointers**.
- **Per‑file/per‑dir threads** for locality and fast listing.
- **Crash‑safe** via a local **write‑ahead log (WAL)** and a **reconciler**.
- **Client‑side encryption** and integrity (chunk + file hashes).
- **No external DB**: use a Git‑style **.filecord/** index; everything is reconstructible from Discord.

---

## Glossary
- **CID**: Content ID, e.g., `sha256:<hex>` or `blake3:<hex>`.
- **HEAD pointer**: Tiny pinned message containing `{ head: <manifest_msg_id>, prev: <...>, node: <abs_path> }`. We **edit** this on updates.
- **Dir manifest**: JSON message listing children (name, type, manifest id, thread id).
- **File manifest**: JSON message listing chunk message IDs, chunk hashes, file hash.
- **Thread**: Discord thread (per file/dir). May auto‑archive; we store its ID and bump when needed.

---

## Repo layout
Local cache (no server DB required):

```
.filecord/
  config.json                     # bot token, guild_id, root_channel_id per user root
  refs/
    roots/<user_id>.head.json     # root dir HEAD pointer (manifest msg id)
    dirs/<abs_path>.head.json     # dir HEAD + thread id
    files/<abs_path>.head.json    # file HEAD + thread id
  threads/
    <node_id>.txt                 # thread_id (node_id = hash of abs_path)
  manifests/
    <manifest_msg_id>.json        # cached copies of manifests
  cids/
    <hash>.msgs                   # newline list of chunk msg_ids (for dedup/GC)
  wal.log                         # write‑ahead log of in‑flight commits
  lost+found/                     # orphans recovered by reconciler
  keys/<root_id>.json             # (optional) encrypted per‑root key metadata
```

`abs_path` uses POSIX style, rooted at the user’s logical root (`/`).

---

## Manifests & message formats

> **Message size budgeting:** Keep headers small. Aim for ~1,300–1,600 Base64 chars per chunk body (≈ ~1.0–1.2 KiB raw) to stay comfortably below 2,000 chars total.

### Chunk message (content‑only)
```text
{"v":1,"type":"chunk","i":42,"cid":"sha256:...","file":"sha256:<whole-file>","enc":"xchacha20p","z":"zstd"}
---BEGIN---
<base64 payload>
---END---
```

### File manifest (pinned in the file thread)
```json
{
  "v": 1,
  "type": "file",
  "name": "trip.jpg",
  "size": 1048576,
  "chunks": [
    {"i":0,"cid":"sha256:...","msg":"123..."},
    {"i":1,"cid":"sha256:...","msg":"124..."}
  ],
  "file_cid": "sha256:<whole-file>",
  "enc": {"alg":"xchacha20poly1305","nonce":"..."},
  "z": "zstd",
  "ctime": 1692230400,
  "mtime": 1692230400,
  "prev": "<old_file_manifest_msg_id_or_null>"
}
```

### Dir manifest (pinned in the dir thread or root channel)
```json
{
  "v": 1,
  "type": "dir",
  "name": "Photos",
  "children": [
    {"t":"dir","name":"2025","manifest":"222...","thread":"555..."},
    {"t":"file","name":"trip.jpg","manifest":"333...","thread":"666..."}
  ],
  "ctime": 1692230400,
  "mtime": 1692230400,
  "prev": "<old_dir_manifest_or_null>"
}
```

### HEAD pointer message (pinned; edited atomically)
```json
{ "head": "<current_manifest_msg_id>", "prev": "<previous_manifest_msg_id>", "node": "/Photos" }
```

---

## Commit protocol
Immutable manifests; mutate by flipping HEAD. Crash‑safe via WAL.

1. **BEGIN (intent)**: append WAL entry `{ op, path, tmp_id, state: "begin" }`.
2. **PREPARE**: compress → encrypt → chunk → hash file; compute `file_cid`.
3. **THREAD**: create per‑file thread if absent; persist thread id.
4. **CHUNKS**: post chunk messages (parallel, bounded). WAL `state:"chunks_posted"`.
5. **FILE MANIFEST**: post + pin; add ✅ reaction. WAL `state:"file_manifest_posted"`.
6. **DIR MANIFEST**: read dir HEAD → set `prev`; post + pin new manifest (child updated). WAL `state:"dir_manifest_posted"`.
7. **HEAD FLIP**: edit dir HEAD pointer `{ head: new, prev: old }`. WAL `state:"head_flipped"`.
8. **FINALIZE**: update `.filecord/refs/*`, cache manifests, update `cids/*`, WAL `state:"done"`.

Rollback/repair is handled by the **reconciler** (see below).

---

## Discord adapter (rate limits)
Implement a single client with automatic backoff/jitter and per‑route buckets.

```cpp
struct Msg { std::string id, channel_id, content; };
struct Thread { std::string id, parent_channel_id; bool archived; };

class DiscordClient {
 public:
  Thread createThread(std::string channel_id, std::string name);
  Msg    postMessage(std::string channel_or_thread_id, std::string content);
  void   editMessage(std::string channel_id, std::string msg_id, std::string new_content);
  void   pinMessage(std::string channel_id, std::string msg_id);
  void   addReaction(std::string channel_id, std::string msg_id, std::string emoji);
  std::vector<Msg> listMessages(std::string channel_or_thread_id, std::string after="", int limit=100);
  Msg    getMessage(std::string channel_id, std::string msg_id);
  Thread getThread(std::string thread_id);   // unarchive if needed
  void   bumpThread(std::string thread_id);  // tiny heartbeat message
};
```

**Concurrency defaults**: uploads 4–8 chunks in parallel per thread; at most 2–4 threads concurrently; retry 429s using `X-RateLimit-Reset-After + jitter`.

---

## API & CLI surface

### Minimal HTTP API (backend)
- `POST /v1/mkdir?path=/a/b`
- `POST /v1/put?path=/a/b/file` (multipart or streaming body)
- `GET  /v1/get?path=/a/b/file` (streams raw bytes)
- `GET  /v1/ls?path=/a/b` (lists children)
- `POST /v1/mv?src=/a/x&dst=/a/y`
- `POST /v1/rm?path=/a/b/file` (marks for GC)
- `POST /v1/reconcile`

### CLI
- `filecord init`
- `filecord mkdir /dir`
- `filecord put ./local.bin /dir/file.bin`
- `filecord get /dir/file.bin > out.bin`
- `filecord ls /dir`
- `filecord mv /a /b`
- `filecord rm /dir/file.bin`
- `filecord verify /dir/file.bin`

---

## Phase plan
Each phase ends with **acceptance criteria** and a runnable state.

### Phase 0 — Bootstrap
**Deliverables**
- `.filecord/` scaffolding & `config.json` template (bot token, guild_id, root_channel_id).
- Helper to write/read simple JSON files with atomic write (temp + rename).

**Acceptance**
- `filecord init` creates `.filecord/` and writes config template.

---

### Phase 1 — Core schemas & codecs (spec only)
**Deliverables**
- JSON schemas (comments or `.md`) for chunk, file manifest, dir manifest, head pointer.
- Decide on `sha256` **or** `blake3` for CIDs; choose `zstd`; choose `xchacha20poly1305`.

**Acceptance**
- Example messages validate by eyeball; unit tests stubbed for encode/decode.

---

### Phase 2 — Discord adapter (REST only)
**Deliverables**
- Minimal REST client with: `postMessage`, `editMessage`, `listMessages`, `pinMessage`, `createThread`, `getThread`, `bumpThread`, `addReaction`.
- **Rate‑limit** handling & retries.

**Acceptance**
- Can create a thread in the root channel and post a message; pins succeed; archived thread can be bumped and read.

---

### Phase 3 — Codec core (C++)
**Deliverables**
- Functions: `compress/decompress (zstd)`, `encrypt/decrypt (xchacha20poly1305)`, `hash (CID)`, Base64 encode/decode, chunker/assembler.
- Unit tests with golden vectors.

**Acceptance**
- Given a buffer, round‑trip: compress→encrypt→chunk→encode→decode→decrypt→decompress equals original; file hash matches.

---

### Phase 4 — Directory scaffolding
**Deliverables**
- Create **root dir HEAD** message in the root channel; pin it; write `.filecord/refs/roots/<user>.head.json`.
- `mkdir` for nested dirs: per‑dir thread, empty dir manifest pinned, dir HEAD pointer pinned/edited; parent dir updated.

**Acceptance**
- `filecord mkdir /a/b/c` produces reachable manifests/threads; `ls /a/b` lists `c` (from manifest).

---

### Phase 5 — Upload small file (PUT)
**Deliverables**
- Full **commit protocol** with WAL.
- Per‑file thread creation; post chunks; post file manifest; update parent dir manifest; flip dir HEAD; cache refs/manifests.

**Acceptance**
- `filecord put ./hello.txt /a/hello.txt` uploads; `ls /a` shows file; manifests pinned; WAL shows `done`.

---

### Phase 6 — Download (GET)
**Deliverables**
- Resolve path → dir HEAD → file manifest; fetch chunk messages (parallel); verify CIDs; decrypt/decompress; stream to stdout.

**Acceptance**
- `filecord get /a/hello.txt > out.txt` matches input (byte‑for‑byte).

---

### Phase 7 — Move/Rename
**Deliverables**
- Update parent dir manifests without altering file thread or file manifest; HEAD flips.

**Acceptance**
- `filecord mv /a/hello.txt /b/hello.txt` works; history preserved; old path disappears from `ls /a`.

---

### Phase 8 — Delete & GC mark
**Deliverables**
- `rm` removes child from dir manifest and flips HEAD; add 🧹 reaction to file manifest.
- GC is not yet deleting chunks; only marking.

**Acceptance**
- `filecord rm /b/hello.txt` hides file from listings; file manifest receives 🧹.

---

### Phase 9 — Reconciler (rebuild from Discord)
**Deliverables**
- BFS from **root dir HEAD**; follow dir/file manifests; repair `.filecord/refs/*`, `threads/*`, `manifests/*`, `cids/*`.
- WAL recovery: detect partial commits and continue or quarantine to `lost+found/`.

**Acceptance**
- Delete `.filecord/` (except config), run `filecord reconcile` → tree is rebuilt; downloads still work.

---

### Phase 10 — GC & optional dedup
**Deliverables**
- Mark‑and‑sweep: mark all reachable msgs (heads, manifests, chunk msgs) starting from root HEAD; sweep unmarked messages **only** for manifests marked 🧹 (or older than a TTL).
- Optional: CID→msg reuse to avoid re‑posting duplicate chunks.

**Acceptance**
- After uploads/deletes, `filecord gc` removes unreferenced chunks safely; verify survivors.

---

### Phase 11 — Verify & audit
**Deliverables**
- `verify` command re‑fetches chunks (or uses local cache) and recomputes hashes; reports mismatches; can auto‑rebuild file manifests if order or chunk mapping is wrong.

**Acceptance**
- Corrupt one chunk message manually → `verify` detects and reports.

---

### Phase 12 — API server (optional control plane)
**Deliverables**
- Lightweight HTTP server (Drogon) providing `/v1/*` endpoints that call the same library as CLI.

**Acceptance**
- `curl -T file.bin "http://localhost:8080/v1/put?path=/x/file.bin"` works.

---

### Phase 13 — Security & keys
**Deliverables**
- Per‑root key file in `.filecord/keys/` protected by a passphrase; envelope encryption.
- Key rotation command.

**Acceptance**
- New root created with encryption on; rotate succeeds; old files still decrypt (dual‑key read until re‑encrypt).

---

### Phase 14 — Performance & polish
**Deliverables**
- Tunable concurrency; progress bars; structured logging; metrics.
- “Bump on access” for archived threads; backoff tuning.

**Acceptance**
- Large directory trees list quickly; uploads respect rate limits without flapping.

---

## Testing checklist
- **Unit**: chunking, Base64, zstd, crypto, JSON encode/decode, CID calc.
- **Integration**:
  - Upload small + medium files; download & compare.
  - Nested dirs create/list.
  - Simulate 429s (mock) → retries OK.
  - Archive threads → auto‑bump restores access.
  - Crash mid‑commit (kill process) → WAL + reconciler completes.
  - Corrupt local refs → reconciler rebuilds from Discord.
- **Property tests**: random chunk sizes, rename storms, concurrent puts in same dir (optimistic merge of children).

---

## Security & key management
- **Client‑side encryption** (`xchacha20poly1305`).
- Key derivation: per‑root master key; per‑file nonce; include alg/nonce in file manifest.
- Protect `.filecord/keys/*` via OS file perms; store passphrase only in memory.
- **Integrity**: verify chunk CIDs and whole‑file `file_cid` during GET and VERIFY.

> **ToS note:** Discord is not a general object store; keep usage within limits; provide an adapter layer to swap to S3/GCS later.

---

## Recovery & reconciler
High‑level algorithm:
1. Discover **root** from `.filecord/refs/roots/*` or channel topic → read **root HEAD**.
2. Read root dir manifest; enqueue children.
3. For each dir: read manifest; for each child: if `t=file`, read **file HEAD** → file manifest; collect chunks.
4. Rebuild `.filecord/refs/*`, `threads/*`, `cids/*`, `manifests/*`.
5. Compare local `refs` vs on‑Discord HEAD messages; overwrite local with Discord truth.
6. Inspect WAL entries not `done`; continue from next step or move unknown artifacts to `lost+found/`.

---

## GC & dedup
- **Mark**: from all live heads and manifests, mark reachable chunk `msg_id`s.
- **Sweep**: delete unmarked chunks only if their file manifest is 🧹 (or older than TTL).
- **Dedup (optional)**: maintain `cids/<hash>.msgs`; when uploading a chunk, reuse existing `msg_id` if accessible.

---

## Build & deps

### Toolchain
- **C++17+**, **CMake**
- Package manager: **vcpkg** (or Conan)

### Libraries
- **Discord**: DPP (aka D++) *or* Boost.Beast/libcurl for REST
- **JSON**: nlohmann/json
- **Crypto**: libsodium (xchacha20‑poly1305) or OpenSSL
- **Compression**: zstd
- **Hashing**: BLAKE3 (fast) or OpenSSL SHA‑256

### `vcpkg.json` (example)
```json
{
  "name": "filecord",
  "version-string": "0.1.0",
  "dependencies": [
    "dpp",
    "nlohmann-json",
    "libsodium",
    "zstd",
    "openssl",
    "cpr"
  ]
}
```

### `CMakeLists.txt` (skeleton)
```cmake
cmake_minimum_required(VERSION 3.20)
project(filecord LANGUAGES CXX)
set(CMAKE_CXX_STANDARD 17)
find_package(nlohmann_json CONFIG REQUIRED)
find_package(unofficial-sodium CONFIG REQUIRED)
find_package(ZSTD CONFIG REQUIRED)
find_package(OpenSSL REQUIRED)
find_package(cpr CONFIG REQUIRED)
# Or: find_package(dpp CONFIG REQUIRED)
add_library(filecord_core
  src/codec.cpp src/discord_client.cpp src/manifests.cpp src/reconciler.cpp src/wal.cpp)
target_link_libraries(filecord_core
  PRIVATE nlohmann_json::nlohmann_json unofficial-sodium::sodium ZSTD::ZSTD OpenSSL::SSL cpr::cpr)
add_executable(filecord_cli src/cli.cpp)
target_link_libraries(filecord_cli PRIVATE filecord_core)
```

---

## Roadmap / nice‑to‑haves
- **Snapshots/time‑travel**: mount any historical dir manifest via `prev` chain.
- **Shareable capabilities**: read‑only tokens that reference a manifest.
- **S3/GCS adapter**: same manifest format; switch storage backend easily.
- **Partial reads**: chunk‑aligned byte‑range GET.
- **Web UI**: Next.js explorer, drag‑drop uploads, integrity badges.
- **Observability**: structured logs, metrics, trace IDs per commit.

---

## Acceptance gates (condensed)
- [ ] Phase 0: `.filecord/` init
- [ ] Phase 2: can post/pin/bump in Discord
- [ ] Phase 3: codec round‑trip green
- [ ] Phase 4: mkdir + ls via manifests
- [ ] Phase 5: put small file end‑to‑end
- [ ] Phase 6: get verifies bytes
- [ ] Phase 9: reconcile rebuilds from scratch
- [ ] Phase 10: gc removes unreferenced safely
- [ ] Phase 13: encryption on by default

---

### Quickstart (after Phase 6)
```bash
filecord init --guild <id> --root-channel <id>
filecord mkdir /docs
filecord put ./README.md /docs/README.md
filecord ls /docs
filecord get /docs/README.md > OUT.md
```

If a thread is archived, operations auto‑bump, then proceed. All state is recoverable from pinned **HEAD** and manifest chains.
