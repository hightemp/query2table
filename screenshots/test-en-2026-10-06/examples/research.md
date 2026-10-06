# Compare Qdrant and Milvus for a small self-hosted semantic search service. Use English sources, explain deployment and licensing, and cite official documentation.

## Qdrant vs. Milvus for a small self-hosted semantic search service

Both are open-source vector databases with permissive licensing, but they differ in deployment complexity and operational footprint. For a *small* self-hosted service, Qdrant is generally the simpler choice; Milvus is more capable at scale but heavier to run.

### Licensing

| | Qdrant | Milvus |
|---|---|---|
| License | Apache License 2.0 | Apache License 2.0 |
| Commercial use | Permitted | Permitted |
| Source | [Qdrant Docker Hub](https://hub.docker.com/r/qdrant/qdrant) | [Milvus Product FAQ](https://milvus.io/docs/product_faq.md) |

Both projects are distributed under the **Apache License 2.0**, so you can self-host, modify, and use them commercially without paying license fees. Milvus's FAQ explicitly states: *"Please adhere to Apache License 2.0 when using Milvus for production or distribution purposes."* Qdrant's Docker image page likewise lists Apache 2.0.

### Deployment

**Qdrant** — The official [Installation guide](https://qdrant.tech/documentation/installation/) describes self-hosting via **Docker, Kubernetes, or binary releases**, and notes that for development and testing Docker is recommended. A single `qdrant/qdrant` container is enough to get started; the [Local Quickstart](https://qdrant.tech/documentation/quickstart/) walks through running it locally with Docker, connecting an SDK, and creating a first collection. Because Qdrant is a single Rust binary, a small deployment needs only one container (plus optional persistent volume), making it very lightweight for a small semantic search service.

**Milvus** — The official docs offer two main self-hosted paths:
- [Run Milvus in Docker (Linux)](https://milvus.io/docs/install_standalone-docker.md) — uses an installation script (`standalone_embed.sh`) for a quick embedded standalone setup.
- [Run Milvus with Docker Compose (Linux)](https://milvus.io/docs/install_standalone-docker-compose.md) — the default deployment starts **three containers**: `milvus-etcd` (metadata), `milvus-minio` (object storage), and `milvus-standalone`. In v3.0.2 the message queue is Woodpecker (embedded, using MinIO/object storage as its WAL backend), so no separate message-queue container is required.

So even the "standalone" Milvus deployment is a multi-component stack (etcd + MinIO + Milvus), which is more moving parts to operate, back up, and monitor than a single Qdrant container.

### Recommendation for a small self-hosted service

- **Choose Qdrant** if you want minimal operational overhead: one container, simple Docker/Kubernetes deployment, Apache 2.0, and a quickstart that gets you to semantic search fast.
- **Choose Milvus** if you expect to grow into large-scale, distributed workloads and are comfortable running its etcd + MinIO + Milvus stack (or using the embedded standalone script for a lighter trial).

For a *small* service, Qdrant's single-binary deployment is the lower-friction option, while both remain equally permissive from a licensing standpoint.

### Sources
- Qdrant Installation: https://qdrant.tech/documentation/installation/
- Qdrant Local Quickstart: https://qdrant.tech/documentation/quickstart/
- Qdrant Docker image (Apache 2.0): https://hub.docker.com/r/qdrant/qdrant
- Milvus Docker install: https://milvus.io/docs/install_standalone-docker.md
- Milvus Docker Compose install: https://milvus.io/docs/install_standalone-docker-compose.md
- Milvus Product FAQ (Apache 2.0): https://milvus.io/docs/product_faq.md

**Pages read**

1. [https://qdrant.tech/documentation/installation/](https://qdrant.tech/documentation/guides/installation/)
2. [Installation - Qdrant](https://qdrant.tech/documentation/installation/)
