#!/bin/bashsetp
docker buildx build --platform linux/amd64,linux/arm64 \
  -f Dockerfile.multiarch -t agentmem:multiarch .