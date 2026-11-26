//! The machine's load, for the window's footer (ZER0's meters without its two dependencies):
//! CPU and RAM from Windows, the GPU's load and memory from NVIDIA's driver library (`nvml.dll`,
//! loaded at run time like pdfium). Without an NVIDIA driver the GPU fields are `None` and the
//! window hides those meters. Every call is a few system calls: no thread, no process.

use serde::Serialize;

#[derive(Clone, Debug, Default, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type), serde(rename = "Meters"))]
pub struct Sample {
    /// all cores, percent, since the previous sample (`None` on the first)
    pub cpu: Option<f32>,
    pub ram_used_mb: Option<u64>,
    pub ram_total_mb: Option<u64>,
    pub gpu_name: Option<String>,
    /// percent of the last sample period the GPU was busy
    pub gpu: Option<f32>,
    pub vram_used_mb: Option<u64>,
    pub vram_total_mb: Option<u64>,
}

pub struct Sampler {
    /// (idle, total) CPU time at the previous sample, in 100 ns units
    cpu_last: Option<(u64, u64)>,
    gpu: Option<nvml::Gpu>,
}

impl Default for Sampler {
    fn default() -> Self {
        Self::new()
    }
}

impl Sampler {
    pub fn new() -> Sampler {
        Sampler { cpu_last: None, gpu: nvml::Gpu::open() }
    }

    /// Keep the CPU baseline fresh without reading anything else (the window is hidden), so the
    /// first sample after it shows again covers the last few seconds, not the hidden time.
    pub fn refresh_cpu(&mut self) {
        self.cpu_last = os::cpu_times();
    }

    pub fn sample(&mut self) -> Sample {
        let mut s = Sample::default();
        if let Some((idle, total)) = os::cpu_times() {
            if let Some((idle0, total0)) = self.cpu_last {
                let dt = total.saturating_sub(total0);
                if dt > 0 {
                    let busy = dt.saturating_sub(idle.saturating_sub(idle0));
                    s.cpu = Some((busy as f64 * 100.0 / dt as f64) as f32);
                }
            }
            self.cpu_last = Some((idle, total));
        }
        if let Some((used, total)) = os::ram_mb() {
            s.ram_used_mb = Some(used);
            s.ram_total_mb = Some(total);
        }
        if let Some(g) = &self.gpu {
            s.gpu_name = g.name.clone();
            s.gpu = g.busy();
            if let Some((used, total)) = g.memory_mb() {
                s.vram_used_mb = Some(used);
                s.vram_total_mb = Some(total);
            }
        }
        s
    }
}

#[cfg(windows)]
mod os {
    use windows_sys::Win32::Foundation::FILETIME;
    use windows_sys::Win32::System::Threading::GetSystemTimes;

    /// (idle, kernel + user) since boot, all cores; kernel time includes idle time.
    pub fn cpu_times() -> Option<(u64, u64)> {
        let zero = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
        let (mut idle, mut kernel, mut user) = (zero, zero, zero);
        if unsafe { GetSystemTimes(&mut idle, &mut kernel, &mut user) } == 0 {
            return None;
        }
        let v = |t: FILETIME| ((t.dwHighDateTime as u64) << 32) | t.dwLowDateTime as u64;
        Some((v(idle), v(kernel) + v(user)))
    }

    /// (used, total) physical memory in MB.
    pub fn ram_mb() -> Option<(u64, u64)> {
        #[repr(C)]
        struct MemoryStatusEx {
            length: u32,
            memory_load: u32,
            total_phys: u64,
            avail_phys: u64,
            total_page_file: u64,
            avail_page_file: u64,
            total_virtual: u64,
            avail_virtual: u64,
            avail_extended_virtual: u64,
        }
        unsafe extern "system" {
            fn GlobalMemoryStatusEx(buffer: *mut MemoryStatusEx) -> i32;
        }
        let mut m: MemoryStatusEx = unsafe { std::mem::zeroed() };
        m.length = std::mem::size_of::<MemoryStatusEx>() as u32;
        if unsafe { GlobalMemoryStatusEx(&mut m) } == 0 {
            return None;
        }
        Some((m.total_phys.saturating_sub(m.avail_phys) >> 20, m.total_phys >> 20))
    }
}

#[cfg(not(windows))]
mod os {
    pub fn cpu_times() -> Option<(u64, u64)> {
        None
    }
    pub fn ram_mb() -> Option<(u64, u64)> {
        None
    }
}

/// NVIDIA's management library: the first GPU's load and memory. Loaded by its full path in the
/// system folder (where the driver puts it), never by a bare name the program's own folder could
/// answer.
mod nvml {
    use std::ffi::{CStr, c_void};
    use std::path::PathBuf;

    use libloading::Library;

    type Device = *mut c_void;
    /// nvmlUtilization_t: { gpu, memory } percent
    type Util = [u32; 2];
    /// nvmlMemory_t: { total, free, used } bytes
    type Mem = [u64; 3];

    pub struct Gpu {
        _lib: Library,
        device: Device,
        utilization: unsafe extern "C" fn(Device, *mut Util) -> i32,
        memory: unsafe extern "C" fn(Device, *mut Mem) -> i32,
        pub name: Option<String>,
    }

    // The device handle is an opaque token NVML accepts from any thread.
    unsafe impl Send for Gpu {}

    fn candidates() -> Vec<PathBuf> {
        let mut v = Vec::new();
        if cfg!(windows) {
            if let Some(root) = std::env::var_os("SystemRoot") {
                v.push(PathBuf::from(root).join("System32").join("nvml.dll"));
            }
            if let Some(pf) = std::env::var_os("ProgramFiles") {
                v.push(PathBuf::from(pf).join("NVIDIA Corporation").join("NVSMI").join("nvml.dll"));
            }
        } else {
            v.push(PathBuf::from("libnvidia-ml.so.1"));
        }
        v
    }

    impl Gpu {
        pub fn open() -> Option<Gpu> {
            let lib = candidates().into_iter().find_map(|p| unsafe { Library::new(&p) }.ok())?;
            unsafe {
                let init = *lib.get::<unsafe extern "C" fn() -> i32>(b"nvmlInit_v2\0").ok()?;
                if init() != 0 {
                    return None;
                }
                let by_index = *lib
                    .get::<unsafe extern "C" fn(u32, *mut Device) -> i32>(b"nvmlDeviceGetHandleByIndex_v2\0")
                    .ok()?;
                let mut device: Device = std::ptr::null_mut();
                if by_index(0, &mut device) != 0 || device.is_null() {
                    return None;
                }
                let utilization: unsafe extern "C" fn(Device, *mut Util) -> i32 =
                    *lib.get(b"nvmlDeviceGetUtilizationRates\0").ok()?;
                let memory: unsafe extern "C" fn(Device, *mut Mem) -> i32 =
                    *lib.get(b"nvmlDeviceGetMemoryInfo\0").ok()?;
                let name = lib
                    .get::<unsafe extern "C" fn(Device, *mut u8, u32) -> i32>(b"nvmlDeviceGetName\0")
                    .ok()
                    .and_then(|get| {
                        let mut buf = [0u8; 96];
                        if get(device, buf.as_mut_ptr(), buf.len() as u32) != 0 {
                            return None;
                        }
                        CStr::from_bytes_until_nul(&buf).ok().map(|c| c.to_string_lossy().into_owned())
                    });
                Some(Gpu { _lib: lib, device, utilization, memory, name })
            }
        }

        pub fn busy(&self) -> Option<f32> {
            let mut u: Util = [0; 2];
            (unsafe { (self.utilization)(self.device, &mut u) } == 0).then_some(u[0] as f32)
        }

        /// (used, total) in MB.
        pub fn memory_mb(&self) -> Option<(u64, u64)> {
            let mut m: Mem = [0; 3];
            (unsafe { (self.memory)(self.device, &mut m) } == 0).then_some((m[2] >> 20, m[0] >> 20))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_samples_give_cpu_and_ram() {
        let mut s = Sampler::new();
        let first = s.sample();
        assert!(first.cpu.is_none(), "no previous sample yet");
        std::thread::sleep(std::time::Duration::from_millis(120));
        let second = s.sample();
        eprintln!("{second:?}");
        if cfg!(windows) {
            let cpu = second.cpu.expect("CPU after two samples");
            assert!((0.0..=100.0).contains(&cpu));
            let (used, total) = (second.ram_used_mb.unwrap(), second.ram_total_mb.unwrap());
            assert!(total > 0 && used <= total);
        }
        if let (Some(u), Some(t)) = (second.vram_used_mb, second.vram_total_mb) {
            assert!(t > 0 && u <= t);
        }
    }
}
