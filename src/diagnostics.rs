//! Content-free settings-window diagnostics for host-owned profiling tools.

use std::ops::Range;

/// Bounded profiling snapshot for one settings window.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SettingsWindowDiagnostics {
    /// Whether the reusable settings window wrapper currently considers the OS
    /// window visible.
    pub visible: bool,
    /// Stable selected section id supplied by the host model.
    pub selected_section_id: String,
    /// Stable selected page id supplied by the host model.
    pub selected_page_id: String,
    /// Diagnostics for the selected page's detail row surface.
    pub detail_rows: SettingsWindowRowSurfaceDiagnostics,
    /// Diagnostics for the selected page's local split list, when present.
    pub split_list: Option<SettingsWindowRowSurfaceDiagnostics>,
    /// Content-free bounded pager counters for the selected split source.
    pub split_pager: Option<SettingsWindowSplitPagerDiagnostics>,
    /// Counters and timings for recent settings-window work.
    pub performance: SettingsWindowPerformanceDiagnostics,
}

/// Content-free bounded paged split-source counters.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SettingsWindowSplitPagerDiagnostics {
    pub resident_page_count: usize,
    pub resident_item_count: usize,
    pub pending_request_count: usize,
    pub stale_result_count: u64,
}

/// Content-free diagnostics for one row-like settings surface.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SettingsWindowRowSurfaceDiagnostics {
    /// App-neutral surface label owned by the settings-window crate.
    pub surface_id: String,
    /// Total rows/items represented by the surface.
    pub total_row_count: usize,
    /// Rows/items included in the current render tree.
    pub rendered_row_count: usize,
    /// Current visible/rendered model-index range when the surface is windowed.
    pub visible_range: Option<SettingsWindowRangeDiagnostics>,
    /// Overscan rows/items added outside the visible range.
    pub overscan_count: usize,
    /// Human-readable rendering strategy, such as `full_selected_page` or
    /// `fixed_height_windowed`.
    pub row_height_strategy: String,
}

/// Half-open model-index range.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SettingsWindowRangeDiagnostics {
    /// Inclusive start index.
    pub start: usize,
    /// Exclusive end index.
    pub end: usize,
}

/// Settings-window render and synchronization counters.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SettingsWindowPerformanceDiagnostics {
    /// Number of panel render-tree construction passes observed.
    pub render_count: u64,
    /// Last panel render-tree construction time in microseconds.
    pub last_render_tree_micros: u64,
    /// Number of model synchronization calls observed.
    pub model_sync_count: u64,
    /// Last model synchronization time in microseconds.
    pub last_model_sync_micros: u64,
    /// Number of option synchronization calls observed.
    pub option_sync_count: u64,
    /// Last option synchronization time in microseconds.
    pub last_option_sync_micros: u64,
    /// Total text-input or color-picker input entities synchronized.
    pub input_sync_count: u64,
    /// Input entities synchronized by the most recent input-sync operation.
    pub last_input_sync_entity_count: usize,
    /// Total color preview lookups observed.
    pub color_preview_lookup_count: u64,
    /// Color preview lookups during the last render-tree construction pass.
    pub last_render_color_preview_lookup_count: u64,
    /// Total color model lookup calls observed while resolving previews.
    pub color_model_lookup_count: u64,
    /// Color model lookup calls during the last render-tree construction pass.
    pub last_render_color_model_lookup_count: u64,
    /// Largest recent measured timing bucket.
    pub dominant_cost_category: String,
}

impl SettingsWindowRangeDiagnostics {
    pub(crate) fn from_range(range: Range<usize>) -> Self {
        Self {
            start: range.start,
            end: range.end,
        }
    }
}
