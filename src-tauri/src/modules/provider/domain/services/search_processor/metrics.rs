use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Metrics for the search results processing pipeline
///
/// Tracks performance and throughput of each stage for observability.
#[derive(Debug, Clone)]
pub struct PipelineMetrics {
    /// Total duration of the entire pipeline
    pub total_duration: Duration,

    /// Duration of each stage by name
    pub stage_durations: HashMap<String, Duration>,

    /// Number of results input to the pipeline
    pub input_count: usize,

    /// Number of results output from the pipeline
    pub output_count: usize,

    /// Number of duplicate groups found
    pub duplicate_groups: usize,

    /// Number of anime merged from multiple providers
    pub merge_count: usize,

    /// Number of results filtered out by quality threshold
    pub filtered_count: usize,

    /// Number of results truncated by limit
    pub truncated_count: usize,
}

impl PipelineMetrics {
    /// Create empty metrics
    pub fn new() -> Self {
        Self {
            total_duration: Duration::ZERO,
            stage_durations: HashMap::new(),
            input_count: 0,
            output_count: 0,
            duplicate_groups: 0,
            merge_count: 0,
            filtered_count: 0,
            truncated_count: 0,
        }
    }

    /// Calculate deduplication rate (percentage of inputs deduplicated)
    pub fn deduplication_rate(&self) -> f32 {
        if self.input_count == 0 {
            return 0.0;
        }

        let deduplicated = self.input_count - self.duplicate_groups;
        (deduplicated as f32 / self.input_count as f32) * 100.0
    }

    /// Calculate filter rate (percentage of inputs filtered)
    pub fn filter_rate(&self) -> f32 {
        if self.input_count == 0 {
            return 0.0;
        }

        (self.filtered_count as f32 / self.input_count as f32) * 100.0
    }

    /// Calculate throughput (results per second)
    pub fn throughput(&self) -> f32 {
        if self.total_duration.is_zero() {
            return 0.0;
        }

        self.output_count as f32 / self.total_duration.as_secs_f32()
    }

    /// Generate a human-readable report
    pub fn report(&self) -> String {
        let mut lines = vec![
            "=== Pipeline Metrics ===".to_string(),
            format!("Total Duration: {:.2}ms", self.total_duration.as_millis()),
            format!("Input Count: {}", self.input_count),
            format!("Output Count: {}", self.output_count),
            format!("Deduplication Rate: {:.1}%", self.deduplication_rate()),
            format!("Merge Count: {}", self.merge_count),
            format!(
                "Filtered Count: {} ({:.1}%)",
                self.filtered_count,
                self.filter_rate()
            ),
            format!("Truncated Count: {}", self.truncated_count),
            format!("Throughput: {:.1} results/sec", self.throughput()),
            "".to_string(),
            "Stage Durations:".to_string(),
        ];

        // Sort stages by duration (slowest first)
        let mut stages: Vec<_> = self.stage_durations.iter().collect();
        stages.sort_by(|a, b| b.1.cmp(a.1));

        for (stage, duration) in stages {
            let percentage = if !self.total_duration.is_zero() {
                (duration.as_secs_f64() / self.total_duration.as_secs_f64()) * 100.0
            } else {
                0.0
            };
            lines.push(format!(
                "  {}: {:.2}ms ({:.1}%)",
                stage,
                duration.as_millis(),
                percentage
            ));
        }

        lines.join("\n")
    }
}

impl Default for PipelineMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper for timing pipeline stages
pub struct StageTimer {
    stage_name: String,
    start: Instant,
}

impl StageTimer {
    /// Start timing a stage
    pub fn start(stage_name: impl Into<String>) -> Self {
        Self {
            stage_name: stage_name.into(),
            start: Instant::now(),
        }
    }

    /// Stop timing and record duration in metrics
    pub fn stop(self, metrics: &mut PipelineMetrics) -> Duration {
        let duration = self.start.elapsed();
        metrics.stage_durations.insert(self.stage_name, duration);
        duration
    }

    /// Stop timing and record duration in metrics builder
    pub fn stop_builder(self, builder: &mut MetricsBuilder) -> Duration {
        let duration = self.start.elapsed();
        builder.add_stage(self.stage_name, duration);
        duration
    }

    /// Get the stage name
    pub fn stage_name(&self) -> &str {
        &self.stage_name
    }

    /// Get elapsed time so far (without stopping the timer)
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

/// Builder for constructing metrics incrementally
pub struct MetricsBuilder {
    metrics: PipelineMetrics,
    pipeline_start: Option<Instant>,
}

impl MetricsBuilder {
    pub fn new() -> Self {
        Self {
            metrics: PipelineMetrics::new(),
            pipeline_start: None,
        }
    }

    /// Start timing the entire pipeline
    pub fn start_pipeline(&mut self) {
        self.pipeline_start = Some(Instant::now());
    }

    /// Stop timing the entire pipeline
    pub fn stop_pipeline(&mut self) {
        if let Some(start) = self.pipeline_start {
            self.metrics.total_duration = start.elapsed();
        }
    }

    /// Set input count
    pub fn input_count(&mut self, count: usize) {
        self.metrics.input_count = count;
    }

    /// Set output count
    pub fn output_count(&mut self, count: usize) {
        self.metrics.output_count = count;
    }

    /// Set duplicate groups count
    pub fn duplicate_groups(&mut self, count: usize) {
        self.metrics.duplicate_groups = count;
    }

    /// Set merge count
    pub fn merge_count(&mut self, count: usize) {
        self.metrics.merge_count = count;
    }

    /// Set filtered count
    pub fn filtered_count(&mut self, count: usize) {
        self.metrics.filtered_count = count;
    }

    /// Set truncated count
    pub fn truncated_count(&mut self, count: usize) {
        self.metrics.truncated_count = count;
    }

    /// Add stage duration
    pub fn add_stage(&mut self, name: impl Into<String>, duration: Duration) {
        self.metrics.stage_durations.insert(name.into(), duration);
    }

    /// Build the final metrics
    pub fn build(self) -> PipelineMetrics {
        self.metrics
    }
}

impl Default for MetricsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_empty_metrics() {
        let metrics = PipelineMetrics::new();
        assert_eq!(metrics.input_count, 0);
        assert_eq!(metrics.output_count, 0);
        assert_eq!(metrics.deduplication_rate(), 0.0);
        assert_eq!(metrics.filter_rate(), 0.0);
        assert_eq!(metrics.throughput(), 0.0);
    }

    #[test]
    fn test_deduplication_rate() {
        let mut metrics = PipelineMetrics::new();
        metrics.input_count = 100;
        metrics.duplicate_groups = 80;

        // 20 out of 100 were deduplicated = 20%
        assert_eq!(metrics.deduplication_rate(), 20.0);
    }

    #[test]
    fn test_filter_rate() {
        let mut metrics = PipelineMetrics::new();
        metrics.input_count = 100;
        metrics.filtered_count = 25;

        assert_eq!(metrics.filter_rate(), 25.0);
    }

    #[test]
    fn test_throughput() {
        let mut metrics = PipelineMetrics::new();
        metrics.output_count = 100;
        metrics.total_duration = Duration::from_secs(2);

        assert_eq!(metrics.throughput(), 50.0); // 100 results / 2 seconds
    }

    #[test]
    fn test_stage_timer() {
        let mut metrics = PipelineMetrics::new();
        let timer = StageTimer::start("test_stage");

        thread::sleep(Duration::from_millis(10));
        let duration = timer.stop(&mut metrics);

        assert!(duration >= Duration::from_millis(10));
        assert!(metrics.stage_durations.contains_key("test_stage"));
        assert!(metrics.stage_durations["test_stage"] >= Duration::from_millis(10));
    }

    #[test]
    fn test_stage_timer_elapsed() {
        let timer = StageTimer::start("test");
        thread::sleep(Duration::from_millis(10));
        let elapsed = timer.elapsed();

        assert!(elapsed >= Duration::from_millis(10));
        assert_eq!(timer.stage_name(), "test");
    }

    #[test]
    fn test_metrics_builder() {
        let mut builder = MetricsBuilder::new();
        builder.start_pipeline();
        builder.input_count(100);
        builder.output_count(50);
        builder.duplicate_groups(80);
        builder.merge_count(20);
        builder.filtered_count(30);
        builder.truncated_count(0);
        builder.add_stage("stage1", Duration::from_millis(100));
        builder.add_stage("stage2", Duration::from_millis(200));

        thread::sleep(Duration::from_millis(10));
        builder.stop_pipeline();

        let metrics = builder.build();

        assert_eq!(metrics.input_count, 100);
        assert_eq!(metrics.output_count, 50);
        assert_eq!(metrics.duplicate_groups, 80);
        assert_eq!(metrics.merge_count, 20);
        assert_eq!(metrics.filtered_count, 30);
        assert_eq!(metrics.truncated_count, 0);
        assert!(metrics.total_duration >= Duration::from_millis(10));
        assert_eq!(metrics.stage_durations.len(), 2);
    }

    #[test]
    fn test_report_generation() {
        let mut metrics = PipelineMetrics::new();
        metrics.total_duration = Duration::from_millis(1000);
        metrics.input_count = 100;
        metrics.output_count = 50;
        metrics.duplicate_groups = 80;
        metrics.merge_count = 15;
        metrics.filtered_count = 20;
        metrics.truncated_count = 0;
        metrics
            .stage_durations
            .insert("Stage1".to_string(), Duration::from_millis(300));
        metrics
            .stage_durations
            .insert("Stage2".to_string(), Duration::from_millis(700));

        let report = metrics.report();

        // Check that report contains key information
        assert!(report.contains("Pipeline Metrics"));
        assert!(report.contains("1000ms"));
        assert!(report.contains("Input Count: 100"));
        assert!(report.contains("Output Count: 50"));
        assert!(report.contains("Stage1"));
        assert!(report.contains("Stage2"));
    }

    #[test]
    fn test_report_stage_sorting() {
        let mut metrics = PipelineMetrics::new();
        metrics.total_duration = Duration::from_millis(1000);
        metrics
            .stage_durations
            .insert("Fast".to_string(), Duration::from_millis(100));
        metrics
            .stage_durations
            .insert("Slow".to_string(), Duration::from_millis(900));

        let report = metrics.report();

        // "Slow" should appear before "Fast" in the report (sorted by duration)
        let slow_pos = report.find("Slow").unwrap();
        let fast_pos = report.find("Fast").unwrap();
        assert!(slow_pos < fast_pos);
    }

    #[test]
    fn test_zero_duration_throughput() {
        let mut metrics = PipelineMetrics::new();
        metrics.output_count = 100;
        metrics.total_duration = Duration::ZERO;

        // Should not panic, should return 0.0
        assert_eq!(metrics.throughput(), 0.0);
    }

    #[test]
    fn test_zero_input_deduplication_rate() {
        let mut metrics = PipelineMetrics::new();
        metrics.input_count = 0;
        metrics.duplicate_groups = 0;

        // Should not panic, should return 0.0
        assert_eq!(metrics.deduplication_rate(), 0.0);
    }

    #[test]
    fn test_zero_input_filter_rate() {
        let mut metrics = PipelineMetrics::new();
        metrics.input_count = 0;
        metrics.filtered_count = 0;

        // Should not panic, should return 0.0
        assert_eq!(metrics.filter_rate(), 0.0);
    }

    #[test]
    fn test_100_percent_deduplication() {
        let mut metrics = PipelineMetrics::new();
        metrics.input_count = 100;
        metrics.duplicate_groups = 100;

        // No deduplication occurred
        assert_eq!(metrics.deduplication_rate(), 0.0);
    }

    #[test]
    fn test_100_percent_filtering() {
        let mut metrics = PipelineMetrics::new();
        metrics.input_count = 100;
        metrics.filtered_count = 100;

        // All filtered out
        assert_eq!(metrics.filter_rate(), 100.0);
    }

    #[test]
    fn test_metrics_builder_without_start() {
        let mut builder = MetricsBuilder::new();
        builder.stop_pipeline();

        let metrics = builder.build();
        // Should have zero duration if never started
        assert_eq!(metrics.total_duration, Duration::ZERO);
    }

    #[test]
    fn test_stage_timer_multiple_stops() {
        let mut metrics1 = PipelineMetrics::new();

        let timer = StageTimer::start("test");
        thread::sleep(Duration::from_millis(5));

        // First stop
        let duration1 = timer.stop(&mut metrics1);

        // Timer is consumed, can't stop again (this tests the move semantics)
        assert!(duration1 >= Duration::from_millis(5));
        assert!(metrics1.stage_durations.contains_key("test"));
    }

    #[test]
    fn test_very_fast_operation() {
        let timer = StageTimer::start("instant");
        let mut metrics = PipelineMetrics::new();

        // Stop immediately
        let duration = timer.stop(&mut metrics);

        // Should measure something, even if very small
        assert!(duration >= Duration::ZERO);
    }

    #[test]
    fn test_stage_names_preserved() {
        let mut builder = MetricsBuilder::new();
        builder.add_stage("Stage One", Duration::from_millis(10));
        builder.add_stage("Stage Two", Duration::from_millis(20));

        let metrics = builder.build();
        assert!(metrics.stage_durations.contains_key("Stage One"));
        assert!(metrics.stage_durations.contains_key("Stage Two"));
        assert_eq!(metrics.stage_durations.len(), 2);
    }

    #[test]
    fn test_metrics_with_no_stages() {
        let metrics = PipelineMetrics::new();
        let report = metrics.report();

        // Report should still be valid with empty stages
        assert!(report.contains("Pipeline Metrics"));
        assert!(report.contains("Stage Durations"));
    }

    #[test]
    fn test_extremely_high_throughput() {
        let mut metrics = PipelineMetrics::new();
        metrics.output_count = 1_000_000;
        metrics.total_duration = Duration::from_millis(1);

        let throughput = metrics.throughput();
        // 1 million results in 0.001 seconds = 1 billion per second
        assert!(throughput > 900_000_000.0);
    }

    #[test]
    fn test_metrics_builder_all_fields() {
        let mut builder = MetricsBuilder::new();
        builder.start_pipeline();
        builder.input_count(100);
        builder.output_count(50);
        builder.duplicate_groups(80);
        builder.merge_count(20);
        builder.filtered_count(30);
        builder.truncated_count(10);

        thread::sleep(Duration::from_millis(5));
        builder.stop_pipeline();

        let metrics = builder.build();
        assert_eq!(metrics.input_count, 100);
        assert_eq!(metrics.output_count, 50);
        assert_eq!(metrics.duplicate_groups, 80);
        assert_eq!(metrics.merge_count, 20);
        assert_eq!(metrics.filtered_count, 30);
        assert_eq!(metrics.truncated_count, 10);
        assert!(metrics.total_duration >= Duration::from_millis(5));
    }
}
