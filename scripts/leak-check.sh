#!/usr/bin/env bash
set -u

MARKER="${1:-}"
VAULT_DIR="${2:-}"
BUNDLE_ID="com.techiesolo.znotes"

if [[ -z "$MARKER" ]]; then
  echo "usage: $0 <marker> [vault_dir]" >&2
  exit 2
fi

dirs=()
[[ -n "$VAULT_DIR" ]] && dirs+=("$VAULT_DIR")

case "$(uname -s)" in
  Darwin)
    dirs+=(
      "$HOME/Library/Application Support/$BUNDLE_ID"
      "$HOME/Library/WebKit/$BUNDLE_ID"
      "$HOME/Library/Caches/$BUNDLE_ID"
      "$HOME/Library/Saved Application State/$BUNDLE_ID.savedState"
      "$HOME/Library/Preferences/$BUNDLE_ID.plist"
      "$HOME/Library/Logs/DiagnosticReports"
      "${TMPDIR:-/tmp}"
    )
    ;;
  Linux)
    dirs+=(
      "$HOME/.local/share/$BUNDLE_ID"
      "$HOME/.config/$BUNDLE_ID"
      "$HOME/.cache/$BUNDLE_ID"
      "$HOME/.local/share/znotes"
      "$HOME/.cache/znotes"
      "/tmp"
    )
    ;;
esac

found=0
scanned=0
for d in "${dirs[@]}"; do
  if [[ ! -e "$d" ]]; then
    echo "skip (absent): $d"
    continue
  fi
  scanned=$((scanned + 1))
  echo "scan: $d"
  while IFS= read -r -d '' f; do
    [[ -r "$f" ]] || continue
    if grep -aqF -- "$MARKER" "$f" 2>/dev/null; then
      echo "  LEAK: $f"
      found=1
    fi
  done < <(find "$d" -type f -user "$(id -un)" -size -200M -print0 2>/dev/null)
done

echo "scanned $scanned location(s) for marker '$MARKER'"
if [[ $found -ne 0 ]]; then
  echo "RESULT: plaintext marker FOUND"
  exit 1
fi
echo "RESULT: marker not found anywhere"
exit 0
