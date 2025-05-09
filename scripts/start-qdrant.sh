docker run -p 6334:6334 \
  -v $(pwd)/data/qdrant:/qdrant/storage \
  -e QDRANT__SERVICE__GRPC_PORT="6334" \
  -d qdrant/qdrant:latest
