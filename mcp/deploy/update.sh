docker pull ghcr.io/stakwork/mesh-mcp:latest

docker stop mcp.sphinx &> /dev/null
docker rm mcp.sphinx &> /dev/null

docker-compose up -d

docker logs -f mcp.sphinx