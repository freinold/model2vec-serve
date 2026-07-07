---
layout: home

hero:
  name: model2vec-serve
  text: Lightweight Embeddings Server
  tagline: Serve static model2vec embedding models through OpenAI- and TEI-compatible endpoints.
  actions:
    - theme: brand
      text: Get Started
      link: /getting-started
    - theme: alt
      text: API Reference
      link: /api/openai
    - theme: alt
      text: View on GitHub
      link: https://github.com/freinold/model2vec-serve

features:
  - title: OpenAI Compatible
    details: Drop-in replacement for OpenAI's embeddings endpoint with POST /v1/embeddings.
  - title: TEI Compatible
    details: Reuse existing Hugging Face TEI clients with POST /embed and GET /info.
  - title: Optional API Key Auth
    details: Protect embedding endpoints with Authorization Bearer tokens.
  - title: Production Ready
    details: Structured JSON logs, Prometheus metrics, health/readiness probes, and a Helm chart.
---
