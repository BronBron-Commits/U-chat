#!/usr/bin/env bash

ROOT="$(pwd)"
VCS_DIR="$ROOT/.unhidra_vcs"
OBJ="$VCS_DIR/objects"
META="$VCS_DIR/meta"

mkdir -p "$OBJ" "$META"

# Timestamp snapshot ID
gen_id() {
    date +"%Y%m%d%H%M%S"
}

# Save snapshot
cmd_save() {
    ID=$(gen_id)
    mkdir -p "$OBJ/$ID"

    echo "Saving snapshot..."

    rsync -a --delete \
        --exclude "target" \
        --exclude ".unhidra_vcs" \
        "$ROOT/" "$OBJ/$ID/" >/dev/null 2>&1

    echo "$ID" >> "$META/history"
    echo "Snapshot saved: $ID"
}

# List snapshots
cmd_list() {
    cat "$META/history"
}

# Diff snapshots
cmd_diff() {
    OLD="$1"
    NEW="$2"

    if [ ! -d "$OBJ/$OLD" ]; then
        echo "Missing: $OLD"
        exit 1
    fi
    if [ ! -d "$OBJ/$NEW" ]; then
        echo "Missing: $NEW"
        exit 1
    fi

    diff -ruN "$OBJ/$OLD" "$OBJ/$NEW"
}

# Checkout snapshot
cmd_checkout() {
    ID="$1"

    if [ ! -d "$OBJ/$ID" ]; then
        echo "Snapshot $ID missing"
        exit 1
    fi

    echo "Restoring snapshot $ID..."

    rsync -a --delete \
        "$OBJ/$ID/" "$ROOT/" >/dev/null 2>&1

    echo "NOW_ACTIVE=$ID" > "$META/active"
    echo "Active snapshot: $ID"
}

case "$1" in
    save) cmd_save ;;
    list) cmd_list ;;
    diff) cmd_diff "$2" "$3" ;;
    checkout) cmd_checkout "$2" ;;
    *)
        echo "Usage:"
        echo "./unhidra-vcs.sh save"
        echo "./unhidra-vcs.sh list"
        echo "./unhidra-vcs.sh diff OLD_ID NEW_ID"
        echo "./unhidra-vcs.sh checkout ID"
        ;;
esac
