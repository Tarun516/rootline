# ADR 0002: Product client and versioned protocol

- Status: Accepted
- Date: 2026-09-15
- Owners: Rootline project
- Supersedes: None
- Superseded by: None

## Context

Rootline's Rust engine owns indexing, persistence, and local queries. The product UI needs fast client-side graph interaction, not a second full-stack application server. Internal graph structs will evolve independently from what the UI needs.

## Decision drivers

- Keep the Rust runtime as the application backend.
- Provide responsive client-side interaction and static output.
- Keep visualization-library types out of domain contracts.
- Allow engine and client versions to fail safely when incompatible.
- Send bounded projections rather than the complete graph.

## Considered options

A full-stack React framework could add server rendering and another runtime boundary, but those do not serve the core local product. A framework-neutral UI decision would postpone useful constraints. Direct JSON serialization of internal Rust structs would couple engine implementation to the client.

## Decision

Build the product UI with React, TypeScript, and Vite in `apps/web`. A future public site is a separate application. The local engine/client boundary is a versioned protocol with a startup handshake, bounded projection DTOs, and stable public errors. Map DTOs into Rootline web-domain models, then into the selected canvas library. Add server transport only when required by the client; keep async runtime concerns at that boundary.

## Consequences

Protocol projections require deliberate design and compatibility tests. UI and engine internals can evolve separately. The application can remain a static client served by or connected to the Rust runtime. A future desktop wrapper does not define the protocol.

## Validation

The first UI slice can navigate repository → module/file → symbol through versioned DTOs, reject incompatible protocol versions clearly, and avoid transferring the stored graph wholesale.

## Revisit triggers

A demonstrated requirement for server rendering in the product client, transport constraints, or measured limitations in Vite/static-client deployment.

## References

- [Repository organization](../14-repository-organization.md)
- [Semantic zoom and UX](../08-semantic-zoom-and-ux.md)
