# MinIO Setup

## Prerequisites
* Container engine & management tool installation (e.g. [Rancher Desktop](https://rancherdesktop.io/), [Podman Desktop](https://podman-desktop.io/downloads))

## Change directory to your workspace
```bash
cd <your workspace>
```

## Create docker-compose.yaml
```yaml
version: "3"
services:
  minio:
    image: minio/minio:latest
    ports:
      - ${MINIO_PORT:-9000}:9000
      - ${MINIO_CONSOLE_PORT:-9001}:9001
    volumes:
      - ./minio/data:/export
    environment:
      MINIO_ROOT_USER: <MINIO admin user>
      MINIO_ROOT_PASSWORD: <MINIO admin password>
    command: server /export --console-address ":9001"
```

## Create shared volume
```bash
mkdir -p ./minio/data
```

## Specify Username and Password
Specify your minio secret in your docker-compose.yaml.
```bash
MINIO_ROOT_USER: your_username
MINIO_ROOT_PASSWORD: your_secret
```

## Run MinIO

### Docker
```shell-session
docker compose up -d
```

### Podman
```shell-session
podman-compose up -d
```

## Create a bucket for testing
- Access http://127.0.0.1:9001/
- Goto Administrator > Buckets page and click "Create Bucket" on the top right.
- Specify bucket name and click "Create Bucket".

## Stop MinIO

### Docker
```shell-session
docker compose down
```

### Podman
```shell-session
podman-compose down
```
