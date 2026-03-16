#!/usr/bin/env bash
# Example: Sync credentials to a Docker sandbox and run Claude Code
#
# Prerequisites:
#   - sync-auth installed on host
#   - A private Git repo for credentials (e.g. github.com/USER/credentials)
#   - Docker installed
#   - link-foundation/sandbox image pulled
#
# Usage:
#   ./examples/docker/sync-and-code.sh <credentials-repo-url> <project-repo-url>
#
# Example:
#   ./examples/docker/sync-and-code.sh \
#     https://github.com/myuser/my-credentials.git \
#     https://github.com/myuser/my-project.git

set -euo pipefail

CREDENTIALS_REPO="${1:?Usage: $0 <credentials-repo-url> <project-repo-url>}"
PROJECT_REPO="${2:?Usage: $0 <credentials-repo-url> <project-repo-url>}"
CONTAINER_NAME="sandbox-$(date +%s)"
SANDBOX_IMAGE="${SANDBOX_IMAGE:-ghcr.io/link-foundation/sandbox:latest}"

echo "=== Step 1: Push host credentials to repo ==="
sync-auth --repo "$CREDENTIALS_REPO" push
echo ""

echo "=== Step 2: Start sandbox container ==="
docker run -d \
  --name "$CONTAINER_NAME" \
  "$SANDBOX_IMAGE" \
  sleep infinity
echo "Container: $CONTAINER_NAME"
echo ""

echo "=== Step 3: Install sync-auth in container ==="
docker exec "$CONTAINER_NAME" bash -c "cargo install sync-auth"
echo ""

echo "=== Step 4: Pull credentials into container ==="
docker exec "$CONTAINER_NAME" bash -c "sync-auth --repo '$CREDENTIALS_REPO' pull"
echo ""

echo "=== Step 5: Clone project in container ==="
docker exec "$CONTAINER_NAME" bash -c "cd /workspace && git clone '$PROJECT_REPO' project"
echo ""

echo "=== Step 6: Start credential sync daemon in container ==="
docker exec "$CONTAINER_NAME" bash -c "sync-auth --repo '$CREDENTIALS_REPO' daemon start --interval 120"
echo ""

echo "=== Ready! ==="
echo "Credentials synced. You can now run Claude Code in the container:"
echo "  docker exec -it $CONTAINER_NAME bash -c 'cd /workspace/project && claude'"
echo ""
echo "To stop the container:"
echo "  docker stop $CONTAINER_NAME && docker rm $CONTAINER_NAME"
