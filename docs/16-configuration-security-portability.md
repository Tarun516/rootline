# Configuration, local security, and portability

## Explicit configuration flow

Application boundaries read CLI arguments, environment variables, and configuration files. They validate and merge those inputs into typed application and engine configuration, then pass configuration explicitly to libraries.

```text
CLI / local server / future desktop shell
  → validate input sources
  → AppConfig
  → EngineConfig and component-specific config
```

Scanner, parser, resolver, and store libraries should not read environment variables throughout their internals. Explicit configuration improves tests, embedding, reproducibility, and comparison between CLI and client behavior. Analysis outputs record the configuration digest relevant to their facts.

## Local transport security

If Rootline serves HTTP locally, loopback binding alone is not a security boundary. The server should use:

- loopback-only binding;
- a random per-session authorization token or an equivalently strong local authentication mechanism;
- strict Origin checks and narrow CORS policy;
- no wildcard CORS;
- repository-derived path allowlists for source access;
- safe public error responses;
- explicit protocol-version handshake.

The exact mechanism needs a threat-model spike before server implementation. IPC may change the transport details but does not remove the need for access control and bounded file access.

## Source privacy

Repository content stays local by default. External semantic providers require explicit configuration and visible data boundaries. The product should identify what projection or source context an enrichment task would transmit. Logs, diagnostics, analytics, and protocol errors do not include source content by default.

Avoid persisting unrelated absolute filesystem paths, credentials, or secrets. Treat repository files, generated text, and model output as untrusted data rather than instructions.

## Repository boundary

Scanning and source-serving must resolve exact targets within the selected repository. Symlink, junction, path traversal, and stale-file behavior require explicit policies. A valid graph path does not grant arbitrary access to other local files. Source access should be checked against the current indexed artifact allowlist and revision/freshness state.

## Cross-platform filesystem correctness

Linux, macOS, and Windows differ in separators, drives, case sensitivity, Unicode behavior, permissions, symlinks, junctions, and path encodings. These differences affect artifact identity, hashing, import resolution, and persisted graph keys.

Use platform-native path types at filesystem boundaries. Lossy path-to-string conversion is for display only, never identity or hashing. Repository-relative path semantics should be centralized and tested. Add cross-platform CI as soon as real scanner and resolver code exists; do not defer Windows behavior until release.

## Test matrix

At minimum, fixtures should cover:

- ignored and re-included files;
- symlinks/junctions inside and outside the repository;
- mixed separators and drive paths where relevant;
- case-colliding names;
- non-UTF-8 or otherwise unusual names where supported;
- permission denial and vanished files;
- stale index versus changed source;
- malicious traversal in protocol requests;
- unauthorized local requests and disallowed Origins.

Security and portability decisions should be documented through ADRs when their implementation boundary becomes concrete.
