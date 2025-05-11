#!/bin/bash

if ! command -v uv; then
  echo 'make sure uv (https://docs.astral.sh/uv/getting-started/installation/) is installed before running'
  exit 1;
fi

for lang in en pt-br es; do
  WORDS="data/words/$lang-words.txt"
  EMBEDS="data/embeds/$lang-embeds.txt"
  N=500

  if [ "$lang" = "en" ]; then
    MODEL="BAAI/bge-small-en-v1.5"
  else
    MODEL="sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2"
  fi

  # build search space from Contexto API
  if [ ! -f "$WORDS" ]; then
    uv run konteksto-builder/scrape.py --lang "$lang" --out-file "$WORDS" --n-past-games "$N"
  fi

  # generate embeddings
  if [ ! -f "$EMBEDS" ]; then
    uv run konteksto-builder/embed.py --in-file "$WORDS" --out-file "$EMBEDS" --model "$MODEL"
  fi
done
