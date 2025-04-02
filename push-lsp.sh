docker buildx build --platform linux/amd64 -t mesh-lsp -f lsp/Dockerfile .

docker tag mesh-lsp sphinxlightning/mesh-lsp:latest

docker push sphinxlightning/mesh-lsp:latest
