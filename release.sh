#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'EOF'
Usage:
  ./release.sh [vX.Y.Z|X.Y.Z]

Behavior:
  1) Creates an annotated git tag (vX.Y.Z)
  2) Pushes current branch to origin
  3) Pushes the tag to origin
  4) GitHub Actions "Build and Release" runs automatically on tag push
     and publishes build artifacts to GitHub Releases.

Notes:
  - If tag is omitted, version is read from Cargo.toml and prefixed with "v".
  - Requires: git
  - Optional: gh (to watch workflow run)
EOF
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
    usage
    exit 0
fi

if ! command -v git >/dev/null 2>&1; then
    echo "[ERROR] git not found."
    exit 1
fi

if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    echo "[ERROR] Not inside a git repository."
    exit 1
fi

if [[ -n "$(git status --porcelain)" ]]; then
    echo "[ERROR] Working tree is not clean. Commit/stash changes first."
    exit 1
fi

TAG_INPUT="${1:-}"
if [[ -z "$TAG_INPUT" ]]; then
    VERSION="$(sed -n 's/^version[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p' Cargo.toml | head -n1)"
    if [[ -z "$VERSION" ]]; then
        echo "[ERROR] Could not read version from Cargo.toml."
        exit 1
    fi
    TAG="v$VERSION"
else
    if [[ "$TAG_INPUT" == v* ]]; then
        TAG="$TAG_INPUT"
    else
        TAG="v$TAG_INPUT"
    fi
fi

if git rev-parse -q --verify "refs/tags/$TAG" >/dev/null; then
    echo "[ERROR] Tag already exists locally: $TAG"
    exit 1
fi

if git ls-remote --exit-code --tags origin "refs/tags/$TAG" >/dev/null 2>&1; then
    echo "[ERROR] Tag already exists on origin: $TAG"
    exit 1
fi

BRANCH="$(git rev-parse --abbrev-ref HEAD)"
if [[ "$BRANCH" == "HEAD" ]]; then
    echo "[ERROR] Detached HEAD. Checkout a branch first."
    exit 1
fi

echo "[INFO] Branch: $BRANCH"
echo "[INFO] Tag:    $TAG"

git tag -a "$TAG" -m "Release $TAG"
echo "[INFO] Created tag $TAG"

echo "[INFO] Pushing branch to origin..."
git push origin "$BRANCH"

echo "[INFO] Pushing tag to origin..."
git push origin "$TAG"

REMOTE_URL="$(git remote get-url origin)"
REPO_PATH=""
if [[ "$REMOTE_URL" =~ ^git@github.com:(.+)\.git$ ]]; then
    REPO_PATH="${BASH_REMATCH[1]}"
elif [[ "$REMOTE_URL" =~ ^https?://github.com/(.+)\.git$ ]]; then
    REPO_PATH="${BASH_REMATCH[1]}"
elif [[ "$REMOTE_URL" =~ ^https?://github.com/(.+)$ ]]; then
    REPO_PATH="${BASH_REMATCH[1]}"
fi

if [[ -n "$REPO_PATH" ]]; then
    echo "[INFO] Release page:"
    echo "       https://github.com/$REPO_PATH/releases/tag/$TAG"
fi

if command -v gh >/dev/null 2>&1 && gh auth status >/dev/null 2>&1; then
    echo "[INFO] Waiting for GitHub Actions run..."
    RUN_ID=""
    for _ in {1..30}; do
        RUN_ID="$(gh run list \
            --workflow "Build and Release" \
            --limit 30 \
            --json databaseId,headBranch,event \
            --jq ".[] | select(.headBranch == \"$TAG\" and .event == \"push\") | .databaseId" \
            | head -n1)"
        if [[ -n "$RUN_ID" ]]; then
            break
        fi
        sleep 3
    done

    if [[ -n "$RUN_ID" ]]; then
        echo "[INFO] Watching run id: $RUN_ID"
        gh run watch "$RUN_ID" --exit-status
    else
        echo "[WARN] Could not find workflow run automatically. Check Actions tab manually."
    fi
else
    echo "[INFO] 'gh' not found or not authenticated. Check Actions tab for workflow progress."
fi

echo "[DONE] Tag pushed. GitHub workflow will publish release artifacts."
