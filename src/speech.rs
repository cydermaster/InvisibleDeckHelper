//! Speech output: Piper TTS (piper1-gpl) plus WAV playback.
//!
//! Piper runs as a child process. The Flatpak bundles the `piper-tts` wheel and
//! the `en_GB-alan-medium` voice, so the text never leaves the machine.
//!
//! Playback avoids a build-time audio dependency: we `dlopen` libpulse-simple
//! at runtime and fall back to whichever common command-line player exists.
//! If none is available the app still works, it just doesn't speak.

use std::ffi::CString;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Where the piper entry point and voice live inside the Flatpak (or a source
/// checkout). Overridable so the app can be run outside the sandbox.
const APP_ID: &str = "org.cydermaster.InvisibleDeckHelper";
const VOICE_FILE: &str = "en_GB-alan-medium.onnx";

fn candidate_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(p) = std::env::var("IDH_DATA_DIR") {
        roots.push(PathBuf::from(p));
    }
    // Flatpak install location.
    roots.push(PathBuf::from(format!("/app/share/{APP_ID}")));
    // Running from a source tree / local build.
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            roots.push(dir.to_path_buf());
            roots.push(dir.join("../data"));
        }
    }
    roots.push(PathBuf::from("data"));
    roots
}

/// Path to the voice model, if we can find one.
pub fn voice_path() -> Option<PathBuf> {
    for root in candidate_roots() {
        for candidate in [root.join(VOICE_FILE), root.join("voices").join(VOICE_FILE)] {
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

/// How piper should be invoked (kept out of `Speaker` so it can be cloned freely).
fn piper_command() -> Option<Vec<String>> {
    if let Ok(explicit) = std::env::var("IDH_PIPER") {
        return Some(vec![explicit]);
    }
    // Flatpak: the console script installed alongside the wheel.
    if Path::new("/app/bin/piper").is_file() {
        return Some(vec!["/app/bin/piper".to_string()]);
    }
    // Flatpak: module invocation, useful if the script is missing.
    if Path::new("/app/lib/python3/site-packages/piper").is_dir() {
        return Some(vec![
            "python3".to_string(),
            "-m".to_string(),
            "piper".to_string(),
        ]);
    }
    // Source tree / host.
    let local = PathBuf::from("piper-test/.venv/bin/piper");
    if local.is_file() {
        return Some(vec![local.to_string_lossy().to_string()]);
    }
    None
}

#[derive(Clone)]
pub struct Speaker {
    voice: PathBuf,
    piper: Vec<String>,
}

impl Speaker {
    /// Returns None when piper or the voice is unavailable — callers treat that
    /// as "silent mode" rather than an error.
    pub fn detect() -> Option<Speaker> {
        let voice = voice_path()?;
        let piper = piper_command()?;
        Some(Speaker { voice, piper })
    }

    pub fn describe(&self) -> String {
        format!(
            "{} via {}",
            self.voice.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default(),
            self.piper.first().cloned().unwrap_or_default()
        )
    }

    /// Synthesise `text` to a temporary WAV and play it.
    pub fn say(&self, text: &str) -> Result<(), String> {
        // Unique per call: speech runs on a thread and calls can overlap.
        static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let n = SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let wav_path =
            std::env::temp_dir().join(format!("idh-{}-{}.wav", std::process::id(), n));
        self.synthesise(text, &wav_path)?;
        let played = play_wav(&wav_path);
        let _ = std::fs::remove_file(&wav_path);
        played
    }

    pub fn synthesise(&self, text: &str, out: &Path) -> Result<(), String> {
        let mut cmd = Command::new(&self.piper[0]);
        for arg in &self.piper[1..] {
            cmd.arg(arg);
        }
        cmd.arg("-m")
            .arg(&self.voice)
            .arg("-f")
            .arg(out)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| format!("could not run piper: {e}"))?;
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(text.as_bytes())
                .map_err(|e| format!("could not send text to piper: {e}"))?;
        }
        let output = child
            .wait_with_output()
            .map_err(|e| format!("piper failed: {e}"))?;
        if !output.status.success() {
            // Piper writes an onnxruntime affinity warning to stderr even on
            // success, so only surface it when the exit status is bad.
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(format!("piper exited {}: {}", output.status, err.trim()));
        }
        if !out.is_file() {
            return Err("piper produced no audio file".to_string());
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Playback: dlopen libpulse-simple, else fall back to CLI players.
// ---------------------------------------------------------------------------

mod pulse {
    use std::ffi::{c_char, c_int, c_void, CString};

    pub type PaSimple = c_void;

    /// PA_SAMPLE_S16LE
    pub const PA_SAMPLE_S16LE: c_int = 3;
    /// PA_STREAM_PLAYBACK
    pub const PA_STREAM_PLAYBACK: c_int = 1;

    #[repr(C)]
    pub struct SampleSpec {
        pub format: c_int,
        pub rate: u32,
        pub channels: u8,
    }

    pub type NewFn = unsafe extern "C" fn(
        *const c_char,      // server
        *const c_char,      // name
        c_int,              // dir
        *const c_char,      // dev
        *const c_char,      // stream_name
        *const SampleSpec,  // ss
        *const c_void,      // map
        *const c_void,      // attr
        *mut c_int,         // error
    ) -> *mut PaSimple;
    pub type WriteFn = unsafe extern "C" fn(*mut PaSimple, *const c_void, usize, *mut c_int) -> c_int;
    pub type DrainFn = unsafe extern "C" fn(*mut PaSimple, *mut c_int) -> c_int;
    pub type FreeFn = unsafe extern "C" fn(*mut PaSimple);

    pub struct Lib {
        #[allow(dead_code)] // held so the library stays loaded for the fn pointers
        pub handle: *mut c_void,
        pub new: NewFn,
        pub write: WriteFn,
        pub drain: DrainFn,
        pub free: FreeFn,
    }

    unsafe impl Send for Lib {}
    unsafe impl Sync for Lib {}

    extern "C" {
        fn dlopen(filename: *const c_char, flag: c_int) -> *mut c_void;
        fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
        fn dlclose(handle: *mut c_void) -> c_int;
    }
    const RTLD_NOW: c_int = 2;

    impl Lib {
        pub fn open() -> Option<Lib> {
            for soname in ["libpulse-simple.so.0", "libpulse-simple.so"] {
                let cname = CString::new(soname).ok()?;
                let handle = unsafe { dlopen(cname.as_ptr(), RTLD_NOW) };
                if handle.is_null() {
                    continue;
                }
                let sym = |name: &str| -> Option<*mut c_void> {
                    let c = CString::new(name).ok()?;
                    let p = unsafe { dlsym(handle, c.as_ptr()) };
                    if p.is_null() {
                        None
                    } else {
                        Some(p)
                    }
                };
                let (Some(new), Some(write), Some(drain), Some(free)) = (
                    sym("pa_simple_new"),
                    sym("pa_simple_write"),
                    sym("pa_simple_drain"),
                    sym("pa_simple_free"),
                ) else {
                    unsafe { dlclose(handle) };
                    continue;
                };
                return Some(Lib {
                    handle,
                    new: unsafe { std::mem::transmute::<*mut c_void, NewFn>(new) },
                    write: unsafe { std::mem::transmute::<*mut c_void, WriteFn>(write) },
                    drain: unsafe { std::mem::transmute::<*mut c_void, DrainFn>(drain) },
                    free: unsafe { std::mem::transmute::<*mut c_void, FreeFn>(free) },
                });
            }
            None
        }
    }
}

/// Minimal RIFF/WAVE (16-bit PCM) reader — enough to pull out what libpulse needs.
fn parse_wav(bytes: &[u8]) -> Result<(u32, u8, Vec<u8>), String> {
    if bytes.len() < 12 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err("not a RIFF/WAVE file".to_string());
    }
    let mut pos = 12usize;
    let mut rate = 0u32;
    let mut channels = 1u8;
    let mut bits = 16u16;
    while pos + 8 <= bytes.len() {
        let id = &bytes[pos..pos + 4];
        let size = u32::from_le_bytes([
            bytes[pos + 4],
            bytes[pos + 5],
            bytes[pos + 6],
            bytes[pos + 7],
        ]) as usize;
        let body_start = pos + 8;
        let body_end = (body_start + size).min(bytes.len());
        match id {
            b"fmt " => {
                if body_end - body_start >= 16 {
                    channels = u16::from_le_bytes([bytes[body_start + 2], bytes[body_start + 3]]) as u8;
                    rate = u32::from_le_bytes([
                        bytes[body_start + 4],
                        bytes[body_start + 5],
                        bytes[body_start + 6],
                        bytes[body_start + 7],
                    ]);
                    bits = u16::from_le_bytes([bytes[body_start + 14], bytes[body_start + 15]]);
                }
            }
            b"data" => {
                if bits != 16 {
                    return Err(format!("unsupported bit depth {bits}"));
                }
                return Ok((rate, channels, bytes[body_start..body_end].to_vec()));
            }
            _ => {}
        }
        pos = body_start + size + (size & 1);
    }
    Err("no data chunk".to_string())
}

fn play_with_pulse(path: &Path) -> Result<(), String> {
    let lib = pulse::Lib::open().ok_or("libpulse-simple not available")?;
    let bytes = std::fs::read(path).map_err(|e| format!("read wav: {e}"))?;
    let (rate, channels, pcm) = parse_wav(&bytes)?;

    let name = CString::new("Invisible Deck Helper").map_err(|e| e.to_string())?;
    let stream = CString::new("Speech").map_err(|e| e.to_string())?;
    let spec = pulse::SampleSpec {
        format: pulse::PA_SAMPLE_S16LE,
        rate,
        channels,
    };
    let mut err: std::ffi::c_int = 0;

    unsafe {
        let s = (lib.new)(
            std::ptr::null(),
            name.as_ptr(),
            pulse::PA_STREAM_PLAYBACK,
            std::ptr::null(),
            stream.as_ptr(),
            &spec,
            std::ptr::null(),
            std::ptr::null(),
            &mut err,
        );
        if s.is_null() {
            return Err(format!("pa_simple_new failed (error {err})"));
        }
        let rc = (lib.write)(s, pcm.as_ptr() as *const _, pcm.len(), &mut err);
        if rc < 0 {
            (lib.free)(s);
            return Err(format!("pa_simple_write failed (error {err})"));
        }
        (lib.drain)(s, &mut err);
        (lib.free)(s);
    }
    Ok(())
}

fn play_with_cli(path: &Path) -> Result<(), String> {
    // Order matters: PulseAudio/PipeWire first, then ALSA, then sox/ffmpeg.
    for (prog, args) in [
        ("paplay", vec![]),
        ("pw-play", vec![]),
        ("aplay", vec!["-q"]),
        ("ffplay", vec!["-nodisp", "-autoexit", "-loglevel", "quiet"]),
        ("play", vec!["-q"]),
    ] {
        let mut cmd = Command::new(prog);
        cmd.args(&args).arg(path);
        cmd.stdout(Stdio::null()).stderr(Stdio::null());
        if cmd.status().map(|s| s.success()).unwrap_or(false) {
            return Ok(());
        }
    }
    Err("no audio player available (looked for paplay, pw-play, aplay, ffplay, play)".to_string())
}

pub fn play_wav(path: &Path) -> Result<(), String> {
    match play_with_pulse(path) {
        Ok(()) => Ok(()),
        Err(pulse_err) => play_with_cli(path).map_err(|cli_err| format!("{pulse_err}; {cli_err}")),
    }
}
