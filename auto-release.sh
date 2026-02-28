#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'EOF'
Usage:
  ./auto-release.sh [vX.Y.Z|X.Y.Z]

Behavior:
  1) Asks for release version (if not provided as argument)
  2) Stages all changes and creates a release commit (if there are changes)
  3) Pushes current branch to origin
  4) Creates annotated tag (vX.Y.Z)
  5) Pushes the tag to origin
  6) Tag push triggers GitHub Actions "Build and Release" workflow

Notes:
  - Only version is requested from you.
  - Commit message is auto-generated as: chore(release): vX.Y.Z
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

VERSION_INPUT="${1:-}"
if [[ -z "$VERSION_INPUT" ]]; then
    read -r -p "Release version (X.Y.Z or vX.Y.Z): " VERSION_INPUT
fi

VERSION_INPUT="${VERSION_INPUT//[[:space:]]/}"
if [[ -z "$VERSION_INPUT" ]]; then
    echo "[ERROR] Version cannot be empty."
    exit 1
fi

if [[ "$VERSION_INPUT" == v* ]]; then
    TAG="$VERSION_INPUT"
else
    TAG="v$VERSION_INPUT"
fi

if [[ ! "$TAG" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "[ERROR] Invalid version format: '$VERSION_INPUT'"
    echo "        Expected: X.Y.Z or vX.Y.Z"
    exit 1
fi

BRANCH="$(git rev-parse --abbrev-ref HEAD)"
if [[ "$BRANCH" == "HEAD" ]]; then
    echo "[ERROR] Detached HEAD. Checkout a branch first."
    exit 1
fi

if git rev-parse -q --verify "refs/tags/$TAG" >/dev/null; then
    echo "[ERROR] Tag already exists locally: $TAG"
    exit 1
fi

if git ls-remote --exit-code --tags origin "refs/tags/$TAG" >/dev/null 2>&1; then
    echo "[ERROR] Tag already exists on origin: $TAG"
    exit 1
fi

echo "[INFO] Branch: $BRANCH"
echo "[INFO] Tag:    $TAG"

git add -A
if git diff --cached --quiet; then
    echo "[INFO] No local changes to commit. Tag will point to current HEAD."
else
    COMMIT_MSG="chore(release): $TAG"
    git commit -m "$COMMIT_MSG"
    echo "[INFO] Created release commit: $COMMIT_MSG"
fi

echo "[INFO] Pushing branch..."
git push origin "$BRANCH"

git tag -a "$TAG" -m "Release $TAG"
echo "[INFO] Created tag: $TAG"

echo "[INFO] Pushing tag..."
git push origin "$TAG"

REMOTE_URL="$(git remote get-url origin 2>/dev/null || true)"
REPO_PATH=""
if [[ "$REMOTE_URL" =~ ^git@github.com:(.+)\.git$ ]]; then
    REPO_PATH="${BASH_REMATCH[1]}"
elif [[ "$REMOTE_URL" =~ ^https?://github.com/(.+)\.git$ ]]; then
    REPO_PATH="${BASH_REMATCH[1]}"
elif [[ "$REMOTE_URL" =~ ^https?://github.com/(.+)$ ]]; then
    REPO_PATH="${BASH_REMATCH[1]}"
fi

if [[ -n "$REPO_PATH" ]]; then
    echo "[INFO] Release URL: https://github.com/$REPO_PATH/releases/tag/$TAG"
fi

echo "[DONE] Branch and tag pushed. 'Build and Release' should publish this version."
