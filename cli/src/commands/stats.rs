//! `stats` (dev) command: a coarse summary report over an analyzed APK.
//!
//! Thin presentation layer only -- all counting logic lives in the
//! `analysis` crate ([`analysis::compute_stats`], [`analysis::Cfg`]);
//! timing and memory reporting are CLI-only concerns.

use std::fmt::Write as _;
use std::time::Duration;

use analysis::{AnalysisContext, ApkStats, Cfg};
use ir::ApkIR;

/// Aggregate counts for a `stats` report.
#[derive(Debug, Clone, Default)]
pub struct StatsReport {
    pub counts: ApkStats,
    pub cfg_blocks: usize,
    pub cfg_edges: usize,
}

/// Build the XREF database and per-method CFGs, and collect summary counts.
pub fn collect(ir: &ApkIR) -> StatsReport {
    let db = AnalysisContext::new(ir).build();
    let counts = analysis::compute_stats(ir, &db);

    let mut cfg_blocks = 0usize;
    let mut cfg_edges = 0usize;
    for dex in &ir.dex_files {
        for class in &dex.classes {
            for method in &class.methods {
                let cfg = Cfg::build(method);
                cfg_blocks += cfg.block_count();
                cfg_edges += cfg.edge_count();
            }
        }
    }

    StatsReport {
        counts,
        cfg_blocks,
        cfg_edges,
    }
}

/// Render a human-readable report, including CLI-collected timing and peak
/// memory figures.
pub fn render(report: &StatsReport, elapsed: Duration, peak_rss_kb: Option<u64>) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "Classes: {}", report.counts.classes);
    let _ = writeln!(out, "Methods: {}", report.counts.methods);
    let _ = writeln!(out, "Instructions: {}", report.counts.instructions);
    let _ = writeln!(out, "Strings: {}", report.counts.strings);
    let _ = writeln!(out, "Fields: {}", report.counts.fields);
    let _ = writeln!(out, "XREF edges: {}", report.counts.xref_edges);
    let _ = writeln!(out, "CFG blocks: {}", report.cfg_blocks);
    let _ = writeln!(out, "CFG edges: {}", report.cfg_edges);
    let _ = writeln!(out, "Analysis time: {:.3}s", elapsed.as_secs_f64());
    match peak_rss_kb {
        Some(kb) => {
            let _ = writeln!(out, "Peak RAM: {} MB", kb / 1024);
        }
        None => {
            let _ = writeln!(out, "Peak RAM: n/a");
        }
    }
    out
}

/// Read peak resident set size (`VmHWM`) from `/proc/self/status`, in
/// kilobytes.
///
/// Returns `None` on non-Linux platforms, or if `/proc/self/status` is
/// unavailable/unparseable (e.g. a sandboxed environment). Never panics.
pub fn peak_rss_kb() -> Option<u64> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    for line in status.lines() {
        if let Some(rest) = line.strip_prefix("VmHWM:") {
            let digits: String = rest.chars().filter(char::is_ascii_digit).collect();
            return digits.parse().ok();
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_includes_all_fields() {
        let report = StatsReport {
            counts: ApkStats {
                classes: 1,
                methods: 2,
                instructions: 3,
                strings: 4,
                fields: 5,
                xref_edges: 6,
            },
            cfg_blocks: 7,
            cfg_edges: 8,
        };
        let text = render(&report, Duration::from_millis(1500), Some(2048));

        assert!(text.contains("Classes: 1"));
        assert!(text.contains("Methods: 2"));
        assert!(text.contains("Instructions: 3"));
        assert!(text.contains("Strings: 4"));
        assert!(text.contains("Fields: 5"));
        assert!(text.contains("XREF edges: 6"));
        assert!(text.contains("CFG blocks: 7"));
        assert!(text.contains("CFG edges: 8"));
        assert!(text.contains("Analysis time: 1.500s"));
        assert!(text.contains("Peak RAM: 2 MB"));
    }

    #[test]
    fn render_handles_missing_peak_rss() {
        let text = render(&StatsReport::default(), Duration::ZERO, None);
        assert!(text.contains("Peak RAM: n/a"));
    }
}
