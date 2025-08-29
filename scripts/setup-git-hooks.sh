#!/usr/bin/env sh
set -e

chmod +x .githooks/pre-commit
chmod +x .githooks/pre-push

git config core.hooksPath .githooks
echo "✅ Git hooks enabled (core.hooksPath = .githooks)"
