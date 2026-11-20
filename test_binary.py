#!/usr/bin/env python3
"""Black-box tests for the compiled web-searcher binary. Stdlib only, no requirements.txt.

Usage:
  python3 test_binary.py                      # offline tests only (no network needed)
  python3 test_binary.py --live               # also hit Parallel/Ollama over the network
  python3 test_binary.py --bin path/to/exe    # override binary path (or env WEB_SEARCHER_BIN)

TOOLS below is a COPY of the backend AI's tool schemas, used to check that every tool the model
can see is accepted by the binary. It only catches drift if you keep it in sync with the backend.
"""
import argparse
import json
import os
import subprocess
import sys

DEFAULT_BIN = "./target/x86_64-unknown-linux-musl/release/web-searcher"
FETCH_URL = "https://docs.parallel.ai/getting-started/rate-limits"

# ---- Copy of the backend's tool schemas (keep in sync) ----
TOOLS = [
  {"type": "function", "function": {
    "name": "web_search",
    "description": (
      "Search the web for information. Use this when you need current or factual information, "
      "for research comparison, documentation, and troubleshooting questions. "
      "Multiple `search_queries` can be issued in a single call."),
    "parameters": {
      "type": "object",
      "properties": {
        "objective": {"type": "string", "description":
          "One-sentence goal of this search. Keep it to one goal. May name preferred sources or how recent results should be."},
        "search_queries": {"type": "array", "items": {"type": "string"}, "description":
          "Keyword queries of 3-6 words each. At least one required; 2-3 work best. "
          "The `site:` operator takes a bare domain, not a URL path. Do not use quoted phrases."},
      },
      "required": ["objective", "search_queries"],
    },
  }},
  {"type": "function", "function": {
    "name": "web_fetch",
    "description": (
      "Fetch and extract relevant content from specific web URLs. Use only when "
      "web_search excerpts are insufficient, or when the user asked about a specific URL or page, "
      "or you need exact wording or quotes that excerpts may have truncated, or a full-page analysis, "
      "or when web_search excerpts are conflicting or clearly insufficient to answer."),
    "parameters": {
      "type": "object",
      "properties": {
        "objective": {"type": "string", "description":
          "What to look for in the pages. Maximum 200 characters."},
        "urls": {"type": "array", "items": {"type": "string"}, "description":
          "HTTP or HTTPS URLs to read. Maximum 20 per call."},
        "search_queries": {"type": "array", "items": {"type": "string"}, "description":
          "Optional. Keyword queries of 3-6 words each that focus the excerpts. "
          "When fetching results from web_search, pass the same queries."},
      },
      "required": ["objective", "urls"],
    },
  }},
  {"type": "function", "function": {
    "name": "image_search",
    "description": (
      "Find images on the web. Returns direct image URLs, the page each image appears on, "
      "the page title, and pixel dimensions when known. Results link to images but do not "
      "describe them; for information about a subject, use web_search instead. "
      "Use only when the user needs to see or obtain an image."),
    "parameters": {
      "type": "object",
      "properties": {
        "objective": {"type": "string", "description":
          "Natural-language description of the images to find, including subject, style, or context "
          "that should rank them, such as 'official headshot' or 'diagram, not a photo'."},
        "search_queries": {"type": "array", "items": {"type": "string"}, "description":
          "1-3 keyword queries of 2-6 words each, up to 200 characters each."},
      },
      "required": ["objective", "search_queries"],
    },
  }},
]
# ---- end copy ----

results = []


def run(bin_path, stdin_text, timeout=90):
  p = subprocess.run([bin_path], input=stdin_text, capture_output=True, text=True, timeout=timeout)
  return p.returncode, p.stdout, p.stderr


def env(name, args, as_string=False, with_id=False):
  e = {"name": name, "arguments": json.dumps(args) if as_string else args}
  if with_id:
    e["id"] = "call_1"
  return json.dumps(e)


def check(label, cond, detail=""):
  results.append((label, bool(cond)))
  print(f"  [{'PASS' if cond else 'FAIL'}] {label}" + ("" if cond else f"\n         {detail}"))


def expect_error(bin_path, label, stdin_text, *needles):
  code, out, err = run(bin_path, stdin_text)
  ok = code != 0 and all(n in err for n in needles)
  check(label, ok, f"exit={code} stdout={out[:200]!r} stderr={err[:400]!r}")


def expect_json_list(bin_path, label, stdin_text, required_keys):
  code, out, err = run(bin_path, stdin_text)
  try:
    data = json.loads(out)
  except json.JSONDecodeError:
    check(label, False, f"exit={code} stdout not JSON: {out[:200]!r} stderr={err[:400]!r}")
    return None
  ok = (code == 0 and isinstance(data, list) and len(data) > 0
        and all(isinstance(r, dict) and all(k in r for k in required_keys) for r in data))
  check(label, ok, f"exit={code} stderr={err[:400]!r} first={str(data)[:300]}")
  if err.strip():
    print(f"         (stderr: {err.strip()[:300]})")  # e.g. a Parallel failure that fell back to Ollama
  return data


def minimal_args(schema):
  """Build minimal valid args from a JSON-schema 'parameters' block (required fields only)."""
  props, args = schema["properties"], {}
  for key in schema.get("required", []):
    t = props[key]["type"]
    if t == "string":
      args[key] = "test"
    elif t == "array":
      args[key] = [FETCH_URL] if key == "urls" else ["rate limits test"]
    else:
      raise ValueError(f"unhandled schema type {t} for {key}")
  return args


def offline(b):
  print("\n== Offline: envelope / argument validation ==")
  expect_error(b, "bare args (no envelope) -> envelope error naming keys",
               json.dumps({"urls": [FETCH_URL], "objective": "x"}), "invalid tool-call envelope", "name", "urls")
  expect_error(b, "empty input -> error", "", "invalid tool-call envelope")
  expect_error(b, "not JSON -> error", "hello", "invalid tool-call envelope")
  expect_error(b, "missing arguments -> error", json.dumps({"name": "web_fetch"}), "arguments")
  expect_error(b, "unknown tool -> clear error", env("web_crawl", {}), "Unknown tool call", "web_crawl")
  expect_error(b, "objective=-1 -> arg type error naming the tool",
               env("web_fetch", {"urls": [FETCH_URL], "objective": -1}), "web_fetch", "invalid type")
  expect_error(b, "web_fetch missing urls -> arg error",
               env("web_fetch", {"objective": "x"}), "web_fetch", "urls")
  expect_error(b, "web_search missing search_queries -> arg error",
               env("web_search", {"objective": "x"}), "web_search", "search_queries")
  expect_error(b, "image_search missing objective -> arg error",
               env("image_search", {"search_queries": ["q"]}), "image_search", "objective")


def live(b):
  print("\n== Live: network required ==")
  print(" -- envelope shapes (the id/arguments variants that broke before) --")
  expect_json_list(b, "web_fetch: no id, arguments as object",
                   env("web_fetch", {"urls": [FETCH_URL], "objective": "Find the rate limits"}), ["content"])
  expect_json_list(b, "web_fetch: with id, arguments as JSON string",
                   env("web_fetch", {"urls": [FETCH_URL], "objective": "Find the rate limits"},
                       as_string=True, with_id=True), ["content"])
  expect_json_list(b, "web_fetch: with search_queries",
                   env("web_fetch", {"urls": [FETCH_URL], "objective": "Find the rate limits",
                                     "search_queries": ["parallel api rate limits"]}), ["content"])

  print(" -- per-tool output shape --")
  expect_json_list(b, "web_search -> [{title,url,content}]",
                   env("web_search", {"objective": "Find Parallel API rate limits",
                                      "search_queries": ["parallel.ai rate limits", "parallel api quota"]}),
                   ["title", "url", "content"])
  imgs = expect_json_list(b, "image_search -> [{image_url,...}]",
                          env("image_search", {"objective": "diagram of a transformer, not a photo",
                                               "search_queries": ["transformer architecture diagram"]}),
                          ["image_url"])
  if imgs:
    check("image_search: all image_url are http(s)", all(r["image_url"].startswith("http") for r in imgs))
  expect_json_list(b, "image_search: caller 'mode':'advanced' is ignored (still succeeds)",
                   env("image_search", {"objective": "golden gate bridge", "search_queries": ["golden gate bridge"],
                                        "mode": "advanced"}), ["image_url"])
  expect_json_list(b, "image_search: unknown caller fields (override/advanced_settings) are dropped",
                   env("image_search", {"objective": "golden gate bridge", "search_queries": ["golden gate bridge"],
                                        "override": {"advanced_settings": {"location": "us"}},
                                        "advanced_settings": {"max_results": 20}}), ["image_url"])

  print(" -- every TOOLS schema is accepted by the binary --")
  for t in TOOLS:
    fn = t["function"]
    keys = {"web_search": ["title", "url", "content"], "web_fetch": ["content"], "image_search": ["image_url"]}.get(fn["name"])
    if keys is None:
      check(f"TOOLS['{fn['name']}'] is known to the binary", False, "add it to handle_tool_call and this harness")
      continue
    expect_json_list(b, f"TOOLS['{fn['name']}'] minimal args from schema",
                     env(fn["name"], minimal_args(fn["parameters"]), as_string=True), keys)


def main():
  ap = argparse.ArgumentParser()
  ap.add_argument("--bin", default=os.environ.get("WEB_SEARCHER_BIN", DEFAULT_BIN))
  ap.add_argument("--live", action="store_true", help="also run network tests")
  a = ap.parse_args()
  if not os.path.isfile(a.bin):
    sys.exit(f"binary not found: {a.bin}\nBuild: cargo build --release --target x86_64-unknown-linux-musl")
  offline(a.bin)
  if a.live:
    live(a.bin)
  else:
    print("\n(skipping network tests; pass --live)")
  failed = [l for l, ok in results if not ok]
  print(f"\n{len(results) - len(failed)}/{len(results)} passed")
  for l in failed:
    print(f"  FAILED: {l}")
  sys.exit(1 if failed else 0)


if __name__ == "__main__":
  main()