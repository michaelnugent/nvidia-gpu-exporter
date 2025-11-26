# NVIDIA Prometheus Exporter

A Prometheus exporter for NVIDIA GPUs using the NVIDIA Management Library (NVML)

## Requirements

The NVML shared library (`libnvidia-ml.so.1`) needs to be loadable. When running in a container it must be either baked in or mounted from the host.

### Library Loading Issues

If you are running a debian distro, you may encounter "cannot open shared object file" errors when libnvidia-ml1 package is installed.  The issue is that `libloading` (used by `nvml-wrapper`) looks for `libnvidia-ml.so` but the older package only installs a symlink to `libnvidia-ml.so.1`.

```bash
sudo ln -s /usr/lib/x86_64-linux-gnu/libnvidia-ml.so.1 /usr/lib/x86_64-linux-gnu/libnvidia-ml.so
```

This is a one-time setup that persists across reboots.

## Building

```bash
cargo build --release
```

## Running

```bash
./target/release/nvidia-gpu-exporter
```

### Command-line Options

- `--web-listen-address`: Address to listen on for web interface and telemetry (default: `0.0.0.0:9445`)
- `--web-telemetry-path`: Path under which to expose metrics (default: `/metrics`)

Example:

```bash
./target/release/nvidia-gpu-exporter --web-listen-address 0.0.0.0:9445 --web-telemetry-path /metrics
```

## Testing

```bash
cargo test
```

## Metrics

The exporter exposes the following Prometheus metrics:

### System Metrics

- `nvidia_up` - NVML Metric Collection Operational (1 = working, 0 = error)
- `nvidia_driver_info{version="..."}` - NVML driver version info
- `nvidia_device_count` - Count of NVIDIA GPU devices found

### Device Information

- `nvidia_info{index="...",minor="...",uuid="...",name="..."}` - Device metadata (always 1)

### Temperature & Cooling

- `nvidia_temperatures{minor="..."}` - GPU temperature in Celsius
- `nvidia_fanspeed{minor="..."}` - Fan speed percentage (0-100)

### Memory Metrics

- `nvidia_memory_total{minor="..."}` - Total memory in bytes
- `nvidia_memory_used{minor="..."}` - Used memory in bytes
- `nvidia_utilization_memory{minor="..."}` - Memory utilization percentage (0-100)

### GPU Utilization

- `nvidia_utilization_gpu{minor="..."}` - Current GPU utilization percentage (0-100)
- `nvidia_utilization_gpu_average{minor="..."}` - GPU utilization averaged over 10s (0-100)

### Power Metrics

- `nvidia_power_usage{minor="..."}` - Current power usage in milliwatts
- `nvidia_power_usage_average{minor="..."}` - Power usage averaged over 10s in milliwatts
- `nvidia_power_limit_milliwatts{minor="..."}` - Current power management limit in milliwatts
- `nvidia_power_limit_default_milliwatts{minor="..."}` - Default power management limit in milliwatts

### Clock Speeds

- `nvidia_clock_graphics_mhz{minor="..."}` - Current graphics clock speed in MHz
- `nvidia_clock_sm_mhz{minor="..."}` - Current SM (Streaming Multiprocessor) clock speed in MHz
- `nvidia_clock_memory_mhz{minor="..."}` - Current memory clock speed in MHz
- `nvidia_clock_graphics_max_mhz{minor="..."}` - Maximum graphics clock speed in MHz
- `nvidia_clock_sm_max_mhz{minor="..."}` - Maximum SM clock speed in MHz
- `nvidia_clock_memory_max_mhz{minor="..."}` - Maximum memory clock speed in MHz

### Performance State

- `nvidia_performance_state{minor="..."}` - Current P-State (0-15, where 0 is maximum performance)

### PCIe Metrics

- `nvidia_pcie_link_generation{minor="..."}` - Current PCIe link generation (1-4+)
- `nvidia_pcie_link_width{minor="..."}` - Current PCIe link width (number of lanes)
- `nvidia_pcie_tx_throughput_kb{minor="..."}` - PCIe transmit throughput in KB/s
- `nvidia_pcie_rx_throughput_kb{minor="..."}` - PCIe receive throughput in KB/s

### Encoder/Decoder

- `nvidia_encoder_utilization{minor="..."}` - Video encoder utilization percentage (0-100)
- `nvidia_decoder_utilization{minor="..."}` - Video decoder utilization percentage (0-100)

### ECC Errors (Data Center GPUs)

- `nvidia_ecc_errors_corrected_total{minor="..."}` - Total corrected ECC errors (lifetime)
- `nvidia_ecc_errors_uncorrected_total{minor="..."}` - Total uncorrected ECC errors (lifetime)

### Process Information

- `nvidia_compute_processes{minor="..."}` - Number of compute processes currently running on the GPU
- `nvidia_graphics_processes{minor="..."}` - Number of graphics processes currently running on the GPU

### Notes

- All per-device metrics are labeled with `minor` which is the GPU's minor device number
- Metrics that are not supported by a particular GPU model will report `0`
- ECC metrics are only available on data center GPUs (Tesla, A100, H100, etc.)
- Clock speeds and some advanced metrics may not be available on all consumer GPUs
- I cannot test MIG, if anyone wants to send me a card that supports it, I can make sure it works :)

## License

MIT
