#!/usr/bin/env bash
set -euo pipefail

if [[ ! -f .env.production ]]; then
  echo "Missing .env.production. Copy .env.production.example and fill it before deploying." >&2
  exit 1
fi

if [[ ! -f secrets/github-app.pem ]]; then
  echo "Missing secrets/github-app.pem. Add the GitHub App private key before deploying." >&2
  exit 1
fi

if docker compose version >/dev/null 2>&1; then
  compose=(docker compose)
elif command -v docker-compose >/dev/null 2>&1; then
  compose=(docker-compose)
else
  echo "Missing Docker Compose. Install docker-compose-plugin or docker-compose." >&2
  exit 1
fi

"${compose[@]}" --env-file .env.production -f docker-compose.prod.yml pull caddy
"${compose[@]}" --env-file .env.production -f docker-compose.prod.yml up -d --build
"${compose[@]}" --env-file .env.production -f docker-compose.prod.yml ps
