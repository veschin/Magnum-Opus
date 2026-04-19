#!/usr/bin/env bash
# Prune stale Cargo debug artefacts while keeping the build cache warm.
#
# Triggers on Bash tool use; acts only when tool_input.command contained
# `cargo build|test|run|bench`.
#
# Two passes over executables in target/debug/deps/ >10 MiB and untouched
# for >1 minute:
#   1. STALE     - base name does not match any tests/examples/benches
#                  source file nor the lib crate -> delete.
#   2. DUPLICATE - among live-name binaries with different hashes, keep
#                  only the newest; delete older copies.
#
# .rlib / .rmeta / .d / .o / .dwp are untouched so dependency compilation
# cache survives (no re-download, no recompile).
set -euo pipefail

INPUT=$(cat)
CMD=$(echo "$INPUT" | jq -r '.tool_input.command // empty')
case "$CMD" in
    *"cargo build"*|*"cargo test"*|*"cargo run"*|*"cargo bench"*) ;;
    *) exit 0 ;;
esac

PROJECT="/home/veschin/ai/magnum-opus/magnum_opus"
DEPS="$PROJECT/target/debug/deps"
[ -d "$DEPS" ] || exit 0

LIVE_FILE=$(mktemp)
trap 'rm -f "$LIVE_FILE"' EXIT

{
    [ -d "$PROJECT/tests" ]    && find "$PROJECT/tests"    -maxdepth 1 -name '*.rs' -printf '%f\n' | sed 's/\.rs$//'
    [ -d "$PROJECT/examples" ] && find "$PROJECT/examples" -maxdepth 1 -name '*.rs' -printf '%f\n' | sed 's/\.rs$//'
    [ -d "$PROJECT/benches" ]  && find "$PROJECT/benches"  -maxdepth 1 -name '*.rs' -printf '%f\n' | sed 's/\.rs$//'
    echo "magnum_opus"
} | sort -u > "$LIVE_FILE"

REMOVED=0
FREED=0
declare -A LATEST_MT
declare -A LATEST_F

delete_file() {
    local f=$1
    local sz
    sz=$(stat -c%s "$f" 2>/dev/null || echo 0)
    rm -f "$f" "$f.d" 2>/dev/null || true
    REMOVED=$((REMOVED + 1))
    FREED=$((FREED + sz))
}

while IFS= read -r f; do
    BASE=$(basename "$f")
    NAME=$(echo "$BASE" | sed -E 's/-[0-9a-f]{16}$//')
    NAME_ALT=${NAME//-/_}

    IN_LIVE=false
    if grep -qxF "$NAME" "$LIVE_FILE" || grep -qxF "$NAME_ALT" "$LIVE_FILE"; then
        IN_LIVE=true
    fi

    if ! $IN_LIVE; then
        delete_file "$f"
        continue
    fi

    MT=$(stat -c%Y "$f" 2>/dev/null || echo 0)
    KEY="$NAME_ALT"
    PREV_MT=${LATEST_MT[$KEY]:-}
    PREV_F=${LATEST_F[$KEY]:-}

    if [ -z "$PREV_MT" ]; then
        LATEST_MT[$KEY]=$MT
        LATEST_F[$KEY]=$f
    elif [ "$MT" -gt "$PREV_MT" ]; then
        delete_file "$PREV_F"
        LATEST_MT[$KEY]=$MT
        LATEST_F[$KEY]=$f
    else
        delete_file "$f"
    fi
done < <(find "$DEPS" -maxdepth 1 -type f \
    ! -name '*.rlib' ! -name '*.rmeta' ! -name '*.d' \
    ! -name '*.o' ! -name '*.dwp' \
    -size +10M -mmin +1 -print)

if [ "$REMOVED" -gt 0 ]; then
    MB=$((FREED / 1048576))
    echo "cargo-sweep: removed $REMOVED artefact(s), freed ${MB} MiB" >&2
fi

exit 0
