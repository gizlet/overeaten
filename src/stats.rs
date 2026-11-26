//! System numbers for the stats panel (from gizlet/little-prince): CPU load of the whole machine and of this
//! process, free and total memory, this process's resident memory, the SoC
//! temperature and CPU clock. Sampled twice a second.
//!
//! Linux (Gizlet OS) reads /proc and /sys; macOS asks Mach; elsewhere the
//! fields stay empty.

#[derive(Clone, Copy, Default, Debug)]
pub struct Sample {
    /// Whole machine, 0..100, across all cores.
    pub cpu: Option<f32>,
    /// This process, 0..100 of the whole machine.
    pub cpu_self: Option<f32>,
    pub cores: u32,
    pub mem_free_mb: Option<f32>,
    pub mem_total_mb: Option<f32>,
    pub mem_self_mb: Option<f32>,
    pub temp_c: Option<f32>,
    pub mhz: Option<f32>,
}

#[derive(Default)]
pub struct Stats {
    pub now: Sample,
    since: f32,
    last: Option<Ticks>,
    /// Frame times in milliseconds, newest last.
    pub frames: Vec<f32>,
}

#[derive(Clone, Copy, Debug)]
struct Ticks {
    busy: f64,
    total: f64,
    own: f64,
}

impl Stats {
    pub fn new() -> Stats {
        let mut s = Stats { since: f32::MAX, ..Default::default() };
        s.update(0.0);
        s
    }

    pub fn update(&mut self, dt: f32) {
        if dt > 0.0 {
            self.frames.push(dt * 1000.0);
            if self.frames.len() > 90 {
                self.frames.remove(0);
            }
        }
        self.since += dt;
        if self.since < 0.5 {
            return;
        }
        self.since = 0.0;
        let t = ticks();
        match (self.last, t) {
            (Some(a), Some(b)) => {
                let total = b.total - a.total;
                // macOS updates its tick counters coarsely: when they have not
                // moved yet, keep the old starting point for the next sample.
                if total > 0.0 {
                    self.now.cpu = Some(((b.busy - a.busy) / total * 100.0).clamp(0.0, 100.0) as f32);
                    self.now.cpu_self = Some(((b.own - a.own) / total * 100.0).clamp(0.0, 100.0) as f32);
                    self.last = t;
                }
            }
            _ => self.last = t,
        }
        let cpu = (self.now.cpu, self.now.cpu_self);
        let frames = std::mem::take(&mut self.frames);
        self.now = sample();
        (self.now.cpu, self.now.cpu_self) = cpu;
        self.frames = frames;
    }

    pub fn frame_ms(&self) -> f32 {
        let n = self.frames.len().min(30);
        if n == 0 {
            return 0.0;
        }
        self.frames[self.frames.len() - n..].iter().sum::<f32>() / n as f32
    }
}

fn cores() -> u32 {
    std::thread::available_parallelism().map_or(1, |n| n.get() as u32)
}

#[cfg(target_os = "linux")]
fn ticks() -> Option<Ticks> {
    // Both files count in clock ticks, so they compare directly.
    let stat = std::fs::read_to_string("/proc/stat").ok()?;
    let f: Vec<f64> = stat.lines().next()?.split_whitespace().skip(1).filter_map(|v| v.parse().ok()).collect();
    if f.len() < 4 {
        return None;
    }
    let total: f64 = f.iter().take(8).sum();
    let idle = f[3] + f.get(4).copied().unwrap_or(0.0);
    let me = std::fs::read_to_string("/proc/self/stat").ok()?;
    // Fields after the command name, which may contain spaces, end at ')'.
    let rest: Vec<&str> = me.rsplit_once(')')?.1.split_whitespace().collect();
    let own = rest.get(11)?.parse::<f64>().ok()? + rest.get(12)?.parse::<f64>().ok()?;
    Some(Ticks { busy: total - idle, total, own })
}

#[cfg(target_os = "linux")]
fn sample() -> Sample {
    let mut s = Sample { cores: cores(), ..Default::default() };
    if let Ok(m) = std::fs::read_to_string("/proc/meminfo") {
        let kb = |key: &str| {
            m.lines().find(|l| l.starts_with(key)).and_then(|l| l.split_whitespace().nth(1)).and_then(|v| v.parse::<f32>().ok())
        };
        s.mem_total_mb = kb("MemTotal:").map(|v| v / 1024.0);
        s.mem_free_mb = kb("MemAvailable:").map(|v| v / 1024.0);
    }
    if let Ok(m) = std::fs::read_to_string("/proc/self/statm") {
        let page = unsafe { libc::sysconf(libc::_SC_PAGESIZE) } as f32;
        s.mem_self_mb = m.split_whitespace().nth(1).and_then(|v| v.parse::<f32>().ok()).map(|p| p * page / 1048576.0);
    }
    let read = |p: &str| std::fs::read_to_string(p).ok().and_then(|v| v.trim().parse::<f32>().ok());
    s.temp_c = read("/sys/class/thermal/thermal_zone0/temp").map(|v| v / 1000.0);
    s.mhz = read("/sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq").map(|v| v / 1000.0);
    s
}

#[cfg(target_os = "macos")]
#[allow(deprecated)]
fn ticks() -> Option<Ticks> {
    unsafe {
        let mut info: libc::host_cpu_load_info = std::mem::zeroed();
        let mut count = (std::mem::size_of::<libc::host_cpu_load_info>() / 4) as libc::mach_msg_type_number_t;
        let r = libc::host_statistics(libc::mach_host_self(), libc::HOST_CPU_LOAD_INFO, &mut info as *mut _ as _, &mut count);
        if r != 0 {
            return None;
        }
        let t: Vec<f64> = info.cpu_ticks.iter().map(|&v| v as f64).collect();
        let total: f64 = t.iter().sum();
        let idle = t[libc::CPU_STATE_IDLE as usize];
        // Mach counts in 100 Hz ticks summed over cores; rusage in microseconds.
        let mut ru: libc::rusage = std::mem::zeroed();
        libc::getrusage(libc::RUSAGE_SELF, &mut ru);
        let us = |tv: libc::timeval| tv.tv_sec as f64 * 1e6 + tv.tv_usec as f64;
        let own = (us(ru.ru_utime) + us(ru.ru_stime)) / 1e4;
        Some(Ticks { busy: total - idle, total, own })
    }
}

#[cfg(target_os = "macos")]
#[allow(deprecated)]
fn sample() -> Sample {
    let mut s = Sample { cores: cores(), ..Default::default() };
    unsafe {
        let page = libc::sysconf(libc::_SC_PAGESIZE) as f64;
        let mut vm: libc::vm_statistics64 = std::mem::zeroed();
        let mut count = libc::HOST_VM_INFO64_COUNT;
        if libc::host_statistics64(libc::mach_host_self(), libc::HOST_VM_INFO64, &mut vm as *mut _ as _, &mut count) == 0 {
            let free = (vm.free_count as f64 + vm.inactive_count as f64 + vm.purgeable_count as f64) * page;
            s.mem_free_mb = Some((free / 1048576.0) as f32);
        }
        let mut total: u64 = 0;
        let mut len = std::mem::size_of::<u64>();
        if libc::sysctlbyname(c"hw.memsize".as_ptr(), &mut total as *mut u64 as _, &mut len, std::ptr::null_mut(), 0) == 0 {
            s.mem_total_mb = Some((total as f64 / 1048576.0) as f32);
        }
        let mut info: libc::mach_task_basic_info = std::mem::zeroed();
        let mut count = (std::mem::size_of::<libc::mach_task_basic_info>() / 4) as libc::mach_msg_type_number_t;
        if libc::task_info(libc::mach_task_self(), libc::MACH_TASK_BASIC_INFO, &mut info as *mut _ as _, &mut count) == 0 {
            s.mem_self_mb = Some((info.resident_size as f64 / 1048576.0) as f32);
        }
    }
    s
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn ticks() -> Option<Ticks> {
    None
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn sample() -> Sample {
    Sample { cores: cores(), ..Default::default() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn samples_this_machine() {
        let mut s = Stats::new();
        // Burn a little CPU between samples so there is something to measure.
        for _ in 0..5 {
            let t = std::time::Instant::now();
            let mut x = 0u64;
            while t.elapsed().as_millis() < 150 {
                x = x.wrapping_mul(31).wrapping_add(7);
            }
            std::hint::black_box(x);
            s.update(0.6);
            if s.now.cpu.is_some() {
                break;
            }
        }
        if cfg!(any(target_os = "linux", target_os = "macos")) {
            let n = s.now;
            // macOS tick counters can stall for a while, so a sample may be
            // missing there; when there is one it must be a percentage.
            assert!(n.cpu.is_none_or(|v| (0.0..=100.0).contains(&v)), "{n:?}");
            if cfg!(target_os = "linux") {
                assert!(n.cpu.is_some(), "{n:?}");
            }
            assert!(n.mem_total_mb.is_some_and(|v| v > 0.0) && n.mem_free_mb.is_some(), "{n:?}");
            assert!(n.mem_self_mb.is_some_and(|v| v > 0.0), "{n:?}");
        }
    }
}

