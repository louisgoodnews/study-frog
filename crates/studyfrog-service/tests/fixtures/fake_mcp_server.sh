#!/bin/sh
# A tiny fake MCP server for the tests (STORY-0121): one JSON-RPC message per line on standard input,
# one answer per line on standard output. It offers a read-only tool `lookup` and a tool `save` that
# says nothing about itself, answers unknown methods with an error, and writes a log line that is
# not JSON before such an error (a client must skip it).
while IFS= read -r line; do
  id=$(printf '%s' "$line" | sed -n 's/.*"id":\([0-9]*\).*/\1/p')
  case "$line" in
    *'"notifications/'*) continue ;;
    *'"initialize"'*) r='{"protocolVersion":"2025-06-18","capabilities":{"tools":{}},"serverInfo":{"name":"fake","version":"0.1"}}' ;;
    *'"tools/list"'*) r='{"tools":[{"name":"lookup","description":"Find a paper","inputSchema":{"type":"object","properties":{"q":{"type":"string"}}},"annotations":{"readOnlyHint":true}},{"name":"save","description":"Save a citation","inputSchema":{"type":"object"}}]}' ;;
    *'"resources/list"'*) r='{"resources":[{"uri":"fake://library","name":"Library"}]}' ;;
    *'"prompts/list"'*) r='{"prompts":[{"name":"cite"}]}' ;;
    *'"tools/call"'*)
      tool=$(printf '%s' "$line" | sed -n 's/.*"name":"\([a-z]*\)".*/\1/p')
      r="{\"content\":[{\"type\":\"text\",\"text\":\"ran $tool\"}],\"isError\":false}" ;;
    *)
      echo "fake server: no such method"
      printf '{"jsonrpc":"2.0","id":%s,"error":{"code":-32601,"message":"no such method"}}\n' "$id"
      continue ;;
  esac
  printf '{"jsonrpc":"2.0","id":%s,"result":%s}\n' "$id" "$r"
done
