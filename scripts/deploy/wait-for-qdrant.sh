#!/bin/sh

set -e

echo "Waiting for Qdrant to be ready..."
until curl -s http://qdrant:6333/healthz | grep -q "passed"; do
  sleep 1
done

echo "Qdrant is ready. Starting web service..."
exec /app/bin/web "$@"
