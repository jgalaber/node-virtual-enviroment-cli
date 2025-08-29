#!/usr/bin/env sh
set -e

chmod +x .githooks/pre-commit
chmod +x .githooks/commit-msg
chmod +x .githooks/pre-push

git config core.hooksPath .githooks
echo "✅ Git hooks enabled (core.hooksPath = .githooks)"
