# Web Search and Web Fetch tools
An exe for my [modal-hosted AI](https://modal.com/) to access the web, via Parallel. 
When Parallel isn't available for whatever reason, it falls back to Ollama. 

## Compilation: 
Install `musl`:

```bash
sudo apt install musl-tools
rustup target add x86_64-unknown-linux-musl
```

Then build project
```bash
cargo build --release --target x86_64-unknown-linux-musl
```

## Usage:

The exe reads **one tool call as JSON from stdin** and prints the result to stdout. On failure it prints `Error: ...` to stderr and exits with code 1.

```bash
echo '<tool-call-json>' | ./target/x86_64-unknown-linux-musl/release/<BINARY>
```

### Tool call format

```json
{
  "id": "call_7945d98c45dd4e0583c04db8",
  "name": "web_search",
  "arguments": "{\"objective\": \"...\", \"search_queries\": [\"...\"]}"
}
```

- `id`: the tool call ID from the AI.
- `name`: `web_search` or `web_fetch`. Anything else returns an error.
- `arguments`: the arguments as a **stringified** JSON object, not a nested object.

### web_search

Search the web for information. Use this when you need current or factual information, for research comparison, documentation, and troubleshooting questions. Multiple `search_queries` can be issued in a single call.

| Argument | Type | Required | Description |
|---|---|---|---|
| `objective` | string | Yes | One-sentence goal of this search. Keep it to one goal. May name preferred sources or how recent results should be. |
| `search_queries` | string[] | Yes | Keyword queries of 3-6 words each. At least one is required; 2-3 work best. The `site:` operator takes a bare domain, not a URL path. Do not use quoted phrases. |

```bash
cat <<'EOF' | ./<BINARY>
{
  "id": "call_1",
  "name": "web_search",
  "arguments": "{\"objective\": \"Find github repositories for text-splitter and semchunk-rs\", \"search_queries\": [\"github text-splitter rust\", \"github semchunk-rs\"]}"
}
EOF
```

**Output:** a JSON array of `{ "title", "url", "content" }`.

### web_fetch

Fetch and extract relevant content from specific web URLs. Use only when web_search excerpts are insufficient, or when the user asked about a specific URL or page, or you need exact wording or quotes that excerpts may have truncated, or a full-page analysis, or when web_search excerpts are conflicting or clearly insufficient to answer.

| Argument | Type | Required | Description |
|---|---|---|---|
| `objective` | string | Yes | What to look for in the pages. Maximum 200 characters. |
| `urls` | string[] | Yes | HTTP or HTTPS URLs to read. Maximum 20 per call. |
| `search_queries` | string[] | No | Keyword queries of 3-6 words each that focus the excerpts. When fetching results from `web_search`, pass the same queries. |

```bash
cat <<'EOF' | ./<BINARY>
{
  "id": "call_2",
  "name": "web_fetch",
  "arguments": "{\"urls\": [\"https://crates.io/crates/text-splitter/0.30.1\"], \"objective\": \"Get features and usage of the text-splitter crate\", \"search_queries\": [\"text-splitter crate features\"]}"
}
EOF
```

**Output:** a JSON array with one object per page.

| Field | Type | Description |
|---|---|---|
| `title` | string or null | Page title. `null` if unavailable. |
| `content` | string | The page's relevant excerpts as markdown, joined with newlines. |
| `publish_date` | string | `YYYY-MM-DD`. Omitted if unknown. |
| `links` | string[] | See the note below. Omitted if empty. |

```json
[
  {
    "title": "text-splitter - crates.io: Rust Package Registry",
    "content": "# text-splitter v0.30.1\nSplit text into semantic chunks, ...",
    "links": ["https://crates.io/crates/text-splitter/0.30.1"]
  }
]
```