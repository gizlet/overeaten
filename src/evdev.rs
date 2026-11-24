//! The handheld's controls read straight from evdev on Gizlet OS. Taken from
//! gizlet/os's launcher (`launcher/src/evdev.rs`).
//!
//! The R36S reports its buttons and sticks as `r36s_Gamepad` and its volume
//! keys as `gpio-keys-vol`. raylib's DRM platform only knows a fixed
//! mapping, so the game reads Linux's standard gamepad codes itself:
//! BTN_EAST is A, BTN_SOUTH B, BTN_NORTH X and BTN_WEST Y. Keyboards are
//! left to raylib. Devices are rescanned every two seconds for hotplug.

use crate::input::{Button, RawPad};
use std::ffi::CStr;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};

const EV_KEY: u16 = 0x01;
const EV_ABS: u16 = 0x03;
const KEY_CNT: usize = 0x300;

const KEY_A: usize = 30;
const KEY_VOLUMEDOWN: usize = 114;
const KEY_VOLUMEUP: usize = 115;
const KEY_POWER: usize = 116;
const BTN_SOUTH: usize = 0x130;
const BTN_EAST: usize = 0x131;
const BTN_NORTH: usize = 0x133;
const BTN_WEST: usize = 0x134;
const BTN_TL: usize = 0x136;
const BTN_TR: usize = 0x137;
const BTN_TL2: usize = 0x138;
const BTN_TR2: usize = 0x139;
const BTN_SELECT: usize = 0x13a;
const BTN_START: usize = 0x13b;
const BTN_MODE: usize = 0x13c;
const BTN_THUMBL: usize = 0x13d;
const BTN_THUMBR: usize = 0x13e;
const BTN_DPAD_UP: usize = 0x220;
const BTN_DPAD_DOWN: usize = 0x221;
const BTN_DPAD_LEFT: usize = 0x222;
const BTN_DPAD_RIGHT: usize = 0x223;

const ABS_X: usize = 0x00;
const ABS_Y: usize = 0x01;
const ABS_Z: usize = 0x02;
const ABS_RX: usize = 0x03;
const ABS_RY: usize = 0x04;
const ABS_RZ: usize = 0x05;
const ABS_HAT0X: usize = 0x10;
const ABS_HAT0Y: usize = 0x11;
const ABS_CNT: usize = 0x40;

const MAP: [(usize, Button); 20] = [
    (BTN_DPAD_UP, Button::Up),
    (BTN_DPAD_DOWN, Button::Down),
    (BTN_DPAD_LEFT, Button::Left),
    (BTN_DPAD_RIGHT, Button::Right),
    (BTN_EAST, Button::A),
    (BTN_SOUTH, Button::B),
    (BTN_NORTH, Button::X),
    (BTN_WEST, Button::Y),
    (BTN_TL, Button::L1),
    (BTN_TR, Button::R1),
    (BTN_TL2, Button::L2),
    (BTN_TR2, Button::R2),
    (BTN_SELECT, Button::Select),
    (BTN_START, Button::Start),
    (BTN_MODE, Button::Home),
    (BTN_THUMBL, Button::L3),
    (BTN_THUMBR, Button::R3),
    (KEY_VOLUMEUP, Button::VolumeUp),
    (KEY_VOLUMEDOWN, Button::VolumeDown),
    (KEY_POWER, Button::Power),
];

/// `_IOC(_IOC_READ, 'E', nr, size)`.
fn eviocg(nr: u64, size: usize) -> u64 {
    (2 << 30) | ((size as u64) << 16) | ((b'E' as u64) << 8) | nr
}

fn bit(bits: &[u8], n: usize) -> bool {
    bits.get(n / 8).is_some_and(|b| b & (1 << (n % 8)) != 0)
}

#[derive(Clone, Copy, Default)]
struct Abs {
    value: i32,
    min: i32,
    max: i32,
    flat: i32,
}

impl Abs {
    /// -1..1 around the middle of the range, with the driver's dead zone.
    fn centered(self) -> f32 {
        let half = (self.max - self.min) as f32 / 2.0;
        if half <= 0.0 {
            return 0.0;
        }
        let v = self.value as f32 - (self.min as f32 + half);
        if v.abs() <= self.flat as f32 {
            0.0
        } else {
            (v / half).clamp(-1.0, 1.0)
        }
    }

    /// 0..1 from the bottom of the range.
    fn ranged(self) -> f32 {
        let span = (self.max - self.min) as f32;
        if span <= 0.0 {
            0.0
        } else {
            ((self.value - self.min) as f32 / span).clamp(0.0, 1.0)
        }
    }
}

struct Device {
    path: String,
    fd: OwnedFd,
    name: String,
    gamepad: bool,
    keys: Vec<bool>,
    abs: [Abs; ABS_CNT],
    has_abs: [bool; ABS_CNT],
}

impl Device {
    fn open(path: &str) -> Option<Device> {
        let c = std::ffi::CString::new(path).ok()?;
        let raw = unsafe { libc::open(c.as_ptr(), libc::O_RDONLY | libc::O_NONBLOCK | libc::O_CLOEXEC) };
        if raw < 0 {
            return None;
        }
        let fd = unsafe { OwnedFd::from_raw_fd(raw) };
        let mut keys = [0u8; KEY_CNT / 8];
        if unsafe { libc::ioctl(raw, eviocg(0x20 + EV_KEY as u64, keys.len()) as _, keys.as_mut_ptr()) } < 0 {
            return None;
        }
        let gamepad =
            (BTN_SOUTH..=BTN_THUMBR).any(|b| bit(&keys, b)) || (BTN_DPAD_UP..=BTN_DPAD_RIGHT).any(|b| bit(&keys, b));
        let extra = [KEY_VOLUMEUP, KEY_VOLUMEDOWN, KEY_POWER].iter().any(|&k| bit(&keys, k));
        // Keyboards are raylib's.
        if bit(&keys, KEY_A) || !(gamepad || extra) {
            return None;
        }
        let mut name = [0u8; 256];
        unsafe { libc::ioctl(raw, eviocg(0x06, name.len() - 1) as _, name.as_mut_ptr()) };
        let name = CStr::from_bytes_until_nul(&name).map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        let mut abs_bits = [0u8; ABS_CNT / 8];
        unsafe { libc::ioctl(raw, eviocg(0x20 + EV_ABS as u64, abs_bits.len()) as _, abs_bits.as_mut_ptr()) };
        let mut abs = [Abs::default(); ABS_CNT];
        let mut has_abs = [false; ABS_CNT];
        for (code, slot) in abs.iter_mut().enumerate() {
            if !bit(&abs_bits, code) {
                continue;
            }
            let mut info: libc::input_absinfo = unsafe { std::mem::zeroed() };
            let size = std::mem::size_of::<libc::input_absinfo>();
            if unsafe { libc::ioctl(raw, eviocg(0x40 + code as u64, size) as _, &mut info) } == 0 {
                *slot = Abs { value: info.value, min: info.minimum, max: info.maximum, flat: info.flat };
                has_abs[code] = true;
            }
        }
        // Buttons already held when the launcher starts.
        let mut held = [0u8; KEY_CNT / 8];
        unsafe { libc::ioctl(raw, eviocg(0x18, held.len()) as _, held.as_mut_ptr()) };
        let keys = (0..KEY_CNT).map(|k| bit(&held, k)).collect();
        Some(Device { path: path.to_string(), fd, name, gamepad, keys, abs, has_abs })
    }

    /// Applies pending events. False when the device is gone.
    fn read(&mut self) -> bool {
        let mut events: [libc::input_event; 64] = unsafe { std::mem::zeroed() };
        loop {
            let size = std::mem::size_of_val(&events);
            let n = unsafe { libc::read(self.fd.as_raw_fd(), events.as_mut_ptr().cast(), size) };
            if n < 0 {
                return std::io::Error::last_os_error().raw_os_error() == Some(libc::EAGAIN);
            }
            if n == 0 {
                return false;
            }
            let count = n as usize / std::mem::size_of::<libc::input_event>();
            for e in &events[..count] {
                let code = e.code as usize;
                match e.type_ {
                    EV_KEY if code < KEY_CNT && e.value != 2 => self.keys[code] = e.value != 0,
                    EV_ABS if code < ABS_CNT => self.abs[code].value = e.value,
                    _ => {}
                }
            }
            if count < events.len() {
                return true;
            }
        }
    }

    fn apply(&self, pad: &mut RawPad) {
        for (code, b) in MAP {
            if self.keys[code] {
                pad.set(b);
            }
        }
        if self.has_abs[ABS_HAT0X] {
            let v = self.abs[ABS_HAT0X].value;
            if v < 0 {
                pad.set(Button::Left);
            } else if v > 0 {
                pad.set(Button::Right);
            }
        }
        if self.has_abs[ABS_HAT0Y] {
            let v = self.abs[ABS_HAT0Y].value;
            if v < 0 {
                pad.set(Button::Up);
            } else if v > 0 {
                pad.set(Button::Down);
            }
        }
        for (slot, code) in [ABS_X, ABS_Y, ABS_RX, ABS_RY].into_iter().enumerate() {
            if self.has_abs[code] {
                pad.axes[slot] = self.abs[code].centered();
            }
        }
        for (slot, code) in [(4, ABS_Z), (5, ABS_RZ)] {
            if self.has_abs[code] {
                pad.axes[slot] = self.abs[code].ranged();
            }
        }
    }
}

#[derive(Default)]
pub struct Evdev {
    devices: Vec<Device>,
    scanned: Option<f64>,
}

impl Evdev {
    pub fn poll(&mut self, time: f64) {
        if self.scanned.is_none_or(|t| time - t > 2.0) {
            self.scanned = Some(time);
            self.scan();
        }
        self.devices.retain_mut(Device::read);
    }

    fn scan(&mut self) {
        let Ok(dir) = std::fs::read_dir("/dev/input") else { return };
        let mut paths: Vec<String> = dir
            .flatten()
            .map(|e| e.path().to_string_lossy().into_owned())
            .filter(|p| p.rsplit('/').next().is_some_and(|n| n.starts_with("event")))
            .collect();
        paths.sort();
        for path in paths {
            if !self.devices.iter().any(|d| d.path == path) {
                if let Some(d) = Device::open(&path) {
                    self.devices.push(d);
                }
            }
        }
    }

    /// Adds the devices' state to `pad`; true when a gamepad is among them.
    pub fn state(&self, pad: &mut RawPad) -> bool {
        for d in &self.devices {
            d.apply(pad);
            if d.gamepad && pad.name.is_none() {
                pad.name = Some(d.name.clone());
            }
        }
        self.devices.iter().any(|d| d.gamepad)
    }
}
