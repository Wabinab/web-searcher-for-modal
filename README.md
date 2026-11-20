# Web Search, Web Fetch and Image Search tools
An exe for my [modal-hosted AI](https://modal.com/) to access the web, via Parallel. 
When Parallel isn't available for whatever reason, `web_search` and `web_fetch` fall back to Ollama. 
`image_search` has no Ollama equivalent, so it does not fall back.

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

**Exit code is the success signal, not stderr.** When Parallel fails and the Ollama fallback succeeds, the exe still exits 0 with results on stdout, but writes a line like `parallel web_fetch failed, falling back to ollama: ...` to stderr. Don't treat any stderr output as failure.

### Tool call format

```json
{
  "id": "call_7945d98c45dd4e0583c04db8",
  "name": "web_search",
  "arguments": "{\"objective\": \"...\", \"search_queries\": [\"...\"]}"
}
```

- `id`: optional. The tool call ID from the AI. Nothing reads it, and a missing `id` is accepted.
- `name`: required. `web_search`, `web_fetch` or `image_search`. Anything else returns an error.
- `arguments`: required. The arguments as a **stringified** JSON object (OpenAI style) or a nested JSON object (Ollama style). Both are accepted.

The arguments are not inferred from their shape. The exe dispatches only on `name`, so bare arguments with no envelope (`{"urls": [...], "objective": "..."}`) are rejected with an error listing the top-level keys it received.

Unknown fields inside `arguments` are ignored.

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

**Output:** a JSON array of `{ "title", "url", "content" }`. Parallel results may also carry `publish_date` (`YYYY-MM-DD`), omitted if unknown. Ollama fallback results never have it.

**Fallback:** if Parallel fails, each query (the `objective` plus every `search_queries` entry, up to 10) is searched on Ollama. Queries that fail individually are skipped. If every query fails, the exe exits 1 and the error contains both the Parallel and Ollama messages.

### web_fetch

Fetch and extract relevant content from specific web URLs. Use only when web_search excerpts are insufficient, or when the user asked about a specific URL or page, or you need exact wording or quotes that excerpts may have truncated, or a full-page analysis, or when web_search excerpts are conflicting or clearly insufficient to answer.

| Argument | Type | Required | Description |
|---|---|---|---|
| `objective` | string | Yes | What to look for in the pages. Maximum 200 characters (not enforced by this exe; Parallel may reject longer ones). |
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
| `publish_date` | string | `YYYY-MM-DD`. Omitted if unknown (always omitted on Ollama fallback). |
| `links` | string[] | Omitted if empty. **Meaning depends on the backend:** from Parallel it is a single-element array holding the page's own URL; from the Ollama fallback it holds the links found on the page. |

```json
[
  {
    "title": "text-splitter - crates.io: Rust Package Registry",
    "content": "# text-splitter v0.30.1\nSplit text into semantic chunks, ...",
    "links": ["https://crates.io/crates/text-splitter/0.30.1"]
  }
]
```

**Fallback:** if Parallel fails, each URL is fetched on Ollama one by one. URLs that fail individually are skipped silently, so the output can contain fewer pages than `urls`. If every URL fails, the exe exits 1 and the error contains both the Parallel and Ollama messages.

### image_search

Find images on the web. Returns direct image URLs, the page each image appears on, the page title, and pixel dimensions when known. Results link to images but do not describe them; for information about a subject, use `web_search` instead.

| Argument | Type | Required | Description |
|---|---|---|---|
| `objective` | string | Yes | Natural-language description of the images to find, including subject, style, or context that should rank them, such as "official headshot" or "diagram, not a photo". |
| `search_queries` | string[] | Yes | 1-3 keyword queries of 2-6 words each, up to 200 characters each. |

```bash
cat <<'EOF' | ./<BINARY>
{
  "name": "image_search",
  "arguments": {"objective": "diagram of a transformer, not a photo", "search_queries": ["transformer architecture diagram"]}
}
EOF
```

**Output:** a JSON array of `{ "image_url", "source_page_url", "title", "width", "height" }`. Only `image_url` is guaranteed; the other fields are omitted when Parallel doesn't return them.

**Limitations:**
- `mode` is always `fast`. Parallel's `advanced` mode needs an API key or sign-in and returns a tool error without one, so a `mode` sent by the caller is ignored.
- `override` and `advanced_settings` (e.g. `location`, `max_results`) are not supported. Per Parallel's docs they are server-side config sent in the `x-parallel-mcp-config` header, which requires credentials. Fields sent by the caller are dropped.
- No fallback. Any Parallel failure exits 1 with the error message.

## Errors

All errors go to stderr as `Error: BadRequest: ...` with exit code 1. The message names the layer that rejected the input:

| Message starts with | Meaning |
|---|---|
| `invalid tool-call envelope` | Stdin wasn't a valid `{name, arguments}` object. Lists the top-level keys it saw. |
| `<tool> args invalid` | The envelope was fine but `arguments` failed validation. Line/column refer to the arguments payload, not the envelope. |
| `Unknown tool call` | `name` wasn't one of the three tools. |
| `<fn>: Parallel tool returned isError` | Parallel returned a tool-level error (HTTP 200). |
| `<fn>: JSON-RPC error` | Parallel returned a JSON-RPC error object. |
| `web_search failed` / `web_fetch failed` | Parallel and the Ollama fallback both failed. |

Argument and envelope errors never trigger the Ollama fallback, because there's nothing valid to retry with.

## Testing

Unit tests:

```bash
cargo test                      # offline tests
cargo test -- --ignored         # also run live tests (network required)
```

Black-box tests against the compiled binary (Python 3 standard library only, no `requirements.txt`):

```bash
python3 test_binary.py          # offline: envelope and argument validation
python3 test_binary.py --live   # also calls Parallel/Ollama over the network
python3 test_binary.py --bin path/to/exe
```

The default binary path is `./target/x86_64-unknown-linux-musl/release/web-searcher`; override with `--bin` or the `WEB_SEARCHER_BIN` environment variable. Run this after every release build, since passing `cargo test` doesn't prove the shipped binary works. The live run prints any stderr output, so a silent Parallel failure that fell back to Ollama is visible.

`test_binary.py` contains a copy of the backend's tool schemas (`TOOLS`) to check that every tool the model can see is accepted by the exe. Keep it in sync with the backend when schemas change.