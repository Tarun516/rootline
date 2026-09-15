//! Rootline analysis engine: repository scanning, language adapters,
//! resolution, and fact-graph construction.
//!
//! The crate exposes only its intentional API surface; everything else
//! stays module-private while the architecture is still settling. Core
//! domain types live in `rootline-core` and are re-exported from there,
//! never through this crate.

mod graph;
mod inventory;
mod python;

pub use graph::{DiagnosticKind, GraphDiagnostic, ResolvedGraph, build};
pub use inventory::{Inventory, ListingMode, ScanError, ScanIssue, ScanIssueKind, scan};
pub use python::{
    ADAPTER_NAME, ADAPTER_VERSION, ModuleIndex, PythonAdapter, PythonAdapterError, Resolution,
    supports_extension,
};
