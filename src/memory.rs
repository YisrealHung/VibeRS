use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug)]
pub struct MemoryStats {
    /// Physical RAM currently mapped in Working Set (MB)
    pub working_set_mb: f32,
    /// Private committed memory of the process (MB)
    pub private_mb: f32,
}

pub struct MemoryMonitor {
    last_check: Instant,
    pub stats: MemoryStats,
}

impl Default for MemoryMonitor {
    fn default() -> Self {
        let stats = query_process_memory();
        Self {
            last_check: Instant::now(),
            stats,
        }
    }
}

impl MemoryMonitor {
    pub fn update_if_needed(&mut self) {
        if self.last_check.elapsed() >= Duration::from_millis(800) {
            self.stats = query_process_memory();
            self.last_check = Instant::now();
        }
    }

    pub fn force_refresh(&mut self) {
        self.stats = query_process_memory();
        self.last_check = Instant::now();
    }
}

#[cfg(target_os = "windows")]
mod win_mem {
    use super::MemoryStats;
    use std::ffi::c_void;

    #[repr(C)]
    #[allow(non_snake_case)]
    struct PROCESS_MEMORY_COUNTERS_EX {
        cb: u32,
        PageFaultCount: u32,
        PeakWorkingSetSize: usize,
        WorkingSetSize: usize,
        QuotaPeakPagedPoolUsage: usize,
        QuotaPagedPoolUsage: usize,
        QuotaPeakNonPagedPoolUsage: usize,
        QuotaNonPagedPoolUsage: usize,
        PagefileUsage: usize,
        PeakPagefileUsage: usize,
        PrivateUsage: usize,
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn GetCurrentProcess() -> *mut c_void;
        fn K32GetProcessMemoryInfo(
            process: *mut c_void,
            ppsmemcounters: *mut PROCESS_MEMORY_COUNTERS_EX,
            cb: u32,
        ) -> i32;
        fn SetProcessWorkingSetSize(
            hprocess: *mut c_void,
            dwminimumworkingsetsize: usize,
            dwmaximumworkingsetsize: usize,
        ) -> i32;
    }

    pub fn query() -> MemoryStats {
        unsafe {
            let mut counters: PROCESS_MEMORY_COUNTERS_EX = std::mem::zeroed();
            counters.cb = std::mem::size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32;
            let handle = GetCurrentProcess();
            let ok = K32GetProcessMemoryInfo(handle, &mut counters, counters.cb);
            if ok != 0 {
                MemoryStats {
                    working_set_mb: counters.WorkingSetSize as f32 / (1024.0 * 1024.0),
                    private_mb: counters.PrivateUsage as f32 / (1024.0 * 1024.0),
                }
            } else {
                MemoryStats {
                    working_set_mb: 0.0,
                    private_mb: 0.0,
                }
            }
        }
    }

    pub fn trim_working_set() {
        unsafe {
            let handle = GetCurrentProcess();
            // usize::MAX (-1) tells Windows to trim unused pages from the working set
            let _ = SetProcessWorkingSetSize(handle, usize::MAX, usize::MAX);
        }
    }
}

#[cfg(target_os = "windows")]
pub fn query_process_memory() -> MemoryStats {
    win_mem::query()
}

#[cfg(target_os = "windows")]
pub fn trim_working_set() {
    win_mem::trim_working_set();
}

#[cfg(not(target_os = "windows"))]
pub fn query_process_memory() -> MemoryStats {
    MemoryStats {
        working_set_mb: 32.0,
        private_mb: 28.0,
    }
}

#[cfg(not(target_os = "windows"))]
pub fn trim_working_set() {}

