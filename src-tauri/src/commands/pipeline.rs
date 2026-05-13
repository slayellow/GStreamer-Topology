use std::env;
use std::fs;
use std::io::Read;
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

use base64::{engine::general_purpose, Engine as _};
use ssh2::Session;
use tauri::State;

use crate::models::{
    ElementMetadataResponse, ElementPadTemplateMetadata, ElementPropertyMetadata,
    GStreamerProbeResponse, MetadataAuthority, PipelineDocument, PipelineSimulationResponse,
    PlaybackMediaKind, PlaybackPrepareResponse, PlaybackProcessState, PlaybackProtocol,
    PlaybackStatusResponse, PlaybackStream, RemoteProbeResponse, RemoteTargetRequest, SourceKind,
};
use crate::parser::{normalize_text, parse_document};

#[derive(Default)]
pub struct PlaybackState {
    session: Mutex<Option<PlaybackSession>>,
}

struct PlaybackSession {
    child: Child,
    command: String,
    pid: u32,
}

impl Drop for PlaybackSession {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[tauri::command]
pub fn parse_pipeline_text(raw_text: String, source_name: Option<String>) -> PipelineDocument {
    let normalization = normalize_text(&raw_text);
    parse_document(
        raw_text,
        normalization.normalized_text,
        SourceKind::PastedText,
        None,
        source_name,
        normalization.diagnostics,
    )
}

#[tauri::command]
pub fn load_local_pipeline_file(path: String) -> Result<PipelineDocument, String> {
    let source_path = Path::new(&path);
    let raw_bytes = fs::read(source_path)
        .map_err(|error| format!("failed to read `{}`: {error}", source_path.display()))?;
    let raw_text = String::from_utf8_lossy(&raw_bytes).into_owned();
    let normalization = normalize_text(&raw_text);
    let source_name = source_path
        .file_name()
        .map(|value| value.to_string_lossy().into_owned());

    Ok(parse_document(
        raw_text,
        normalization.normalized_text,
        SourceKind::LocalFile,
        Some(path),
        source_name,
        normalization.diagnostics,
    ))
}

#[tauri::command]
pub fn save_export_file(path: String, contents: String) -> Result<Option<String>, String> {
    let path = path.trim();
    if path.is_empty() {
        return Ok(None);
    }

    write_export_payload(Path::new(path), &contents)?;
    Ok(Some(path.to_string()))
}

#[tauri::command]
pub fn save_export_file_to_downloads(
    file_name: String,
    contents: String,
) -> Result<String, String> {
    let export_dir = default_export_dir();
    fs::create_dir_all(&export_dir).map_err(|error| {
        format!(
            "failed to create export folder `{}`: {error}",
            export_dir.display()
        )
    })?;
    let target_path = next_available_export_path(&export_dir, &safe_export_file_name(&file_name)?);

    write_export_payload(&target_path, &contents)?;
    Ok(target_path.to_string_lossy().into_owned())
}

#[tauri::command]
pub fn suggest_export_file_path(file_name: String) -> Result<String, String> {
    let export_dir = default_export_dir();
    fs::create_dir_all(&export_dir).map_err(|error| {
        format!(
            "failed to create export folder `{}`: {error}",
            export_dir.display()
        )
    })?;
    let file_name = safe_export_file_name(&file_name)?;
    Ok(next_available_export_path(&export_dir, &file_name)
        .to_string_lossy()
        .into_owned())
}

fn write_export_payload(path: &Path, contents: &str) -> Result<(), String> {
    let bytes = general_purpose::STANDARD
        .decode(contents)
        .map_err(|error| format!("failed to decode export payload: {error}"))?;
    if path.exists() && path.is_dir() {
        return Err(format!("export path is a directory: {}", path.display()));
    }

    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            return Err(format!(
                "export folder does not exist: {}",
                parent.display()
            ));
        }
    }

    fs::write(path, bytes)
        .map_err(|error| format!("failed to save export `{}`: {error}", path.display()))?;

    Ok(())
}

fn default_export_dir() -> PathBuf {
    home_dir()
        .map(|path| path.join("Downloads").join("GStreamer Topology Exports"))
        .unwrap_or_else(|| {
            env::current_dir()
                .unwrap_or_else(|_| env::temp_dir())
                .join("GStreamer Topology Exports")
        })
}

fn home_dir() -> Option<PathBuf> {
    env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

fn safe_export_file_name(file_name: &str) -> Result<String, String> {
    let leaf_name = file_name
        .trim()
        .rsplit(|character| character == '/' || character == '\\')
        .next()
        .unwrap_or("")
        .trim();

    if leaf_name.is_empty() {
        return Err("export file name is empty.".to_string());
    }

    let sanitized = leaf_name
        .chars()
        .map(|character| {
            if matches!(
                character,
                '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*'
            ) {
                '-'
            } else {
                character
            }
        })
        .collect::<String>()
        .trim_matches(|character| character == '.' || character == ' ')
        .to_string();

    if sanitized.is_empty() {
        Err("export file name does not contain usable characters.".to_string())
    } else {
        Ok(sanitized)
    }
}

fn next_available_export_path(folder: &Path, file_name: &str) -> PathBuf {
    let initial_path = folder.join(file_name);
    if !initial_path.exists() {
        return initial_path;
    }

    let stem = Path::new(file_name)
        .file_stem()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("gstreamer-topology");
    let extension = Path::new(file_name)
        .extension()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty());

    for index in 1..1000 {
        let candidate = match extension {
            Some(extension) => folder.join(format!("{stem}-{index}.{extension}")),
            None => folder.join(format!("{stem}-{index}")),
        };

        if !candidate.exists() {
            return candidate;
        }
    }

    folder.join(format!("{stem}-{}", chrono_like_timestamp()))
}

fn chrono_like_timestamp() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "now".to_string())
}

fn gst_inspect_executable_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "gst-inspect-1.0.exe"
    } else {
        "gst-inspect-1.0"
    }
}

fn gst_launch_executable_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "gst-launch-1.0.exe"
    } else {
        "gst-launch-1.0"
    }
}

fn hidden_command(program: &Path) -> Command {
    let mut command = Command::new(program);
    hide_console_window(&mut command);
    command
}

#[cfg(target_os = "windows")]
fn hide_console_window(command: &mut Command) {
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x08000000;
    command.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(target_os = "windows"))]
fn hide_console_window(_command: &mut Command) {}

fn push_unique_candidate(candidates: &mut Vec<PathBuf>, path: PathBuf) {
    if !candidates.iter().any(|candidate| candidate == &path) {
        candidates.push(path);
    }
}

fn gst_inspect_command_candidates() -> Vec<PathBuf> {
    gstreamer_command_candidates(gst_inspect_executable_name())
}

fn gst_launch_command_candidates() -> Vec<PathBuf> {
    gstreamer_command_candidates(gst_launch_executable_name())
}

fn gstreamer_command_candidates(executable_name: &str) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    push_unique_candidate(&mut candidates, PathBuf::from(executable_name));

    if let Some(path_value) = env::var_os("PATH") {
        for folder in env::split_paths(&path_value) {
            push_unique_candidate(&mut candidates, folder.join(executable_name));
        }
    }

    for variable in [
        "GSTREAMER_1_0_ROOT_X86_64",
        "GSTREAMER_1_0_ROOT_MSVC_X86_64",
        "GSTREAMER_ROOT_X86_64",
        "GSTREAMER_DIR",
    ] {
        if let Some(root) = env::var_os(variable) {
            let root = PathBuf::from(root);
            push_unique_candidate(&mut candidates, root.join("bin").join(executable_name));
            push_unique_candidate(&mut candidates, root.join(executable_name));
        }
    }

    if let Some(home) = home_dir() {
        for folder in ["anaconda3/bin", "miniconda3/bin", "mambaforge/bin"] {
            push_unique_candidate(&mut candidates, home.join(folder).join(executable_name));
        }
    }

    if cfg!(target_os = "macos") {
        for folder in [
            "/opt/homebrew/bin",
            "/usr/local/bin",
            "/Library/Frameworks/GStreamer.framework/Versions/Current/bin",
            "/Library/Frameworks/GStreamer.framework/Versions/1.0/bin",
        ] {
            push_unique_candidate(&mut candidates, Path::new(folder).join(executable_name));
        }
    } else if cfg!(target_os = "windows") {
        for folder in [
            r"C:\gstreamer\1.0\msvc_x86_64\bin",
            r"C:\gstreamer\1.0\mingw_x86_64\bin",
            r"C:\Program Files\gstreamer\1.0\msvc_x86_64\bin",
            r"C:\Program Files\gstreamer\1.0\mingw_x86_64\bin",
        ] {
            push_unique_candidate(&mut candidates, Path::new(folder).join(executable_name));
        }
    } else {
        for folder in ["/usr/bin", "/usr/local/bin"] {
            push_unique_candidate(&mut candidates, Path::new(folder).join(executable_name));
        }
    }

    candidates
}

fn resolve_gst_inspect_command() -> Result<PathBuf, String> {
    let candidates = gst_inspect_command_candidates();
    let mut diagnostics = Vec::new();

    for candidate in &candidates {
        match hidden_command(candidate).arg("--version").output() {
            Ok(output) if output.status.success() => return Ok(candidate.clone()),
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr)
                    .trim()
                    .to_string()
                    .if_empty("version command failed.");
                diagnostics.push(format!("{}: {stderr}", candidate.display()));
            }
            Err(error) => diagnostics.push(format!("{}: {error}", candidate.display())),
        }
    }

    Err(format!(
        "gst-inspect-1.0 is not available. Checked {} candidate path(s): {}",
        candidates.len(),
        diagnostics.join("; ")
    ))
}

fn resolve_gst_launch_command() -> Result<PathBuf, String> {
    let candidates = gst_launch_command_candidates();
    let mut diagnostics = Vec::new();

    for candidate in &candidates {
        match hidden_command(candidate).arg("--version").output() {
            Ok(output) if output.status.success() => return Ok(candidate.clone()),
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr)
                    .trim()
                    .to_string()
                    .if_empty("version command failed.");
                diagnostics.push(format!("{}: {stderr}", candidate.display()));
            }
            Err(error) => diagnostics.push(format!("{}: {error}", candidate.display())),
        }
    }

    Err(format!(
        "gst-launch-1.0 is not available. Checked {} candidate path(s): {}",
        candidates.len(),
        diagnostics.join("; ")
    ))
}

fn run_gst_inspect(args: &[&str]) -> Result<(PathBuf, Output), String> {
    let command_path = resolve_gst_inspect_command()?;
    let output = hidden_command(&command_path)
        .args(args)
        .output()
        .map_err(|error| {
            format!(
                "failed to run `{}` with args `{}`: {error}",
                command_path.display(),
                args.join(" ")
            )
        })?;

    Ok((command_path, output))
}

fn split_pipeline_arguments(pipeline: &str) -> Result<Vec<String>, String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut chars = pipeline.chars().peekable();
    let mut quote: Option<char> = None;
    let mut escaped = false;

    while let Some(character) = chars.next() {
        if escaped {
            current.push(character);
            escaped = false;
            continue;
        }

        if character == '\\' {
            escaped = true;
            continue;
        }

        if let Some(active_quote) = quote {
            if character == active_quote {
                quote = None;
            } else {
                current.push(character);
            }
            continue;
        }

        match character {
            '\'' | '"' => quote = Some(character),
            character if character.is_whitespace() => {
                if !current.is_empty() {
                    args.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(character),
        }

        if chars.peek().is_none() && escaped {
            current.push('\\');
            escaped = false;
        }
    }

    if escaped {
        current.push('\\');
    }

    if quote.is_some() {
        return Err("Pipeline contains an unterminated quote.".to_string());
    }

    if !current.is_empty() {
        args.push(current);
    }

    if args.is_empty() {
        Err("Pipeline text is empty.".to_string())
    } else {
        Ok(args)
    }
}

fn gst_launch_args(raw_text: &str) -> Result<Vec<String>, String> {
    let mut args = vec!["--gst-disable-registry-fork".to_string(), "-q".to_string()];
    args.extend(split_pipeline_arguments(raw_text)?);
    Ok(args)
}

fn output_with_timeout(
    command_path: &Path,
    args: &[String],
    timeout: Duration,
) -> Result<(Output, bool), String> {
    let mut child = hidden_command(command_path)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| {
            format!(
                "failed to run `{}` with args `{}`: {error}",
                command_path.display(),
                args.join(" ")
            )
        })?;
    let started = Instant::now();

    loop {
        match child.try_wait() {
            Ok(Some(_)) => {
                return child
                    .wait_with_output()
                    .map(|output| (output, false))
                    .map_err(|error| format!("failed to read gst-launch output: {error}"));
            }
            Ok(None) if started.elapsed() >= timeout => {
                let _ = child.kill();
                return child
                    .wait_with_output()
                    .map(|output| (output, true))
                    .map_err(|error| format!("failed to stop gst-launch after timeout: {error}"));
            }
            Ok(None) => thread::sleep(Duration::from_millis(50)),
            Err(error) => {
                let _ = child.kill();
                return Err(format!("failed to poll gst-launch process: {error}"));
            }
        }
    }
}

#[tauri::command]
pub fn simulate_local_pipeline(raw_text: String) -> PipelineSimulationResponse {
    let command_path = match resolve_gst_launch_command() {
        Ok(command_path) => command_path,
        Err(error) => {
            return PipelineSimulationResponse {
                available: false,
                authority: MetadataAuthority::Local,
                success: false,
                timed_out: false,
                exit_status: None,
                stdout: String::new(),
                stderr: String::new(),
                diagnostic: Some(error),
                command: gst_launch_executable_name().to_string(),
            };
        }
    };
    let args = match gst_launch_args(&raw_text) {
        Ok(args) => args,
        Err(error) => {
            return PipelineSimulationResponse {
                available: true,
                authority: MetadataAuthority::Local,
                success: false,
                timed_out: false,
                exit_status: None,
                stdout: String::new(),
                stderr: String::new(),
                diagnostic: Some(error),
                command: command_path.display().to_string(),
            };
        }
    };
    let command = format!("{} {}", command_path.display(), args.join(" "));

    match output_with_timeout(&command_path, &args, Duration::from_secs(5)) {
        Ok((output, timed_out)) => {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let success = output.status.success() || timed_out;
            let diagnostic = if timed_out {
                Some(
                    "Simulation stopped after 5 seconds without an immediate GStreamer error."
                        .to_string(),
                )
            } else if success {
                None
            } else {
                Some(
                    stderr
                        .clone()
                        .if_empty("gst-launch-1.0 reported a failure."),
                )
            };

            PipelineSimulationResponse {
                available: true,
                authority: MetadataAuthority::Local,
                success,
                timed_out,
                exit_status: output.status.code(),
                stdout,
                stderr,
                diagnostic,
                command,
            }
        }
        Err(error) => PipelineSimulationResponse {
            available: true,
            authority: MetadataAuthority::Local,
            success: false,
            timed_out: false,
            exit_status: None,
            stdout: String::new(),
            stderr: String::new(),
            diagnostic: Some(error),
            command,
        },
    }
}

#[tauri::command]
pub fn prepare_local_playback(raw_text: String) -> PlaybackPrepareResponse {
    let streams = detect_playback_streams(&raw_text);
    let generated_pipeline = build_generated_playback_pipeline(&streams);
    let command_path = match resolve_gst_launch_command() {
        Ok(command_path) => command_path,
        Err(error) => {
            return PlaybackPrepareResponse {
                available: false,
                playable: false,
                streams,
                generated_pipeline,
                diagnostic: Some(error),
                command: gst_launch_executable_name().to_string(),
            };
        }
    };

    let playable = !streams.is_empty() && generated_pipeline.is_some();
    let diagnostic = if playable {
        None
    } else {
        Some("RTP/RTSP IP/Port가 있는 재생 가능한 스트림을 찾지 못했습니다.".to_string())
    };

    PlaybackPrepareResponse {
        available: true,
        playable,
        streams,
        generated_pipeline,
        diagnostic,
        command: command_path.display().to_string(),
    }
}

#[tauri::command]
pub fn start_local_playback(
    raw_text: String,
    state: State<'_, PlaybackState>,
) -> PlaybackStatusResponse {
    let prepare = prepare_local_playback(raw_text);
    if !prepare.available || !prepare.playable {
        return PlaybackStatusResponse {
            state: PlaybackProcessState::Error,
            pid: None,
            command: Some(prepare.command),
            message: prepare.diagnostic,
        };
    }

    let command_path = match resolve_gst_launch_command() {
        Ok(command_path) => command_path,
        Err(error) => {
            return PlaybackStatusResponse {
                state: PlaybackProcessState::Error,
                pid: None,
                command: Some(gst_launch_executable_name().to_string()),
                message: Some(error),
            };
        }
    };
    let generated_pipeline = match prepare.generated_pipeline {
        Some(pipeline) => pipeline,
        None => {
            return PlaybackStatusResponse {
                state: PlaybackProcessState::Error,
                pid: None,
                command: Some(command_path.display().to_string()),
                message: Some("재생용 Pipeline을 생성하지 못했습니다.".to_string()),
            };
        }
    };
    let args = match gst_launch_args(&generated_pipeline) {
        Ok(args) => args,
        Err(error) => {
            return PlaybackStatusResponse {
                state: PlaybackProcessState::Error,
                pid: None,
                command: Some(command_path.display().to_string()),
                message: Some(error),
            };
        }
    };
    let command_text = format!("{} {}", command_path.display(), args.join(" "));
    let mut session = match state.session.lock() {
        Ok(session) => session,
        Err(_) => {
            return PlaybackStatusResponse {
                state: PlaybackProcessState::Error,
                pid: None,
                command: Some(command_text),
                message: Some("Playback 상태 잠금을 가져오지 못했습니다.".to_string()),
            };
        }
    };

    if let Some(existing) = session.as_mut() {
        match existing.child.try_wait() {
            Ok(None) => {
                return PlaybackStatusResponse {
                    state: PlaybackProcessState::Playing,
                    pid: Some(existing.pid),
                    command: Some(existing.command.clone()),
                    message: Some("이미 실행 중인 Playback Pipeline이 있습니다.".to_string()),
                };
            }
            Ok(Some(_)) => {
                *session = None;
            }
            Err(error) => {
                let command = existing.command.clone();
                let _ = existing.child.kill();
                let _ = existing.child.wait();
                *session = None;
                return PlaybackStatusResponse {
                    state: PlaybackProcessState::Error,
                    pid: None,
                    command: Some(command),
                    message: Some(format!("기존 Playback 상태 확인에 실패했습니다: {error}")),
                };
            }
        }
    }

    let mut command = hidden_command(&command_path);
    command
        .args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    match command.spawn() {
        Ok(child) => {
            let pid = child.id();
            *session = Some(PlaybackSession {
                child,
                command: command_text.clone(),
                pid,
            });
            PlaybackStatusResponse {
                state: PlaybackProcessState::Playing,
                pid: Some(pid),
                command: Some(command_text),
                message: Some("Playback Pipeline을 실행했습니다.".to_string()),
            }
        }
        Err(error) => PlaybackStatusResponse {
            state: PlaybackProcessState::Error,
            pid: None,
            command: Some(command_text),
            message: Some(format!("Playback Pipeline 실행에 실패했습니다: {error}")),
        },
    }
}

#[tauri::command]
pub fn stop_local_playback(state: State<'_, PlaybackState>) -> PlaybackStatusResponse {
    let mut session = match state.session.lock() {
        Ok(session) => session,
        Err(_) => {
            return PlaybackStatusResponse {
                state: PlaybackProcessState::Error,
                pid: None,
                command: None,
                message: Some("Playback 상태 잠금을 가져오지 못했습니다.".to_string()),
            };
        }
    };

    let Some(mut existing) = session.take() else {
        return PlaybackStatusResponse {
            state: PlaybackProcessState::Stopped,
            pid: None,
            command: None,
            message: Some("실행 중인 Playback Pipeline이 없습니다.".to_string()),
        };
    };

    let command = existing.command.clone();
    let pid = existing.pid;
    match existing.child.try_wait() {
        Ok(Some(_)) => PlaybackStatusResponse {
            state: PlaybackProcessState::Stopped,
            pid: Some(pid),
            command: Some(command),
            message: Some("Playback Pipeline이 이미 종료되었습니다.".to_string()),
        },
        Ok(None) => {
            let kill_result = existing.child.kill();
            let wait_result = existing.child.wait();
            if let Err(error) = kill_result {
                return PlaybackStatusResponse {
                    state: PlaybackProcessState::Error,
                    pid: Some(pid),
                    command: Some(command),
                    message: Some(format!("Playback Pipeline 종료에 실패했습니다: {error}")),
                };
            }
            if let Err(error) = wait_result {
                return PlaybackStatusResponse {
                    state: PlaybackProcessState::Error,
                    pid: Some(pid),
                    command: Some(command),
                    message: Some(format!(
                        "Playback Pipeline 종료 대기에 실패했습니다: {error}"
                    )),
                };
            }
            PlaybackStatusResponse {
                state: PlaybackProcessState::Stopped,
                pid: Some(pid),
                command: Some(command),
                message: Some("Playback Pipeline을 정지했습니다.".to_string()),
            }
        }
        Err(error) => PlaybackStatusResponse {
            state: PlaybackProcessState::Error,
            pid: Some(pid),
            command: Some(command),
            message: Some(format!(
                "Playback Pipeline 상태 확인에 실패했습니다: {error}"
            )),
        },
    }
}

#[tauri::command]
pub fn get_local_playback_status(state: State<'_, PlaybackState>) -> PlaybackStatusResponse {
    let mut session = match state.session.lock() {
        Ok(session) => session,
        Err(_) => {
            return PlaybackStatusResponse {
                state: PlaybackProcessState::Error,
                pid: None,
                command: None,
                message: Some("Playback 상태 잠금을 가져오지 못했습니다.".to_string()),
            };
        }
    };

    let Some(existing) = session.as_mut() else {
        return PlaybackStatusResponse {
            state: PlaybackProcessState::Idle,
            pid: None,
            command: None,
            message: Some("Playback Pipeline이 실행 중이 아닙니다.".to_string()),
        };
    };

    match existing.child.try_wait() {
        Ok(None) => PlaybackStatusResponse {
            state: PlaybackProcessState::Playing,
            pid: Some(existing.pid),
            command: Some(existing.command.clone()),
            message: Some("Playback Pipeline이 실행 중입니다.".to_string()),
        },
        Ok(Some(status)) => {
            let command = existing.command.clone();
            let pid = existing.pid;
            *session = None;
            PlaybackStatusResponse {
                state: PlaybackProcessState::Stopped,
                pid: Some(pid),
                command: Some(command),
                message: Some(format!(
                    "Playback Pipeline이 종료되었습니다{}.",
                    status
                        .code()
                        .map(|code| format!(" (exit {code})"))
                        .unwrap_or_default()
                )),
            }
        }
        Err(error) => {
            let command = existing.command.clone();
            let pid = existing.pid;
            let _ = existing.child.kill();
            let _ = existing.child.wait();
            *session = None;
            PlaybackStatusResponse {
                state: PlaybackProcessState::Error,
                pid: Some(pid),
                command: Some(command),
                message: Some(format!(
                    "Playback Pipeline 상태 확인에 실패했습니다: {error}"
                )),
            }
        }
    }
}

fn detect_playback_streams(raw_text: &str) -> Vec<PlaybackStream> {
    let mut streams = Vec::new();

    for uri in extract_rtsp_urls(raw_text) {
        if let Some((host, port)) = parse_rtsp_host_port(&uri) {
            let stream_index = streams.len() + 1;
            streams.push(PlaybackStream {
                id: format!("rtsp-{stream_index}"),
                protocol: PlaybackProtocol::Rtsp,
                media_kind: infer_media_kind(&uri),
                uri: Some(uri.clone()),
                host: Some(host),
                port: Some(port),
                caps: None,
                source: uri.clone(),
                playback_pipeline: format!("playbin uri=\"{uri}\""),
            });
        }
    }

    for candidate in extract_rtp_candidates(raw_text) {
        let Some(port) = candidate.port else {
            continue;
        };
        let stream_index = streams.len() + 1;
        let media_kind = infer_media_kind(&candidate.source);
        let sink = match media_kind {
            PlaybackMediaKind::Audio => "autoaudiosink",
            PlaybackMediaKind::Unknown | PlaybackMediaKind::Video => "autovideosink",
        };
        let host_part = candidate
            .host
            .as_deref()
            .map(|host| format!(" address={host}"))
            .unwrap_or_default();
        let caps_part = candidate
            .caps
            .as_deref()
            .map(|caps| format!(" caps=\"{caps}\""))
            .unwrap_or_default();
        let playback_pipeline =
            format!("udpsrc{host_part} port={port}{caps_part} ! decodebin ! {sink}");

        streams.push(PlaybackStream {
            id: format!("rtp-{stream_index}"),
            protocol: PlaybackProtocol::Rtp,
            media_kind,
            uri: None,
            host: candidate.host,
            port: Some(port),
            caps: candidate.caps,
            source: candidate.source,
            playback_pipeline,
        });
    }

    streams
}

fn build_generated_playback_pipeline(streams: &[PlaybackStream]) -> Option<String> {
    if streams.is_empty() {
        return None;
    }

    Some(
        streams
            .iter()
            .map(|stream| stream.playback_pipeline.as_str())
            .collect::<Vec<_>>()
            .join(" "),
    )
}

fn extract_rtsp_urls(raw_text: &str) -> Vec<String> {
    let mut urls = Vec::new();
    let mut offset = 0;

    while let Some(index) = raw_text[offset..].find("rtsp://") {
        let start = offset + index;
        let rest = &raw_text[start..];
        let end = rest
            .find(|character: char| {
                character.is_whitespace()
                    || matches!(character, '"' | '\'' | '`' | '|' | '&' | ';' | '<' | '>')
            })
            .unwrap_or(rest.len());
        let uri = rest[..end]
            .trim_end_matches(|character| matches!(character, ')' | ']' | ','))
            .to_string();
        if !uri.is_empty() && !urls.contains(&uri) {
            urls.push(uri);
        }
        offset = start + end.max("rtsp://".len());
    }

    urls
}

fn parse_rtsp_host_port(uri: &str) -> Option<(String, u16)> {
    let rest = uri.strip_prefix("rtsp://")?;
    let authority = rest
        .split(|character| matches!(character, '/' | '?' | '#'))
        .next()
        .filter(|value| !value.is_empty())?;
    let host_port = authority.rsplit('@').next().unwrap_or(authority);
    let (host, port_text) = host_port.rsplit_once(':')?;
    let port = port_text.parse::<u16>().ok()?;
    if host.is_empty() {
        return None;
    }

    Some((
        host.trim_matches(|character| matches!(character, '[' | ']'))
            .to_string(),
        port,
    ))
}

struct RtpCandidate {
    caps: Option<String>,
    host: Option<String>,
    port: Option<u16>,
    source: String,
}

fn extract_rtp_candidates(raw_text: &str) -> Vec<RtpCandidate> {
    let tokens = match split_pipeline_arguments(raw_text) {
        Ok(tokens) => tokens,
        Err(_) => raw_text
            .split_whitespace()
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>(),
    };
    let mut candidates = Vec::new();
    let mut index = 0;

    while index < tokens.len() {
        if tokens[index] != "udpsrc" {
            index += 1;
            continue;
        }

        let mut source_tokens = vec![tokens[index].clone()];
        let mut cursor = index + 1;
        while cursor < tokens.len() && tokens[cursor] != "!" {
            source_tokens.push(tokens[cursor].clone());
            cursor += 1;
        }
        if cursor + 1 < tokens.len() && tokens[cursor + 1].contains("application/x-rtp") {
            source_tokens.push(tokens[cursor + 1].clone());
        }

        let source = source_tokens.join(" ");
        if !source.contains("application/x-rtp") {
            index = cursor.saturating_add(1);
            continue;
        }

        let mut host = None;
        let mut port = None;
        let mut caps = None;

        for token in &source_tokens {
            if let Some(value) = parse_token_value(token, "address")
                .or_else(|| parse_token_value(token, "host"))
                .or_else(|| parse_token_value(token, "multicast-group"))
            {
                host = Some(value);
            }
            if let Some(value) = parse_token_value(token, "port") {
                port = value.parse::<u16>().ok();
            }
            if let Some(value) = parse_token_value(token, "caps") {
                caps = Some(value);
            } else if token.contains("application/x-rtp") {
                caps.get_or_insert_with(|| token.trim_matches('"').to_string());
            }
        }

        candidates.push(RtpCandidate {
            caps,
            host,
            port,
            source,
        });
        index = cursor.saturating_add(1);
    }

    candidates
}

fn parse_token_value(token: &str, key: &str) -> Option<String> {
    let (candidate_key, value) = token.split_once('=')?;
    if candidate_key != key {
        return None;
    }

    Some(value.trim_matches('"').trim_matches('\'').to_string()).filter(|value| !value.is_empty())
}

fn infer_media_kind(value: &str) -> PlaybackMediaKind {
    let lower = value.to_ascii_lowercase();
    if lower.contains("audio") {
        PlaybackMediaKind::Audio
    } else if lower.contains("video") || lower.contains("h264") || lower.contains("h265") {
        PlaybackMediaKind::Video
    } else {
        PlaybackMediaKind::Unknown
    }
}

#[tauri::command]
pub fn probe_local_gstreamer() -> GStreamerProbeResponse {
    match run_gst_inspect(&["--version"]) {
        Ok((_, output)) if output.status.success() => GStreamerProbeResponse {
            available: true,
            authority: MetadataAuthority::Local,
            version_output: Some(String::from_utf8_lossy(&output.stdout).trim().to_string()),
            diagnostic: None,
        },
        Ok((_, output)) => GStreamerProbeResponse {
            available: false,
            authority: MetadataAuthority::Local,
            version_output: None,
            diagnostic: Some(
                String::from_utf8_lossy(&output.stderr)
                    .trim()
                    .to_string()
                    .if_empty("gst-inspect-1.0 --version command failed."),
            ),
        },
        Err(error) => GStreamerProbeResponse {
            available: false,
            authority: MetadataAuthority::Local,
            version_output: None,
            diagnostic: Some(error),
        },
    }
}

#[tauri::command]
pub fn inspect_local_element(factory_name: String) -> ElementMetadataResponse {
    let factory_name = factory_name.trim().to_string();
    if factory_name.is_empty() {
        return unavailable_element_metadata(
            MetadataAuthority::Local,
            factory_name,
            "Element factory name is empty.",
        );
    }

    match run_gst_inspect(&[factory_name.as_str()]) {
        Ok((_, output)) if output.status.success() => {
            let raw_output = String::from_utf8_lossy(&output.stdout).into_owned();
            parse_gst_inspect_output(MetadataAuthority::Local, factory_name, raw_output)
        }
        Ok((_, output)) => unavailable_element_metadata(
            MetadataAuthority::Local,
            factory_name,
            String::from_utf8_lossy(&output.stderr)
                .trim()
                .to_string()
                .if_empty("gst-inspect-1.0 could not inspect this element."),
        ),
        Err(error) => unavailable_element_metadata(MetadataAuthority::Local, factory_name, error),
    }
}

#[tauri::command]
pub fn probe_remote_target(
    request: RemoteTargetRequest,
    sample_element: Option<String>,
) -> Result<RemoteProbeResponse, String> {
    let mut session = connect_remote(&request)?;
    let version_output = exec_remote_command(&mut session, "gst-inspect-1.0 --version")?;
    let sample_element_output = match sample_element {
        Some(element) if !element.trim().is_empty() => Some(exec_remote_command(
            &mut session,
            &format!("gst-inspect-1.0 {}", shell_escape(&element)),
        )?),
        _ => None,
    };

    Ok(RemoteProbeResponse {
        host: request.host,
        port: request.port.unwrap_or(22),
        username: request.username,
        version_output,
        sample_element_output,
    })
}

#[tauri::command]
pub fn inspect_remote_element(
    request: RemoteTargetRequest,
    factory_name: String,
) -> Result<ElementMetadataResponse, String> {
    let factory_name = factory_name.trim().to_string();
    if factory_name.is_empty() {
        return Ok(unavailable_element_metadata(
            MetadataAuthority::Remote,
            factory_name,
            "Element factory name is empty.",
        ));
    }

    let mut session = connect_remote(&request)?;
    match exec_remote_command(
        &mut session,
        &format!("gst-inspect-1.0 {}", shell_escape(&factory_name)),
    ) {
        Ok(raw_output) => Ok(parse_gst_inspect_output(
            MetadataAuthority::Remote,
            factory_name,
            raw_output,
        )),
        Err(error) => Ok(unavailable_element_metadata(
            MetadataAuthority::Remote,
            factory_name,
            error,
        )),
    }
}

#[tauri::command]
pub fn simulate_remote_pipeline(
    request: RemoteTargetRequest,
    raw_text: String,
) -> Result<PipelineSimulationResponse, String> {
    let args = match gst_launch_args(&raw_text) {
        Ok(args) => args,
        Err(error) => {
            return Ok(PipelineSimulationResponse {
                available: true,
                authority: MetadataAuthority::Remote,
                success: false,
                timed_out: false,
                exit_status: None,
                stdout: String::new(),
                stderr: String::new(),
                diagnostic: Some(error),
                command: "gst-launch-1.0".to_string(),
            });
        }
    };
    let mut session = connect_remote(&request)?;
    let command = format!(
        "command -v gst-launch-1.0 >/dev/null 2>&1 || {{ echo 'gst-launch-1.0 is not available on remote target.' >&2; exit 127; }}; timeout 5s gst-launch-1.0 {}",
        args.iter()
            .map(|arg| shell_escape(arg))
            .collect::<Vec<_>>()
            .join(" "),
    );
    let output = exec_remote_command_output(&mut session, &command)?;
    let timed_out = output.exit_status == 124;
    let available = output.exit_status != 127;
    let success = available && (output.exit_status == 0 || timed_out);
    let diagnostic = if !available {
        Some(
            output
                .stderr
                .clone()
                .if_empty("gst-launch-1.0 is not available on remote target."),
        )
    } else if timed_out {
        Some(
            "Remote simulation stopped after 5 seconds without an immediate GStreamer error."
                .to_string(),
        )
    } else if success {
        None
    } else {
        Some(
            output
                .stderr
                .clone()
                .if_empty("remote gst-launch-1.0 reported a failure."),
        )
    };

    Ok(PipelineSimulationResponse {
        available,
        authority: MetadataAuthority::Remote,
        success,
        timed_out,
        exit_status: Some(output.exit_status),
        stdout: output.stdout,
        stderr: output.stderr,
        diagnostic,
        command,
    })
}

#[tauri::command]
pub fn load_remote_pipeline(
    request: RemoteTargetRequest,
    path: String,
) -> Result<PipelineDocument, String> {
    let session = connect_remote(&request)?;
    let sftp = session
        .sftp()
        .map_err(|error| format!("failed to open SFTP session: {error}"))?;
    let mut remote_file = sftp
        .open(Path::new(&path))
        .map_err(|error| format!("failed to open remote file `{path}`: {error}"))?;
    let mut raw_text = String::new();
    remote_file
        .read_to_string(&mut raw_text)
        .map_err(|error| format!("failed to read remote file `{path}`: {error}"))?;
    let normalization = normalize_text(&raw_text);

    Ok(parse_document(
        raw_text,
        normalization.normalized_text,
        SourceKind::RemoteFile,
        Some(path.clone()),
        Some(
            Path::new(&path)
                .file_name()
                .map(|value| value.to_string_lossy().into_owned())
                .unwrap_or(path),
        ),
        normalization.diagnostics,
    ))
}

fn connect_remote(request: &RemoteTargetRequest) -> Result<Session, String> {
    let tcp = TcpStream::connect((request.host.as_str(), request.port.unwrap_or(22)))
        .map_err(|error| format!("failed to connect to {}: {error}", request.host))?;
    let mut session =
        Session::new().map_err(|error| format!("failed to create SSH session: {error}"))?;
    session.set_tcp_stream(tcp);
    session
        .handshake()
        .map_err(|error| format!("SSH handshake failed: {error}"))?;
    session
        .userauth_password(&request.username, &request.password)
        .map_err(|error| format!("SSH authentication failed: {error}"))?;

    if !session.authenticated() {
        return Err("SSH authentication did not complete successfully.".into());
    }

    Ok(session)
}

fn exec_remote_command(session: &mut Session, command: &str) -> Result<String, String> {
    let output = exec_remote_command_output(session, command)?;

    if output.exit_status != 0 {
        return Err(format!(
            "remote command `{command}` failed with status {}: {}",
            output.exit_status,
            output.stderr.trim()
        ));
    }

    Ok(output.stdout)
}

struct RemoteCommandOutput {
    exit_status: i32,
    stdout: String,
    stderr: String,
}

fn exec_remote_command_output(
    session: &mut Session,
    command: &str,
) -> Result<RemoteCommandOutput, String> {
    let mut channel = session
        .channel_session()
        .map_err(|error| format!("failed to open SSH channel: {error}"))?;
    channel
        .exec(command)
        .map_err(|error| format!("failed to execute remote command `{command}`: {error}"))?;
    let mut stdout = String::new();
    channel
        .read_to_string(&mut stdout)
        .map_err(|error| format!("failed to read remote command output: {error}"))?;
    let mut stderr = String::new();
    channel
        .stderr()
        .read_to_string(&mut stderr)
        .map_err(|error| format!("failed to read remote command stderr: {error}"))?;
    channel
        .wait_close()
        .map_err(|error| format!("failed to finalize remote command: {error}"))?;
    let exit_status = channel
        .exit_status()
        .map_err(|error| format!("failed to inspect remote command exit status: {error}"))?;

    Ok(RemoteCommandOutput {
        exit_status,
        stdout,
        stderr,
    })
}

fn shell_escape(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn unavailable_element_metadata(
    authority: MetadataAuthority,
    factory_name: String,
    diagnostic: impl Into<String>,
) -> ElementMetadataResponse {
    ElementMetadataResponse {
        available: false,
        authority,
        factory_name,
        long_name: None,
        klass: None,
        description: None,
        plugin_name: None,
        properties: Vec::new(),
        pad_templates: Vec::new(),
        raw_output: None,
        diagnostic: Some(diagnostic.into()),
    }
}

fn parse_gst_inspect_output(
    authority: MetadataAuthority,
    factory_name: String,
    raw_output: String,
) -> ElementMetadataResponse {
    let mut metadata = ElementMetadataResponse {
        available: true,
        authority,
        factory_name,
        long_name: None,
        klass: None,
        description: None,
        plugin_name: None,
        properties: Vec::new(),
        pad_templates: Vec::new(),
        raw_output: Some(raw_output.clone()),
        diagnostic: None,
    };
    let mut section = "";
    let mut current_pad: Option<ElementPadTemplateMetadata> = None;
    let mut collecting_pad_caps = false;

    for line in raw_output.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        match trimmed {
            "Factory Details:" => {
                flush_pad_template(&mut metadata.pad_templates, &mut current_pad);
                collecting_pad_caps = false;
                section = "factory";
                continue;
            }
            "Plugin Details:" => {
                flush_pad_template(&mut metadata.pad_templates, &mut current_pad);
                collecting_pad_caps = false;
                section = "plugin";
                continue;
            }
            "Pad Templates:" => {
                section = "pads";
                continue;
            }
            "Element Properties:" => {
                flush_pad_template(&mut metadata.pad_templates, &mut current_pad);
                collecting_pad_caps = false;
                section = "properties";
                continue;
            }
            "Element Signals:"
            | "Element Actions:"
            | "Children:"
            | "Pads:"
            | "Clocking Interaction:" => {
                flush_pad_template(&mut metadata.pad_templates, &mut current_pad);
                collecting_pad_caps = false;
                section = "";
                continue;
            }
            _ => {}
        }

        match section {
            "factory" => {
                if let Some(value) = parse_gst_field(trimmed, "Long-name") {
                    metadata.long_name = Some(value);
                } else if let Some(value) = parse_gst_field(trimmed, "Klass") {
                    metadata.klass = Some(value);
                } else if let Some(value) = parse_gst_field(trimmed, "Description") {
                    metadata.description.get_or_insert(value);
                }
            }
            "plugin" => {
                if let Some(value) = parse_gst_field(trimmed, "Name") {
                    metadata.plugin_name.get_or_insert(value);
                } else if let Some(value) = parse_gst_field(trimmed, "Description") {
                    metadata.description.get_or_insert(value);
                }
            }
            "pads" => {
                if let Some((direction, name)) = parse_pad_template_heading(trimmed) {
                    flush_pad_template(&mut metadata.pad_templates, &mut current_pad);
                    collecting_pad_caps = false;
                    current_pad = Some(ElementPadTemplateMetadata {
                        direction,
                        name,
                        presence: None,
                        caps: Vec::new(),
                    });
                } else if let Some(value) = parse_gst_field(trimmed, "Availability") {
                    collecting_pad_caps = false;
                    if let Some(pad) = &mut current_pad {
                        pad.presence = Some(value);
                    }
                } else if trimmed == "Capabilities:" {
                    collecting_pad_caps = true;
                } else if collecting_pad_caps {
                    if let Some(pad) = &mut current_pad {
                        pad.caps.push(trimmed.to_string());
                    }
                }
            }
            "properties" => {
                if let Some(property) = parse_property_line(trimmed) {
                    metadata.properties.push(property);
                } else if let Some(property) = metadata.properties.last_mut() {
                    update_property_metadata(property, trimmed);
                }
            }
            _ => {}
        }
    }

    flush_pad_template(&mut metadata.pad_templates, &mut current_pad);
    metadata
}

fn parse_gst_field(line: &str, field: &str) -> Option<String> {
    line.strip_prefix(field)
        .map(|value| value.trim().trim_start_matches(':').trim())
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn parse_pad_template_heading(line: &str) -> Option<(String, String)> {
    let (direction, rest) = line.split_once(" template: ")?;
    let name = rest.trim().trim_matches('\'').to_string();
    if direction.is_empty() || name.is_empty() {
        return None;
    }

    Some((direction.to_string(), name))
}

fn parse_property_line(line: &str) -> Option<ElementPropertyMetadata> {
    let (name, description) = line.split_once(':')?;
    let name = name.trim();

    if name.is_empty()
        || name.starts_with('(')
        || name.contains(char::is_whitespace)
        || matches!(name, "flags" | "Enum" | "Default")
    {
        return None;
    }

    Some(ElementPropertyMetadata {
        name: name.to_string(),
        description: Some(description.trim().to_string()).filter(|value| !value.is_empty()),
        value_type: None,
        default_value: None,
        current_value: None,
    })
}

fn update_property_metadata(property: &mut ElementPropertyMetadata, line: &str) {
    if line.is_empty() || line.starts_with("flags:") || line.starts_with('(') {
        return;
    }

    if let Some(value) = parse_detail_value(line, "Default:") {
        property.default_value = Some(value);
    }

    if let Some(value) = parse_detail_value(line, "Current:") {
        property.current_value = Some(value);
    }

    if property.value_type.is_none() {
        if let Some(value_type) = parse_property_value_type(line) {
            property.value_type = Some(value_type);
        }
    }

    if !line.contains("Default:")
        && !line.contains("Current:")
        && !line.starts_with("Enum ")
        && !line.starts_with("Object of type ")
    {
        match &mut property.description {
            Some(description) if !description.contains(line) => {
                description.push(' ');
                description.push_str(line);
            }
            None => property.description = Some(line.to_string()),
            _ => {}
        }
    }
}

fn parse_detail_value(line: &str, marker: &str) -> Option<String> {
    let start = line.find(marker)? + marker.len();
    let rest = line[start..].trim();
    let end = [" Default:", " Current:"]
        .iter()
        .filter(|candidate| **candidate != marker)
        .filter_map(|candidate| rest.find(candidate))
        .min()
        .unwrap_or(rest.len());
    let value = rest[..end].trim();
    Some(value.trim_end_matches('.').to_string()).filter(|value| !value.is_empty())
}

fn parse_property_value_type(line: &str) -> Option<String> {
    if let Some(default_index) = line.find("Default:") {
        let before_default = line[..default_index].trim();
        return before_default
            .split(" Range:")
            .next()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned);
    }

    if line.starts_with("Enum ") || line.starts_with("Object of type ") {
        return Some(line.to_string());
    }

    None
}

fn flush_pad_template(
    pad_templates: &mut Vec<ElementPadTemplateMetadata>,
    current_pad: &mut Option<ElementPadTemplateMetadata>,
) {
    if let Some(pad) = current_pad.take() {
        pad_templates.push(pad);
    }
}

trait EmptyStringFallback {
    fn if_empty(self, fallback: &str) -> String;
}

impl EmptyStringFallback for String {
    fn if_empty(self, fallback: &str) -> String {
        if self.is_empty() {
            fallback.to_string()
        } else {
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_export_path(file_name: &str) -> String {
        let unique_id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();

        std::env::temp_dir()
            .join(format!("gst-topology-export-{unique_id}-{file_name}"))
            .to_string_lossy()
            .into_owned()
    }

    #[test]
    fn save_export_file_writes_base64_payload() {
        let path = unique_export_path("topology.png");

        let saved_path = save_export_file(
            path.clone(),
            general_purpose::STANDARD.encode([0x89, b'P', b'N', b'G']),
        )
        .expect("base64 export should save");

        assert_eq!(saved_path.as_deref(), Some(path.as_str()));
        assert_eq!(
            fs::read(&path).expect("saved png payload should be readable"),
            vec![0x89, b'P', b'N', b'G']
        );

        let _ = fs::remove_file(path);
    }

    #[test]
    fn save_export_file_treats_blank_path_as_cancel() {
        let saved_path = save_export_file(
            "   ".to_string(),
            general_purpose::STANDARD.encode("ignored"),
        )
        .expect("blank path should be treated as cancel");

        assert_eq!(saved_path, None);
    }

    #[test]
    fn save_export_file_rejects_invalid_base64_payload() {
        let path = unique_export_path("invalid.png");

        let error = save_export_file(path, "not valid base64".to_string())
            .expect_err("invalid base64 should fail");

        assert!(error.contains("failed to decode export payload"));
    }

    #[test]
    fn save_export_file_rejects_directory_target() {
        let error = save_export_file(
            std::env::temp_dir().to_string_lossy().into_owned(),
            general_purpose::STANDARD.encode("ignored"),
        )
        .expect_err("directory target should fail");

        assert!(error.contains("export path is a directory"));
    }

    #[test]
    fn save_export_file_rejects_missing_parent_folder() {
        let path = std::env::temp_dir()
            .join("gst-topology-missing-parent")
            .join("topology.png")
            .to_string_lossy()
            .into_owned();

        let error = save_export_file(path, general_purpose::STANDARD.encode("ignored"))
            .expect_err("missing parent should fail");

        assert!(error.contains("export folder does not exist"));
    }

    #[test]
    fn export_file_name_is_sanitized_for_download_saves() {
        assert_eq!(
            safe_export_file_name("C:\\tmp\\bad:name?.png").expect("file name should sanitize"),
            "bad-name-.png"
        );
        assert!(safe_export_file_name("   ").is_err());
    }

    #[test]
    fn next_available_export_path_avoids_overwriting_existing_files() {
        let first_path = unique_export_path("topology.png");
        fs::write(&first_path, "existing").expect("existing export should be writable");
        let folder = Path::new(&first_path)
            .parent()
            .expect("unique export path should have a parent");
        let file_name = Path::new(&first_path)
            .file_name()
            .and_then(|value| value.to_str())
            .expect("unique export path should have a file name");

        let next_path = next_available_export_path(folder, file_name);

        assert_ne!(next_path, PathBuf::from(&first_path));
        assert!(next_path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .contains("-1"));

        let _ = fs::remove_file(first_path);
    }

    #[test]
    fn split_pipeline_arguments_keeps_quoted_values() {
        let args = split_pipeline_arguments(
            "videotestsrc pattern=smpte ! textoverlay text=\"hello world\" ! fakesink",
        )
        .expect("quoted pipeline should split");

        assert_eq!(
            args,
            vec![
                "videotestsrc",
                "pattern=smpte",
                "!",
                "textoverlay",
                "text=hello world",
                "!",
                "fakesink",
            ]
        );
    }

    #[test]
    fn split_pipeline_arguments_rejects_unterminated_quote() {
        let error = split_pipeline_arguments("videotestsrc text=\"missing")
            .expect_err("unterminated quote should fail");

        assert!(error.contains("unterminated quote"));
    }

    #[test]
    fn parse_gst_inspect_output_collects_property_details() {
        let raw_output = r#"
Factory Details:
  Long-name                Video test source
  Klass                    Source/Video
  Description              Creates a test video stream

Plugin Details:
  Name                     videotestsrc

Pad Templates:
  SRC template: 'src'
    Availability: Always

Element Properties:
  name                : The name of the object
                        flags: readable, writable
                        String. Default: "videotestsrc0"
  pattern             : Type of test pattern to generate
                        flags: readable, writable
                        Enum "GstVideoTestSrcPattern" Default: 0, "smpte"
                           (0): smpte            - SMPTE 100% color bars
  blocksize           : Size in bytes to read per buffer (-1 = default)
                        flags: readable, writable
                        Unsigned Integer. Range: 0 - 4294967295 Default: 4096

Element Signals:
  no-more-pads ()
"#
        .to_string();

        let metadata = parse_gst_inspect_output(
            MetadataAuthority::Remote,
            "videotestsrc".to_string(),
            raw_output,
        );

        assert_eq!(metadata.plugin_name.as_deref(), Some("videotestsrc"));
        assert_eq!(metadata.pad_templates.len(), 1);
        assert_eq!(metadata.pad_templates[0].name, "src");
        assert_eq!(metadata.pad_templates[0].direction, "SRC");
        assert_eq!(
            metadata.pad_templates[0].presence.as_deref(),
            Some("Always")
        );
        assert_eq!(metadata.properties.len(), 3);
        assert_eq!(metadata.properties[0].name, "name");
        assert_eq!(
            metadata.properties[0].value_type.as_deref(),
            Some("String.")
        );
        assert_eq!(
            metadata.properties[0].default_value.as_deref(),
            Some("\"videotestsrc0\"")
        );
        assert_eq!(metadata.properties[1].name, "pattern");
        assert_eq!(
            metadata.properties[1].value_type.as_deref(),
            Some("Enum \"GstVideoTestSrcPattern\"")
        );
        assert_eq!(
            metadata.properties[1].default_value.as_deref(),
            Some("0, \"smpte\"")
        );
        assert_eq!(metadata.properties[2].name, "blocksize");
        assert_eq!(
            metadata.properties[2].value_type.as_deref(),
            Some("Unsigned Integer.")
        );
        assert_eq!(
            metadata.properties[2].default_value.as_deref(),
            Some("4096")
        );
    }

    #[test]
    fn parse_gst_inspect_output_collects_pad_caps() {
        let raw_output = r#"
Factory Details:
  Long-name                Video converter

Pad Templates:
  SINK template: 'sink'
    Availability: Always
    Capabilities:
      video/x-raw
                 format: { ABGR64_LE, BGRA64_LE, AYUV64 }
                  width: [ 1, 32767 ]
                 height: [ 1, 32767 ]
  SRC template: 'src'
    Availability: Always
    Capabilities:
      video/x-raw
                 format: { ABGR64_LE, BGRA64_LE, AYUV64 }

Element Properties:
  name                : The name of the object
"#
        .to_string();

        let metadata = parse_gst_inspect_output(
            MetadataAuthority::Local,
            "videoconvert".to_string(),
            raw_output,
        );

        assert_eq!(metadata.pad_templates.len(), 2);
        assert_eq!(metadata.pad_templates[0].direction, "SINK");
        assert_eq!(
            metadata.pad_templates[0].presence.as_deref(),
            Some("Always")
        );
        assert!(metadata.pad_templates[0]
            .caps
            .iter()
            .any(|line| line == "video/x-raw"));
        assert!(metadata.pad_templates[0]
            .caps
            .iter()
            .any(|line| line.contains("format:")));
        assert_eq!(metadata.pad_templates[1].direction, "SRC");
        assert!(metadata.pad_templates[1]
            .caps
            .iter()
            .any(|line| line == "video/x-raw"));
    }

    #[test]
    fn playback_detection_accepts_rtsp_with_explicit_port() {
        let streams = detect_playback_streams(
            "rtspsrc location=rtsp://192.168.0.10:8554/camera ! rtph264depay ! fakesink",
        );

        assert_eq!(streams.len(), 1);
        assert_eq!(streams[0].protocol, PlaybackProtocol::Rtsp);
        assert_eq!(streams[0].host.as_deref(), Some("192.168.0.10"));
        assert_eq!(streams[0].port, Some(8554));
        assert!(streams[0]
            .playback_pipeline
            .contains("playbin uri=\"rtsp://192.168.0.10:8554/camera\""));
    }

    #[test]
    fn playback_detection_requires_rtsp_port() {
        let streams = detect_playback_streams(
            "rtspsrc location=rtsp://192.168.0.10/camera ! rtph264depay ! fakesink",
        );

        assert!(streams.is_empty());
    }

    #[test]
    fn playback_detection_accepts_rtp_udp_caps() {
        let streams = detect_playback_streams(
            "udpsrc address=239.0.0.1 port=5004 caps=\"application/x-rtp,media=video,encoding-name=H264,payload=96\" ! rtph264depay ! fakesink",
        );

        assert_eq!(streams.len(), 1);
        assert_eq!(streams[0].protocol, PlaybackProtocol::Rtp);
        assert_eq!(streams[0].media_kind, PlaybackMediaKind::Video);
        assert_eq!(streams[0].host.as_deref(), Some("239.0.0.1"));
        assert_eq!(streams[0].port, Some(5004));
        assert!(streams[0]
            .playback_pipeline
            .contains("caps=\"application/x-rtp,media=video"));
    }

    #[test]
    fn playback_detection_splits_dual_rtp_streams() {
        let streams = detect_playback_streams(
            "udpsrc port=5004 caps=\"application/x-rtp,media=video\" ! fakesink udpsrc port=5006 caps=\"application/x-rtp,media=audio\" ! fakesink",
        );

        assert_eq!(streams.len(), 2);
        assert_eq!(streams[0].media_kind, PlaybackMediaKind::Video);
        assert_eq!(streams[1].media_kind, PlaybackMediaKind::Audio);
        assert_eq!(streams[0].port, Some(5004));
        assert_eq!(streams[1].port, Some(5006));
    }

    #[test]
    fn playback_detection_blocks_non_streaming_pipeline() {
        let streams = detect_playback_streams("videotestsrc ! videoconvert ! autovideosink");

        assert!(streams.is_empty());
        assert!(build_generated_playback_pipeline(&streams).is_none());
    }

    #[test]
    fn playback_detection_does_not_preserve_shell_injection_suffix() {
        let streams = detect_playback_streams(
            "rtspsrc location=rtsp://127.0.0.1:8554/live;touch /tmp/owned ! fakesink",
        );

        assert_eq!(streams.len(), 1);
        assert_eq!(
            streams[0].uri.as_deref(),
            Some("rtsp://127.0.0.1:8554/live")
        );
        assert!(!streams[0].playback_pipeline.contains("touch"));
    }
}
