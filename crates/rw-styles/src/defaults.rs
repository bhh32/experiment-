/// Provides functions for creating common default style sets.
///
/// This module contains factory functions that produce pre-configured
/// style catalogs for different use cases (e.g., a minimal set, a
/// Word-compatible set, an ODF-compatible set).

use crate::catalog::StyleCatalog;

/// Create a minimal style catalog with just the essential styles.
pub fn minimal_catalog() -> StyleCatalog {
    StyleCatalog::with_defaults()
}
