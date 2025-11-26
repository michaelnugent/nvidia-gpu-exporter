use crate::metrics::collect_metrics;
use prometheus::{
    core::Collector,
    proto::MetricFamily,
    Gauge, GaugeVec, Opts,
};
use tracing::{debug, warn};

const NAMESPACE: &str = "nvidia";

#[derive(Clone)]
pub struct Exporter {
    up: Gauge,
    info: GaugeVec,
    device_count: Gauge,
    temperatures: GaugeVec,
    device_info: GaugeVec,
    power_usage: GaugeVec,
    power_usage_average: GaugeVec,
    fan_speed: GaugeVec,
    memory_total: GaugeVec,
    memory_used: GaugeVec,
    utilization_memory: GaugeVec,
    utilization_gpu: GaugeVec,
    utilization_gpu_average: GaugeVec,
    // Clock speeds
    clock_graphics: GaugeVec,
    clock_sm: GaugeVec,
    clock_memory: GaugeVec,
    clock_graphics_max: GaugeVec,
    clock_sm_max: GaugeVec,
    clock_memory_max: GaugeVec,
    // Power limits
    power_limit: GaugeVec,
    power_limit_default: GaugeVec,
    // Performance state
    performance_state: GaugeVec,
    // PCIe
    pcie_link_gen: GaugeVec,
    pcie_link_width: GaugeVec,
    pcie_tx_throughput: GaugeVec,
    pcie_rx_throughput: GaugeVec,
    // Encoder/Decoder
    encoder_utilization: GaugeVec,
    decoder_utilization: GaugeVec,
    // ECC errors
    ecc_errors_corrected: GaugeVec,
    ecc_errors_uncorrected: GaugeVec,
    // Processes
    compute_processes: GaugeVec,
    graphics_processes: GaugeVec,
}

impl Default for Exporter {
    fn default() -> Self {
        Self::new()
    }
}

impl Exporter {
    pub fn new() -> Self {
        Self {
            up: Gauge::with_opts(Opts::new("up", "NVML Metric Collection Operational")
                .namespace(NAMESPACE))
                .expect("Failed to create up metric"),
            info: GaugeVec::new(
                Opts::new("driver_info", "NVML Info").namespace(NAMESPACE),
                &["version"],
            )
            .expect("Failed to create driver_info metric"),
            device_count: Gauge::with_opts(
                Opts::new("device_count", "Count of found nvidia devices")
                    .namespace(NAMESPACE),
            )
            .expect("Failed to create device_count metric"),
            device_info: GaugeVec::new(
                Opts::new("info", "Info as reported by the device").namespace(NAMESPACE),
                &["index", "minor", "uuid", "name"],
            )
            .expect("Failed to create info metric"),
            temperatures: GaugeVec::new(
                Opts::new("temperatures", "Temperature as reported by the device")
                    .namespace(NAMESPACE),
                &["minor"],
            )
            .expect("Failed to create temperatures metric"),
            power_usage: GaugeVec::new(
                Opts::new("power_usage", "Power usage as reported by the device")
                    .namespace(NAMESPACE),
                &["minor"],
            )
            .expect("Failed to create power_usage metric"),
            power_usage_average: GaugeVec::new(
                Opts::new(
                    "power_usage_average",
                    "Power usage as reported by the device averaged over 10s",
                )
                .namespace(NAMESPACE),
                &["minor"],
            )
            .expect("Failed to create power_usage_average metric"),
            fan_speed: GaugeVec::new(
                Opts::new("fanspeed", "Fan speed as reported by the device")
                    .namespace(NAMESPACE),
                &["minor"],
            )
            .expect("Failed to create fanspeed metric"),
            memory_total: GaugeVec::new(
                Opts::new("memory_total", "Total memory as reported by the device")
                    .namespace(NAMESPACE),
                &["minor"],
            )
            .expect("Failed to create memory_total metric"),
            memory_used: GaugeVec::new(
                Opts::new("memory_used", "Used memory as reported by the device")
                    .namespace(NAMESPACE),
                &["minor"],
            )
            .expect("Failed to create memory_used metric"),
            utilization_memory: GaugeVec::new(
                Opts::new("utilization_memory", "Memory Utilization as reported by the device")
                    .namespace(NAMESPACE),
                &["minor"],
            )
            .expect("Failed to create utilization_memory metric"),
            utilization_gpu: GaugeVec::new(
                Opts::new("utilization_gpu", "GPU utilization as reported by the device")
                    .namespace(NAMESPACE),
                &["minor"],
            )
            .expect("Failed to create utilization_gpu metric"),
            utilization_gpu_average: GaugeVec::new(
                Opts::new(
                    "utilization_gpu_average",
                    "GPU utilization as reported by the device averaged over 10s",
                )
                .namespace(NAMESPACE),
                &["minor"],
            )
            .expect("Failed to create utilization_gpu_average metric"),
            // Clock speeds in MHz
            clock_graphics: GaugeVec::new(
                Opts::new("clock_graphics_mhz", "Graphics clock speed in MHz")
                    .namespace(NAMESPACE),
                &["minor"],
            )
            .expect("Failed to create clock_graphics metric"),
            clock_sm: GaugeVec::new(
                Opts::new("clock_sm_mhz", "SM clock speed in MHz")
                    .namespace(NAMESPACE),
                &["minor"],
            )
            .expect("Failed to create clock_sm metric"),
            clock_memory: GaugeVec::new(
                Opts::new("clock_memory_mhz", "Memory clock speed in MHz")
                    .namespace(NAMESPACE),
                &["minor"],
            )
            .expect("Failed to create clock_memory metric"),
            clock_graphics_max: GaugeVec::new(
                Opts::new("clock_graphics_max_mhz", "Maximum graphics clock speed in MHz")
                    .namespace(NAMESPACE),
                &["minor"],
            )
            .expect("Failed to create clock_graphics_max metric"),
            clock_sm_max: GaugeVec::new(
                Opts::new("clock_sm_max_mhz", "Maximum SM clock speed in MHz")
                    .namespace(NAMESPACE),
                &["minor"],
            )
            .expect("Failed to create clock_sm_max metric"),
            clock_memory_max: GaugeVec::new(
                Opts::new("clock_memory_max_mhz", "Maximum memory clock speed in MHz")
                    .namespace(NAMESPACE),
                &["minor"],
            )
            .expect("Failed to create clock_memory_max metric"),
            // Power limits in milliwatts
            power_limit: GaugeVec::new(
                Opts::new("power_limit_milliwatts", "Power management limit in milliwatts")
                    .namespace(NAMESPACE),
                &["minor"],
            )
            .expect("Failed to create power_limit metric"),
            power_limit_default: GaugeVec::new(
                Opts::new("power_limit_default_milliwatts", "Default power management limit in milliwatts")
                    .namespace(NAMESPACE),
                &["minor"],
            )
            .expect("Failed to create power_limit_default metric"),
            // Performance state (P0-P15)
            performance_state: GaugeVec::new(
                Opts::new("performance_state", "Current performance state (P-State: 0-15, lower is better)")
                    .namespace(NAMESPACE),
                &["minor"],
            )
            .expect("Failed to create performance_state metric"),
            // PCIe metrics
            pcie_link_gen: GaugeVec::new(
                Opts::new("pcie_link_generation", "PCIe link generation")
                    .namespace(NAMESPACE),
                &["minor"],
            )
            .expect("Failed to create pcie_link_gen metric"),
            pcie_link_width: GaugeVec::new(
                Opts::new("pcie_link_width", "PCIe link width")
                    .namespace(NAMESPACE),
                &["minor"],
            )
            .expect("Failed to create pcie_link_width metric"),
            pcie_tx_throughput: GaugeVec::new(
                Opts::new("pcie_tx_throughput_kb", "PCIe transmit throughput in KB/s")
                    .namespace(NAMESPACE),
                &["minor"],
            )
            .expect("Failed to create pcie_tx_throughput metric"),
            pcie_rx_throughput: GaugeVec::new(
                Opts::new("pcie_rx_throughput_kb", "PCIe receive throughput in KB/s")
                    .namespace(NAMESPACE),
                &["minor"],
            )
            .expect("Failed to create pcie_rx_throughput metric"),
            // Encoder/Decoder utilization (0-100%)
            encoder_utilization: GaugeVec::new(
                Opts::new("encoder_utilization", "Encoder utilization percentage (0-100)")
                    .namespace(NAMESPACE),
                &["minor"],
            )
            .expect("Failed to create encoder_utilization metric"),
            decoder_utilization: GaugeVec::new(
                Opts::new("decoder_utilization", "Decoder utilization percentage (0-100)")
                    .namespace(NAMESPACE),
                &["minor"],
            )
            .expect("Failed to create decoder_utilization metric"),
            // ECC errors
            ecc_errors_corrected: GaugeVec::new(
                Opts::new("ecc_errors_corrected_total", "Total corrected ECC errors")
                    .namespace(NAMESPACE),
                &["minor"],
            )
            .expect("Failed to create ecc_errors_corrected metric"),
            ecc_errors_uncorrected: GaugeVec::new(
                Opts::new("ecc_errors_uncorrected_total", "Total uncorrected ECC errors")
                    .namespace(NAMESPACE),
                &["minor"],
            )
            .expect("Failed to create ecc_errors_uncorrected metric"),
            // Process counts
            compute_processes: GaugeVec::new(
                Opts::new("compute_processes", "Number of compute processes running")
                    .namespace(NAMESPACE),
                &["minor"],
            )
            .expect("Failed to create compute_processes metric"),
            graphics_processes: GaugeVec::new(
                Opts::new("graphics_processes", "Number of graphics processes running")
                    .namespace(NAMESPACE),
                &["minor"],
            )
            .expect("Failed to create graphics_processes metric"),
        }
    }

    pub fn gather(&self) -> Vec<MetricFamily> {
        debug!("Starting metrics collection...");
        match collect_metrics() {
            Ok(data) => {
                debug!("Successfully collected metrics: version={}, device_count={}", data.version, data.devices.len());
                self.up.set(1.0);
                self.info.with_label_values(&[&data.version]).set(1.0);
                self.device_count.set(data.devices.len() as f64);

                for device in &data.devices {
                    self.device_info
                        .with_label_values(&[
                            &device.index,
                            &device.minor_number,
                            &device.uuid,
                            &device.name,
                        ])
                        .set(1.0);
                    self.fan_speed
                        .with_label_values(&[&device.minor_number])
                        .set(device.fan_speed);
                    self.memory_total
                        .with_label_values(&[&device.minor_number])
                        .set(device.memory_total);
                    self.memory_used
                        .with_label_values(&[&device.minor_number])
                        .set(device.memory_used);
                    self.power_usage
                        .with_label_values(&[&device.minor_number])
                        .set(device.power_usage);
                    self.power_usage_average
                        .with_label_values(&[&device.minor_number])
                        .set(device.power_usage_average);
                    self.temperatures
                        .with_label_values(&[&device.minor_number])
                        .set(device.temperature);
                    self.utilization_gpu
                        .with_label_values(&[&device.minor_number])
                        .set(device.utilization_gpu);
                    self.utilization_gpu_average
                        .with_label_values(&[&device.minor_number])
                        .set(device.utilization_gpu_average);
                    self.utilization_memory
                        .with_label_values(&[&device.minor_number])
                        .set(device.utilization_memory);
                    
                    // Clock speeds - set 0 if not available
                    self.clock_graphics
                        .with_label_values(&[&device.minor_number])
                        .set(device.clock_graphics.unwrap_or(0.0));
                    self.clock_sm
                        .with_label_values(&[&device.minor_number])
                        .set(device.clock_sm.unwrap_or(0.0));
                    self.clock_memory
                        .with_label_values(&[&device.minor_number])
                        .set(device.clock_memory.unwrap_or(0.0));
                    self.clock_graphics_max
                        .with_label_values(&[&device.minor_number])
                        .set(device.clock_graphics_max.unwrap_or(0.0));
                    self.clock_sm_max
                        .with_label_values(&[&device.minor_number])
                        .set(device.clock_sm_max.unwrap_or(0.0));
                    self.clock_memory_max
                        .with_label_values(&[&device.minor_number])
                        .set(device.clock_memory_max.unwrap_or(0.0));
                    
                    // Power limits
                    self.power_limit
                        .with_label_values(&[&device.minor_number])
                        .set(device.power_limit.unwrap_or(0.0));
                    self.power_limit_default
                        .with_label_values(&[&device.minor_number])
                        .set(device.power_limit_default.unwrap_or(0.0));
                    
                    // Performance state
                    self.performance_state
                        .with_label_values(&[&device.minor_number])
                        .set(device.performance_state.unwrap_or(0.0));
                    
                    // PCIe metrics
                    self.pcie_link_gen
                        .with_label_values(&[&device.minor_number])
                        .set(device.pcie_link_gen.unwrap_or(0.0));
                    self.pcie_link_width
                        .with_label_values(&[&device.minor_number])
                        .set(device.pcie_link_width.unwrap_or(0.0));
                    self.pcie_tx_throughput
                        .with_label_values(&[&device.minor_number])
                        .set(device.pcie_tx_throughput.unwrap_or(0.0));
                    self.pcie_rx_throughput
                        .with_label_values(&[&device.minor_number])
                        .set(device.pcie_rx_throughput.unwrap_or(0.0));
                    
                    // Encoder/Decoder
                    self.encoder_utilization
                        .with_label_values(&[&device.minor_number])
                        .set(device.encoder_utilization.unwrap_or(0.0));
                    self.decoder_utilization
                        .with_label_values(&[&device.minor_number])
                        .set(device.decoder_utilization.unwrap_or(0.0));
                    
                    // ECC errors
                    self.ecc_errors_corrected
                        .with_label_values(&[&device.minor_number])
                        .set(device.ecc_errors_corrected.unwrap_or(0.0));
                    self.ecc_errors_uncorrected
                        .with_label_values(&[&device.minor_number])
                        .set(device.ecc_errors_uncorrected.unwrap_or(0.0));
                    
                    // Processes
                    self.compute_processes
                        .with_label_values(&[&device.minor_number])
                        .set(device.compute_processes.unwrap_or(0.0));
                    self.graphics_processes
                        .with_label_values(&[&device.minor_number])
                        .set(device.graphics_processes.unwrap_or(0.0));
                }
                debug!("Processed {} devices", data.devices.len());
            }
            Err(e) => {
                warn!("Failed to collect metrics (NVML unavailable): {}. Reporting up=0, device_count=0", e);
                self.up.set(0.0);
                self.device_count.set(0.0);
                // Set driver_info to "unavailable" when NVML fails so the metric is always present
                self.info.with_label_values(&["unavailable"]).set(1.0);
            }
        }

        debug!("Collecting metric families...");
        let mut mfs = Vec::new();
        
        // Helper to filter out empty metric families
        let mut add_metrics = |metrics: Vec<MetricFamily>| {
            for mf in metrics {
                // Only add metric families that have at least one metric
                if !mf.get_metric().is_empty() {
                    mfs.push(mf);
                } else {
                    debug!("Skipping empty metric family: {}", mf.get_name());
                }
            }
        };
        
        add_metrics(self.device_count.collect());
        add_metrics(self.device_info.collect());
        add_metrics(self.fan_speed.collect());
        add_metrics(self.info.collect());
        add_metrics(self.memory_total.collect());
        add_metrics(self.memory_used.collect());
        add_metrics(self.power_usage.collect());
        add_metrics(self.power_usage_average.collect());
        add_metrics(self.temperatures.collect());
        add_metrics(self.up.collect());
        add_metrics(self.utilization_gpu.collect());
        add_metrics(self.utilization_gpu_average.collect());
        add_metrics(self.utilization_memory.collect());
        // Clock speeds
        add_metrics(self.clock_graphics.collect());
        add_metrics(self.clock_sm.collect());
        add_metrics(self.clock_memory.collect());
        add_metrics(self.clock_graphics_max.collect());
        add_metrics(self.clock_sm_max.collect());
        add_metrics(self.clock_memory_max.collect());
        // Power limits
        add_metrics(self.power_limit.collect());
        add_metrics(self.power_limit_default.collect());
        // Performance state
        add_metrics(self.performance_state.collect());
        // PCIe
        add_metrics(self.pcie_link_gen.collect());
        add_metrics(self.pcie_link_width.collect());
        add_metrics(self.pcie_tx_throughput.collect());
        add_metrics(self.pcie_rx_throughput.collect());
        // Encoder/Decoder
        add_metrics(self.encoder_utilization.collect());
        add_metrics(self.decoder_utilization.collect());
        // ECC errors
        add_metrics(self.ecc_errors_corrected.collect());
        add_metrics(self.ecc_errors_uncorrected.collect());
        // Processes
        add_metrics(self.compute_processes.collect());
        add_metrics(self.graphics_processes.collect());

        debug!("Collected {} metric families total (after filtering empty ones)", mfs.len());
        mfs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exporter_creation() {
        let exporter = Exporter::new();
        let mfs = exporter.gather();
        // Should always have at least the 'up' metric
        assert!(!mfs.is_empty());
    }

    #[test]
    fn test_exporter_default_trait() {
        let exporter = Exporter::default();
        let mfs = exporter.gather();
        // Should work the same as new()
        assert!(!mfs.is_empty());
    }

    #[test]
    fn test_exporter_metrics_structure() {
        let exporter = Exporter::new();
        let mfs = exporter.gather();

        // Check that we have the expected metric families
        let metric_names: Vec<String> = mfs.iter().map(|mf| mf.get_name().to_string()).collect();
        
        assert!(metric_names.contains(&format!("{}_up", NAMESPACE)));
        assert!(metric_names.contains(&format!("{}_device_count", NAMESPACE)));
        assert!(metric_names.contains(&format!("{}_driver_info", NAMESPACE)));
    }

    #[test]
    fn test_all_expected_metrics_present() {
        let exporter = Exporter::new();
        let mfs = exporter.gather();

        let metric_names: Vec<String> = mfs.iter().map(|mf| mf.get_name().to_string()).collect();
        
        // Core metrics that should always be present
        let core_metrics = vec![
            format!("{}_up", NAMESPACE),
            format!("{}_device_count", NAMESPACE),
            format!("{}_driver_info", NAMESPACE),
        ];

        for expected in core_metrics {
            assert!(
                metric_names.contains(&expected),
                "Missing core metric: {}",
                expected
            );
        }

        // Get device count to determine if device-specific metrics should be present
        let device_count_metric = mfs.iter()
            .find(|mf| mf.get_name() == format!("{}_device_count", NAMESPACE))
            .expect("device_count metric should be present");
        let device_count = if let Some(metric) = device_count_metric.get_metric().first() {
            metric.get_gauge().get_value() as u64
        } else {
            0
        };

        // Device-specific metrics should only be present if there are devices
        if device_count > 0 {
            let device_metrics = vec![
                format!("{}_info", NAMESPACE),
                format!("{}_temperatures", NAMESPACE),
                format!("{}_fanspeed", NAMESPACE),
                format!("{}_memory_total", NAMESPACE),
                format!("{}_memory_used", NAMESPACE),
                format!("{}_utilization_memory", NAMESPACE),
                format!("{}_utilization_gpu", NAMESPACE),
                format!("{}_utilization_gpu_average", NAMESPACE),
                format!("{}_power_usage", NAMESPACE),
                format!("{}_power_usage_average", NAMESPACE),
                format!("{}_power_limit_milliwatts", NAMESPACE),
                format!("{}_power_limit_default_milliwatts", NAMESPACE),
                format!("{}_clock_graphics_mhz", NAMESPACE),
                format!("{}_clock_sm_mhz", NAMESPACE),
                format!("{}_clock_memory_mhz", NAMESPACE),
                format!("{}_clock_graphics_max_mhz", NAMESPACE),
                format!("{}_clock_sm_max_mhz", NAMESPACE),
                format!("{}_clock_memory_max_mhz", NAMESPACE),
                format!("{}_performance_state", NAMESPACE),
                format!("{}_pcie_link_generation", NAMESPACE),
                format!("{}_pcie_link_width", NAMESPACE),
                format!("{}_pcie_tx_throughput_kb", NAMESPACE),
                format!("{}_pcie_rx_throughput_kb", NAMESPACE),
                format!("{}_encoder_utilization", NAMESPACE),
                format!("{}_decoder_utilization", NAMESPACE),
                format!("{}_ecc_errors_corrected_total", NAMESPACE),
                format!("{}_ecc_errors_uncorrected_total", NAMESPACE),
                format!("{}_compute_processes", NAMESPACE),
                format!("{}_graphics_processes", NAMESPACE),
            ];

            for expected in device_metrics {
                assert!(
                    metric_names.contains(&expected),
                    "Missing device metric: {} (device_count={})",
                    expected,
                    device_count
                );
            }
        }
    }

    #[test]
    fn test_exporter_clone() {
        let exporter1 = Exporter::new();
        let exporter2 = exporter1.clone();
        
        let mfs1 = exporter1.gather();
        let mfs2 = exporter2.gather();
        
        // Both should produce metrics
        assert!(!mfs1.is_empty());
        assert!(!mfs2.is_empty());
        
        // Should have the same number of metric families
        assert_eq!(mfs1.len(), mfs2.len());
    }

    #[test]
    fn test_up_metric_on_error() {
        let exporter = Exporter::new();
        
        // First gather - if NVML is not available, up should be 0
        let mfs = exporter.gather();
        
        // Find the up metric
        let up_metric = mfs.iter().find(|mf| mf.get_name() == format!("{}_up", NAMESPACE));
        assert!(up_metric.is_some(), "up metric should always be present");
    }

    #[test]
    fn test_device_count_metric_always_present() {
        let exporter = Exporter::new();
        let mfs = exporter.gather();
        
        // Find the device_count metric
        let device_count_metric = mfs.iter()
            .find(|mf| mf.get_name() == format!("{}_device_count", NAMESPACE));
        assert!(device_count_metric.is_some(), "device_count metric should always be present");
        
        // Verify it has at least one metric value
        let mf = device_count_metric.unwrap();
        assert!(!mf.get_metric().is_empty(), "device_count should have at least one metric value");
        
        // Get the actual value
        let metric = &mf.get_metric()[0];
        let value = metric.get_gauge().get_value();
        
        // Value should be >= 0 (0 if no GPUs or error, positive if GPUs found)
        assert!(value >= 0.0, "device_count should be >= 0, got {}", value);
    }

    #[test]
    fn test_device_count_set_to_zero_on_error() {
        let exporter = Exporter::new();
        let mfs = exporter.gather();
        
        // Find the device_count metric
        let device_count_metric = mfs.iter()
            .find(|mf| mf.get_name() == format!("{}_device_count", NAMESPACE))
            .expect("device_count metric should always be present");
        
        // Find the up metric to determine if there was an error
        let up_metric = mfs.iter()
            .find(|mf| mf.get_name() == format!("{}_up", NAMESPACE))
            .expect("up metric should always be present");
        
        let up_value = up_metric.get_metric()[0].get_gauge().get_value();
        let device_count_value = device_count_metric.get_metric()[0].get_gauge().get_value();
        
        // If up is 0 (error case), device_count should also be 0
        if up_value == 0.0 {
            assert_eq!(
                device_count_value, 0.0,
                "When up metric is 0 (error), device_count should be 0, got {}",
                device_count_value
            );
        }
        
        // device_count should always be >= 0 regardless
        assert!(device_count_value >= 0.0, "device_count should always be >= 0");
    }

    #[test]
    fn test_namespace_consistency() {
        let exporter = Exporter::new();
        let mfs = exporter.gather();

        // All metrics should have the correct namespace
        for mf in mfs {
            let name = mf.get_name();
            if !name.is_empty() {
                assert!(
                    name.starts_with(&format!("{}_", NAMESPACE)),
                    "Metric {} doesn't have correct namespace",
                    name
                );
            }
        }
    }

    #[test]
    fn test_metric_labels() {
        let exporter = Exporter::new();
        let mfs = exporter.gather();

        // Check that device-specific metrics have the expected labels
        let device_metrics = vec!["temperatures", "power_usage", "fanspeed", "memory_total", 
                                  "memory_used", "utilization_gpu", "utilization_memory"];
        
        for metric_suffix in device_metrics {
            let metric_name = format!("{}_{}", NAMESPACE, metric_suffix);
            if let Some(mf) = mfs.iter().find(|m| m.get_name() == metric_name) {
                // If there are any metrics, they should have the 'minor' label
                for metric in mf.get_metric() {
                    let labels: Vec<String> = metric.get_label()
                        .iter()
                        .map(|l| l.get_name().to_string())
                        .collect();
                    
                    if !labels.is_empty() {
                        assert!(labels.contains(&"minor".to_string()),
                            "Metric {} should have 'minor' label", metric_name);
                    }
                }
            }
        }
    }

    #[test]
    fn test_new_metrics_structure() {
        let exporter = Exporter::new();
        let mfs = exporter.gather();
        let metric_names: Vec<String> = mfs.iter().map(|mf| mf.get_name().to_string()).collect();
        
        // Get device count to determine if device-specific metrics should be present
        let device_count_metric = mfs.iter()
            .find(|mf| mf.get_name() == format!("{}_device_count", NAMESPACE))
            .expect("device_count metric should be present");
        let device_count = if let Some(metric) = device_count_metric.get_metric().first() {
            metric.get_gauge().get_value() as u64
        } else {
            0
        };

        // Device-specific metrics should only be present if there are devices
        if device_count > 0 {
            // Test clock metrics are present
            assert!(metric_names.contains(&format!("{}_clock_graphics_mhz", NAMESPACE)));
            assert!(metric_names.contains(&format!("{}_clock_memory_mhz", NAMESPACE)));
            
            // Test power limit metrics
            assert!(metric_names.contains(&format!("{}_power_limit_milliwatts", NAMESPACE)));
            
            // Test PCIe metrics
            assert!(metric_names.contains(&format!("{}_pcie_link_generation", NAMESPACE)));
            assert!(metric_names.contains(&format!("{}_pcie_link_width", NAMESPACE)));
            
            // Test encoder/decoder metrics
            assert!(metric_names.contains(&format!("{}_encoder_utilization", NAMESPACE)));
            assert!(metric_names.contains(&format!("{}_decoder_utilization", NAMESPACE)));
            
            // Test ECC metrics
            assert!(metric_names.contains(&format!("{}_ecc_errors_corrected_total", NAMESPACE)));
            
            // Test process metrics
            assert!(metric_names.contains(&format!("{}_compute_processes", NAMESPACE)));
            assert!(metric_names.contains(&format!("{}_graphics_processes", NAMESPACE)));
        } else {
            // When no devices, these metrics may be filtered out (which is expected)
            // Just verify core metrics are present
            assert!(metric_names.contains(&format!("{}_up", NAMESPACE)));
            assert!(metric_names.contains(&format!("{}_device_count", NAMESPACE)));
            assert!(metric_names.contains(&format!("{}_driver_info", NAMESPACE)));
        }
    }
}
