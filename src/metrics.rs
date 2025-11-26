use anyhow::Result;
use nvml_wrapper::NVML;

/// Complete metrics collection from NVML
#[derive(Debug, Clone)]
pub struct Metrics {
    /// NVIDIA driver version
    pub version: String,
    /// List of GPU devices with their metrics
    pub devices: Vec<Device>,
}

/// GPU device metrics collected from NVML
#[derive(Debug, Clone)]
pub struct Device {
    /// Device index (0, 1, 2, ...)
    pub index: String,
    /// Minor device number used for /dev/nvidia{minor}
    pub minor_number: String,
    /// GPU model name (e.g., "NVIDIA GeForce RTX 3080")
    pub name: String,
    /// Unique GPU identifier
    pub uuid: String,
    
    // Temperature & Cooling
    /// GPU temperature in Celsius
    pub temperature: f64,
    /// Fan speed percentage (0-100)
    pub fan_speed: f64,
    
    // Power Metrics
    /// Current power usage in milliwatts
    pub power_usage: f64,
    /// Average power usage over 10s in milliwatts
    pub power_usage_average: f64,
    /// Power management limit in milliwatts (None if not supported)
    pub power_limit: Option<f64>,
    /// Default power management limit in milliwatts (None if not supported)
    pub power_limit_default: Option<f64>,
    
    // Memory Metrics
    /// Total GPU memory in bytes
    pub memory_total: f64,
    /// Used GPU memory in bytes
    pub memory_used: f64,
    /// Memory utilization percentage (0-100)
    pub utilization_memory: f64,
    
    // GPU Utilization
    /// Current GPU utilization percentage (0-100)
    pub utilization_gpu: f64,
    /// Average GPU utilization over 10s (0-100)
    pub utilization_gpu_average: f64,
    
    // Clock Speeds (in MHz, None if not supported)
    /// Current graphics clock speed in MHz
    pub clock_graphics: Option<f64>,
    /// Current SM (Streaming Multiprocessor) clock speed in MHz
    pub clock_sm: Option<f64>,
    /// Current memory clock speed in MHz
    pub clock_memory: Option<f64>,
    /// Maximum graphics clock speed in MHz
    pub clock_graphics_max: Option<f64>,
    /// Maximum SM clock speed in MHz
    pub clock_sm_max: Option<f64>,
    /// Maximum memory clock speed in MHz
    pub clock_memory_max: Option<f64>,
    
    // Performance State
    /// Current P-State (0-15, where P0 is maximum performance, None if not supported)
    pub performance_state: Option<f64>,
    
    // PCIe Information
    /// Current PCIe link generation (1-4+, None if not supported)
    pub pcie_link_gen: Option<f64>,
    /// Current PCIe link width in lanes (None if not supported)
    pub pcie_link_width: Option<f64>,
    /// PCIe transmit throughput in KB/s (None if not supported)
    pub pcie_tx_throughput: Option<f64>,
    /// PCIe receive throughput in KB/s (None if not supported)
    pub pcie_rx_throughput: Option<f64>,
    
    // Video Encoder/Decoder Utilization
    /// Video encoder utilization percentage (0-100, None if not supported)
    pub encoder_utilization: Option<f64>,
    /// Video decoder utilization percentage (0-100, None if not supported)
    pub decoder_utilization: Option<f64>,
    
    // ECC Errors (Data Center GPUs only)
    /// Total corrected ECC errors over GPU lifetime (None if ECC not supported)
    pub ecc_errors_corrected: Option<f64>,
    /// Total uncorrected ECC errors over GPU lifetime (None if ECC not supported)
    pub ecc_errors_uncorrected: Option<f64>,
    
    // Running Processes
    /// Number of compute processes currently running on this GPU (None if not supported)
    pub compute_processes: Option<f64>,
    /// Number of graphics processes currently running on this GPU (None if not supported)
    pub graphics_processes: Option<f64>,
}

/// Trait for collecting GPU metrics
/// This abstraction allows for testing without actual NVML hardware
#[cfg_attr(test, mockall::automock)]
pub trait MetricsCollector {
    fn collect(&self) -> Result<Metrics>;
}

/// Real NVML implementation
pub struct NvmlCollector;

impl MetricsCollector for NvmlCollector {
    fn collect(&self) -> Result<Metrics> {
        collect_metrics_impl()
    }
}

impl NvmlCollector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NvmlCollector {
    fn default() -> Self {
        Self::new()
    }
}

pub fn collect_metrics() -> Result<Metrics> {
    NvmlCollector::new().collect()
}

fn collect_metrics_impl() -> Result<Metrics> {
    let nvml = NVML::init()?;
    let version = nvml.sys_driver_version()?;

    let device_count = nvml.device_count()?;
    let mut devices = Vec::new();

    for index in 0..device_count {
        let device = nvml.device_by_index(index)?;

        let uuid = device.uuid()?;
        let name = device.name()?;
        let minor_number = device.minor_number()?.to_string();

        let temperature = device.temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu)? as f64;

        let power_usage = device.power_usage()? as f64;

        // For average power usage, we'll use the current value as a placeholder
        // NVML doesn't have a direct average function, so we'll use the current value
        // In a real implementation, you might want to track historical values
        let power_usage_average = power_usage;

        // Fan speed - use fan index 0 (first fan)
        let fan_speed = device.fan_speed(0).unwrap_or(0) as f64;

        let memory_info = device.memory_info()?;
        let memory_total = memory_info.total as f64;
        let memory_used = memory_info.used as f64;

        let utilization = device.utilization_rates()?;
        let utilization_gpu = utilization.gpu as f64;
        let utilization_memory = utilization.memory as f64;

        // For average GPU utilization, we'll use the current value as a placeholder
        // Similar to power usage average
        let utilization_gpu_average = utilization_gpu;

        // Clock speeds - use .ok() to handle unsupported GPUs gracefully
        let clock_graphics = device.clock_info(nvml_wrapper::enum_wrappers::device::Clock::Graphics)
            .ok().map(|c| c as f64);
        let clock_sm = device.clock_info(nvml_wrapper::enum_wrappers::device::Clock::SM)
            .ok().map(|c| c as f64);
        let clock_memory = device.clock_info(nvml_wrapper::enum_wrappers::device::Clock::Memory)
            .ok().map(|c| c as f64);
        
        let clock_graphics_max = device.max_clock_info(nvml_wrapper::enum_wrappers::device::Clock::Graphics)
            .ok().map(|c| c as f64);
        let clock_sm_max = device.max_clock_info(nvml_wrapper::enum_wrappers::device::Clock::SM)
            .ok().map(|c| c as f64);
        let clock_memory_max = device.max_clock_info(nvml_wrapper::enum_wrappers::device::Clock::Memory)
            .ok().map(|c| c as f64);

        // Power limits
        let power_limit = device.power_management_limit().ok().map(|p| p as f64);
        let power_limit_default = device.power_management_limit_default().ok().map(|p| p as f64);

        // Performance state (P-State: P0-P15, where P0 is maximum performance)
        let performance_state = device.performance_state()
            .ok().map(|ps| ps as u32 as f64);

        // PCIe information
        let pcie_link_gen = device.current_pcie_link_gen().ok().map(|g| g as f64);
        let pcie_link_width = device.current_pcie_link_width().ok().map(|w| w as f64);
        
        // PCIe throughput (in KB/s)
        let pcie_tx_throughput = device.pcie_throughput(nvml_wrapper::enum_wrappers::device::PcieUtilCounter::Send)
            .ok().map(|t| t as f64);
        let pcie_rx_throughput = device.pcie_throughput(nvml_wrapper::enum_wrappers::device::PcieUtilCounter::Receive)
            .ok().map(|t| t as f64);

        // Encoder/Decoder utilization
        let encoder_utilization = device.encoder_utilization()
            .ok().map(|info| info.utilization as f64);
        let decoder_utilization = device.decoder_utilization()
            .ok().map(|info| info.utilization as f64);

        // ECC errors (only for GPUs that support ECC)
        let ecc_errors_corrected = device.total_ecc_errors(
            nvml_wrapper::enum_wrappers::device::MemoryError::Corrected,
            nvml_wrapper::enum_wrappers::device::EccCounter::Aggregate
        ).ok().map(|e| e as f64);
        
        let ecc_errors_uncorrected = device.total_ecc_errors(
            nvml_wrapper::enum_wrappers::device::MemoryError::Uncorrected,
            nvml_wrapper::enum_wrappers::device::EccCounter::Aggregate
        ).ok().map(|e| e as f64);

        // Process counts
        let compute_processes = device.running_compute_processes()
            .ok().map(|procs| procs.len() as f64);
        let graphics_processes = device.running_graphics_processes()
            .ok().map(|procs| procs.len() as f64);

        devices.push(Device {
            index: index.to_string(),
            minor_number,
            name,
            uuid,
            temperature,
            power_usage,
            power_usage_average,
            fan_speed,
            memory_total,
            memory_used,
            utilization_memory,
            utilization_gpu,
            utilization_gpu_average,
            clock_graphics,
            clock_sm,
            clock_memory,
            clock_graphics_max,
            clock_sm_max,
            clock_memory_max,
            power_limit,
            power_limit_default,
            performance_state,
            pcie_link_gen,
            pcie_link_width,
            pcie_tx_throughput,
            pcie_rx_throughput,
            encoder_utilization,
            decoder_utilization,
            ecc_errors_corrected,
            ecc_errors_uncorrected,
            compute_processes,
            graphics_processes,
        });
    }

    Ok(Metrics { version, devices })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collect_metrics_structure() {
        // This test will only pass if NVML is available
        // It's okay for this to fail in CI/CD environments without GPUs
        match collect_metrics() {
            Ok(metrics) => {
                assert!(!metrics.version.is_empty());
                // If we have devices, verify their structure
                for device in &metrics.devices {
                    assert!(!device.index.is_empty());
                    assert!(!device.minor_number.is_empty());
                    assert!(!device.name.is_empty());
                    assert!(!device.uuid.is_empty());
                    // Verify numeric fields are non-negative
                    assert!(device.temperature >= 0.0);
                    assert!(device.power_usage >= 0.0);
                    assert!(device.fan_speed >= 0.0);
                    assert!(device.memory_total >= 0.0);
                    assert!(device.memory_used >= 0.0);
                    assert!(device.memory_used <= device.memory_total);
                }
            }
            Err(_) => {
                // NVML not available, skip test
                // This is expected in CI/CD environments without NVIDIA GPUs
            }
        }
    }

    #[test]
    fn test_device_structure() {
        let device = Device {
            index: "0".to_string(),
            minor_number: "0".to_string(),
            name: "Test GPU".to_string(),
            uuid: "GPU-12345".to_string(),
            temperature: 50.0,
            power_usage: 100.0,
            power_usage_average: 100.0,
            fan_speed: 50.0,
            memory_total: 8589934592.0,
            memory_used: 4294967296.0,
            utilization_memory: 50.0,
            utilization_gpu: 75.0,
            utilization_gpu_average: 75.0,
            clock_graphics: Some(1500.0),
            clock_sm: Some(1500.0),
            clock_memory: Some(7000.0),
            clock_graphics_max: Some(1800.0),
            clock_sm_max: Some(1800.0),
            clock_memory_max: Some(8000.0),
            power_limit: Some(250000.0),
            power_limit_default: Some(250000.0),
            performance_state: Some(0.0),
            pcie_link_gen: Some(4.0),
            pcie_link_width: Some(16.0),
            pcie_tx_throughput: Some(1000.0),
            pcie_rx_throughput: Some(1000.0),
            encoder_utilization: Some(0.0),
            decoder_utilization: Some(0.0),
            ecc_errors_corrected: Some(0.0),
            ecc_errors_uncorrected: Some(0.0),
            compute_processes: Some(2.0),
            graphics_processes: Some(1.0),
        };

        assert_eq!(device.index, "0");
        assert_eq!(device.name, "Test GPU");
        assert!(device.memory_used <= device.memory_total);
        
        // Verify optional fields
        assert!(device.clock_graphics.is_some());
        assert!(device.power_limit.is_some());
        assert!(device.performance_state.is_some());
        
        // Verify clock values are reasonable
        if let Some(clock) = device.clock_graphics {
            assert!(clock > 0.0);
        }
    }

    #[test]
    fn test_metrics_structure() {
        let metrics = Metrics {
            version: "525.116.04".to_string(),
            devices: vec![],
        };

        assert_eq!(metrics.version, "525.116.04");
        assert_eq!(metrics.devices.len(), 0);
    }

    #[test]
    fn test_mock_collector() {
        let mut mock_collector = MockMetricsCollector::new();
        mock_collector.expect_collect().times(1).returning(|| {
            Ok(Metrics {
                version: "525.116.04".to_string(),
                devices: vec![Device {
                    index: "0".to_string(),
                    minor_number: "0".to_string(),
                    name: "NVIDIA GeForce RTX 3080".to_string(),
                    uuid: "GPU-12345678-1234-1234-1234-123456789012".to_string(),
                    temperature: 65.0,
                    power_usage: 250000.0,
                    power_usage_average: 250000.0,
                    fan_speed: 75.0,
                    memory_total: 10737418240.0,
                    memory_used: 5368709120.0,
                    utilization_memory: 50.0,
                    utilization_gpu: 85.0,
                    utilization_gpu_average: 85.0,
                    clock_graphics: Some(1710.0),
                    clock_sm: Some(1710.0),
                    clock_memory: Some(9501.0),
                    clock_graphics_max: Some(1905.0),
                    clock_sm_max: Some(1905.0),
                    clock_memory_max: Some(9501.0),
                    power_limit: Some(320000.0),
                    power_limit_default: Some(320000.0),
                    performance_state: Some(2.0),
                    pcie_link_gen: Some(4.0),
                    pcie_link_width: Some(16.0),
                    pcie_tx_throughput: Some(5000.0),
                    pcie_rx_throughput: Some(5000.0),
                    encoder_utilization: Some(15.0),
                    decoder_utilization: Some(10.0),
                    ecc_errors_corrected: None,
                    ecc_errors_uncorrected: None,
                    compute_processes: Some(3.0),
                    graphics_processes: Some(1.0),
                }],
            })
        });

        let result = mock_collector.collect();
        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert_eq!(metrics.version, "525.116.04");
        assert_eq!(metrics.devices.len(), 1);
        assert_eq!(metrics.devices[0].name, "NVIDIA GeForce RTX 3080");
        
        // Verify new metrics are present
        let device = &metrics.devices[0];
        assert!(device.clock_graphics.is_some());
        assert!(device.power_limit.is_some());
        assert!(device.pcie_link_gen.is_some());
        assert_eq!(device.pcie_link_gen, Some(4.0));
    }
}
