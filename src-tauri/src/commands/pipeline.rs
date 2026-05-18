use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use base64::{engine::general_purpose, Engine as _};
use ssh2::Session;
use tauri::State;

use crate::models::{
    ElementMetadataResponse, ElementPadTemplateMetadata, ElementPropertyMetadata,
    GStreamerProbeResponse, MetadataAuthority, PipelineDocument, PipelineSimulationResponse,
    PlaybackDirection, PlaybackFrameResponse, PlaybackLocation, PlaybackMediaKind,
    PlaybackPrepareResponse, PlaybackProcessState, PlaybackProtocol, PlaybackSourceRole,
    PlaybackStatusResponse, PlaybackStream, RemoteProbeResponse, RemoteTargetRequest, SourceKind,
};
use crate::parser::{normalize_text, parse_document};

#[derive(Default)]
pub struct PlaybackState {
    session: Mutex<Option<PlaybackSession>>,
}

struct PlaybackSession {
    command: String,
    frame_sources: Vec<PlaybackFrameSource>,
    pid: u32,
    preview_dir: Option<PathBuf>,
    preview_server: Option<MjpegPreviewServer>,
    processes: Vec<PlaybackProcessHandle>,
}

struct PlaybackProcess {
    child: Child,
    command: String,
    log_path: Option<PathBuf>,
    pid: u32,
}

struct RemotePlaybackProcess {
    command: String,
    log_path: String,
    pid: u32,
    request: RemoteTargetRequest,
}

enum PlaybackProcessHandle {
    Local(PlaybackProcess),
    Remote(RemotePlaybackProcess),
}

#[derive(Clone)]
struct PlaybackFrameSource {
    folder: PathBuf,
    stream_id: String,
    stream_url: Option<String>,
}

struct MjpegPreviewServer {
    base_url: String,
    handle: Option<JoinHandle<()>>,
    stop: Arc<AtomicBool>,
}

impl Drop for PlaybackSession {
    fn drop(&mut self) {
        for process in &mut self.processes {
            let _ = kill_playback_process(process);
        }
        if let Some(server) = self.preview_server.take() {
            drop(server);
        }
        if let Some(preview_dir) = &self.preview_dir {
            let _ = fs::remove_dir_all(preview_dir);
        }
    }
}

impl Drop for MjpegPreviewServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
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

fn gstreamer_command(program: &Path) -> Command {
    let mut command = hidden_command(program);
    if let Some(prefix) = gstreamer_prefix_for(program) {
        let plugin_path = prefix.join("lib").join("gstreamer-1.0");
        if plugin_path.exists() {
            command.env("GST_PLUGIN_SYSTEM_PATH_1_0", plugin_path);
        }

        if cfg!(target_os = "macos") {
            let library_path = prefix.join("lib");
            if library_path.exists() {
                command.env("DYLD_FALLBACK_LIBRARY_PATH", library_path);
            }

            let typelib_path = prefix.join("lib").join("girepository-1.0");
            if typelib_path.exists() {
                command.env("GI_TYPELIB_PATH", typelib_path);
            }
        }
    }
    if let Some(scanner) = gstreamer_plugin_scanner_for(program) {
        command.env("GST_PLUGIN_SCANNER", scanner);
    }
    command
}

fn gstreamer_prefix_for(program: &Path) -> Option<PathBuf> {
    let canonical_program = fs::canonicalize(program).ok()?;
    Some(canonical_program.parent()?.parent()?.to_path_buf())
}

fn gstreamer_plugin_scanner_for(program: &Path) -> Option<PathBuf> {
    let prefix = gstreamer_prefix_for(program)?;
    let scanner = prefix
        .join("libexec")
        .join("gstreamer-1.0")
        .join("gst-plugin-scanner");

    scanner.exists().then_some(scanner)
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

    if let Some(path_value) = env::var_os("PATH") {
        for folder in env::split_paths(&path_value) {
            push_unique_candidate(&mut candidates, folder.join(executable_name));
        }
    }

    if let Some(home) = home_dir() {
        for folder in ["anaconda3/bin", "miniconda3/bin", "mambaforge/bin"] {
            push_unique_candidate(&mut candidates, home.join(folder).join(executable_name));
        }
    }

    push_unique_candidate(&mut candidates, PathBuf::from(executable_name));

    candidates
}

fn resolve_gst_inspect_command() -> Result<PathBuf, String> {
    let candidates = gst_inspect_command_candidates();
    let mut diagnostics = Vec::new();

    for candidate in &candidates {
        match gstreamer_command(candidate).arg("--version").output() {
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
        match gstreamer_command(candidate).arg("--version").output() {
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
    let output = gstreamer_command(&command_path)
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
    let mut child = gstreamer_command(command_path)
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

struct PlaybackPlan {
    counterpart_pipeline: Option<String>,
    diagnostic: Option<String>,
    generated_pipeline: Option<String>,
    launch_steps: Vec<PlaybackLaunchStep>,
    playable: bool,
    source_location: PlaybackLocation,
    source_pipeline: Option<String>,
    source_role: PlaybackSourceRole,
    streams: Vec<PlaybackStream>,
}

struct PlaybackLaunchStep {
    location: PlaybackLocation,
    pipeline: String,
    request: Option<RemoteTargetRequest>,
}

fn playback_source_location(request: &Option<RemoteTargetRequest>) -> PlaybackLocation {
    if request.is_some() {
        PlaybackLocation::Remote
    } else {
        PlaybackLocation::Local
    }
}

fn build_playback_plan(
    raw_text: &str,
    request: Option<&RemoteTargetRequest>,
    preview_dir: Option<&Path>,
) -> PlaybackPlan {
    let streams = detect_playback_streams(raw_text);
    let source_location = if request.is_some() {
        PlaybackLocation::Remote
    } else {
        PlaybackLocation::Local
    };
    let source_pipeline = Some(raw_text.trim().to_string());
    let source_role = playback_source_role(&streams);
    let mut diagnostics = Vec::new();
    let mut counterpart_pipelines = Vec::new();
    let mut launch_steps = Vec::new();

    if streams.is_empty() {
        diagnostics.push("RTP IP/Port가 있는 재생 가능한 스트림을 찾지 못했습니다.".to_string());
    }

    if source_role == PlaybackSourceRole::Mixed {
        diagnostics.push(
            "Sender와 Receiver가 섞인 RTP Pipeline은 이번 Playback slice에서 지원하지 않습니다."
                .to_string(),
        );
    }

    if source_role == PlaybackSourceRole::Sender && source_location == PlaybackLocation::Remote {
        diagnostics.push(
            "Remote Sender PLD는 udpsink host가 이 PC에서 수신 가능한 IP로 설정되어 있어야 App preview가 표시됩니다."
                .to_string(),
        );
    }

    match source_role {
        PlaybackSourceRole::Sender => {
            let runtime_streams = sender_runtime_streams(&streams, &source_location);
            for stream in &runtime_streams {
                if !supports_frame_preview(stream) {
                    diagnostics.push(format!(
                        "{} 스트림은 video preview를 지원하지 않습니다.",
                        stream.id
                    ));
                    continue;
                }
                let pipeline = preview_pipeline_for_stream(stream, preview_dir);
                counterpart_pipelines.push(pipeline.clone());
                launch_steps.push(PlaybackLaunchStep {
                    location: PlaybackLocation::Local,
                    pipeline,
                    request: None,
                });
            }
            launch_steps.push(PlaybackLaunchStep {
                location: source_location.clone(),
                pipeline: if source_location == PlaybackLocation::Local {
                    rewrite_sender_pipeline_ports(raw_text, &streams, &runtime_streams)
                } else {
                    raw_text.trim().to_string()
                },
                request: request.cloned(),
            });
        }
        PlaybackSourceRole::Receiver => {
            launch_steps.push(PlaybackLaunchStep {
                location: source_location.clone(),
                pipeline: raw_text.trim().to_string(),
                request: request.cloned(),
            });
            for (index, stream) in streams.iter().enumerate() {
                match test_sender_pipeline_for_stream(stream, request, preview_dir, index) {
                    Some(pipeline) => {
                        counterpart_pipelines.push(pipeline.clone());
                        launch_steps.push(PlaybackLaunchStep {
                            location: PlaybackLocation::Local,
                            pipeline,
                            request: None,
                        });
                    }
                    None => diagnostics.push(format!(
                        "{} 스트림은 RAW RTP video test sender 자동 생성을 지원하지 않습니다.",
                        stream.id
                    )),
                }
            }
        }
        PlaybackSourceRole::Mixed | PlaybackSourceRole::Unsupported => {}
    }

    let playable = !streams.is_empty()
        && matches!(
            source_role,
            PlaybackSourceRole::Sender | PlaybackSourceRole::Receiver
        )
        && !launch_steps.is_empty()
        && diagnostics
            .iter()
            .all(|message| message.starts_with("Remote Sender PLD"));
    let counterpart_pipeline = join_pipeline_lines(&counterpart_pipelines);
    let generated_pipeline = join_pipeline_lines(
        &launch_steps
            .iter()
            .map(|step| step.pipeline.clone())
            .collect::<Vec<_>>(),
    );
    let diagnostic = if diagnostics.is_empty() {
        None
    } else {
        Some(diagnostics.join(" "))
    };

    PlaybackPlan {
        counterpart_pipeline,
        diagnostic,
        generated_pipeline,
        launch_steps,
        playable,
        source_location,
        source_pipeline,
        source_role,
        streams,
    }
}

fn join_pipeline_lines(pipelines: &[String]) -> Option<String> {
    if pipelines.is_empty() {
        None
    } else {
        Some(pipelines.join("\n"))
    }
}

fn playback_source_role(streams: &[PlaybackStream]) -> PlaybackSourceRole {
    if streams.is_empty() {
        return PlaybackSourceRole::Unsupported;
    }

    let has_sender = streams
        .iter()
        .any(|stream| stream.direction == PlaybackDirection::Sender);
    let has_receiver = streams
        .iter()
        .any(|stream| stream.direction == PlaybackDirection::Receiver);

    match (has_sender, has_receiver) {
        (true, false) => PlaybackSourceRole::Sender,
        (false, true) => PlaybackSourceRole::Receiver,
        (true, true) => PlaybackSourceRole::Mixed,
        (false, false) => PlaybackSourceRole::Unsupported,
    }
}

fn sender_runtime_streams(
    streams: &[PlaybackStream],
    source_location: &PlaybackLocation,
) -> Vec<PlaybackStream> {
    streams
        .iter()
        .enumerate()
        .map(|(index, stream)| {
            let mut runtime_stream = stream.clone();
            if *source_location == PlaybackLocation::Local {
                runtime_stream.port = Some(local_preview_port(index));
                runtime_stream.host = Some("127.0.0.1".to_string());
            }
            runtime_stream
        })
        .collect()
}

fn local_preview_port(index: usize) -> u16 {
    17000 + u16::try_from(index).unwrap_or(0)
}

fn rewrite_sender_pipeline_ports(
    raw_text: &str,
    original_streams: &[PlaybackStream],
    runtime_streams: &[PlaybackStream],
) -> String {
    let Ok(mut tokens) = split_pipeline_arguments(raw_text) else {
        return raw_text.trim().to_string();
    };
    let mut stream_index = 0usize;
    let mut index = 0usize;

    while index < tokens.len() {
        if tokens[index] != "udpsink" {
            index += 1;
            continue;
        }

        let end = next_link_index(&tokens, index + 1).unwrap_or(tokens.len());
        if let (Some(original), Some(runtime)) = (
            original_streams.get(stream_index),
            runtime_streams.get(stream_index),
        ) {
            rewrite_udpsink_segment(&mut tokens[index..end], original, runtime);
        }
        stream_index += 1;
        index = end.saturating_add(1);
    }

    tokens.join(" ")
}

fn next_link_index(tokens: &[String], start: usize) -> Option<usize> {
    tokens
        .iter()
        .enumerate()
        .skip(start)
        .find_map(|(index, token)| (token == "!").then_some(index))
}

fn rewrite_udpsink_segment(
    tokens: &mut [String],
    original_stream: &PlaybackStream,
    runtime_stream: &PlaybackStream,
) {
    for token in tokens {
        if token.starts_with("port=") && original_stream.port.is_some() {
            if let Some(port) = runtime_stream.port {
                *token = format!("port={port}");
            }
        } else if token.starts_with("host=") || token.starts_with("address=") {
            let Some(host) = runtime_stream.host.as_deref() else {
                continue;
            };
            let key = token.split_once('=').map(|(key, _)| key).unwrap_or("host");
            *token = format!("{key}={host}");
        }
    }
}

#[tauri::command]
pub fn prepare_local_playback(
    raw_text: String,
    request: Option<RemoteTargetRequest>,
) -> PlaybackPrepareResponse {
    let source_location = playback_source_location(&request);
    let command_path = match resolve_gst_launch_command() {
        Ok(command_path) => command_path,
        Err(error) => {
            return PlaybackPrepareResponse {
                available: false,
                playable: false,
                source_role: PlaybackSourceRole::Unsupported,
                source_location,
                counterpart_location: PlaybackLocation::Local,
                streams: detect_playback_streams(&raw_text),
                generated_pipeline: None,
                source_pipeline: Some(raw_text.trim().to_string()),
                counterpart_pipeline: None,
                diagnostic: Some(error),
                command: gst_launch_executable_name().to_string(),
            };
        }
    };

    let plan = build_playback_plan(&raw_text, request.as_ref(), None);

    PlaybackPrepareResponse {
        available: true,
        playable: plan.playable,
        source_role: plan.source_role,
        source_location: plan.source_location,
        counterpart_location: PlaybackLocation::Local,
        streams: plan.streams,
        generated_pipeline: plan.generated_pipeline,
        source_pipeline: plan.source_pipeline,
        counterpart_pipeline: plan.counterpart_pipeline,
        diagnostic: plan.diagnostic,
        command: command_path.display().to_string(),
    }
}

#[tauri::command]
pub fn start_local_playback(
    raw_text: String,
    request: Option<RemoteTargetRequest>,
    state: State<'_, PlaybackState>,
) -> PlaybackStatusResponse {
    let prepare = prepare_local_playback(raw_text.clone(), request.clone());
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
    let preview_dir = match create_playback_preview_dir() {
        Ok(preview_dir) => preview_dir,
        Err(error) => {
            return PlaybackStatusResponse {
                state: PlaybackProcessState::Error,
                pid: None,
                command: Some(command_path.display().to_string()),
                message: Some(error),
            };
        }
    };
    let plan = build_playback_plan(&raw_text, request.as_ref(), Some(&preview_dir));
    if !plan.playable {
        let _ = fs::remove_dir_all(preview_dir);
        return PlaybackStatusResponse {
            state: PlaybackProcessState::Error,
            pid: None,
            command: Some(command_path.display().to_string()),
            message: plan.diagnostic,
        };
    }
    let frame_sources = playback_frame_sources(&plan.streams, &preview_dir);
    let (frame_sources, mut preview_server) = if frame_sources.is_empty() {
        (frame_sources, None)
    } else {
        match start_mjpeg_preview_server(&frame_sources) {
            Ok(server) => {
                let frame_sources = with_mjpeg_stream_urls(&frame_sources, &server.base_url);
                (frame_sources, Some(server))
            }
            Err(_) => (frame_sources, None),
        }
    };
    let command_text = plan
        .launch_steps
        .iter()
        .map(|step| launch_step_display_command(&command_path, step))
        .collect::<Vec<_>>()
        .join("\n");
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

    if session.as_mut().is_some() {
        let existing_status = playback_session_status(&mut session);
        if existing_status.state == PlaybackProcessState::Playing {
            return PlaybackStatusResponse {
                state: PlaybackProcessState::Playing,
                pid: existing_status.pid,
                command: existing_status.command,
                message: Some("이미 실행 중인 Playback Pipeline이 있습니다.".to_string()),
            };
        }
    }

    let mut processes = Vec::new();
    for (index, step) in plan.launch_steps.iter().enumerate() {
        match spawn_playback_process(&command_path, step, index) {
            Ok(process) => processes.push(process),
            Err(error) => {
                for process in &mut processes {
                    let _ = kill_playback_process(process);
                }
                if let Some(server) = preview_server.take() {
                    drop(server);
                }
                let _ = fs::remove_dir_all(&preview_dir);
                return PlaybackStatusResponse {
                    state: PlaybackProcessState::Error,
                    pid: None,
                    command: Some(command_text),
                    message: Some(format!("Playback Pipeline 실행에 실패했습니다: {error}")),
                };
            }
        }
    }

    let pid = processes.first().map(playback_process_pid);
    if let Some(pid) = pid {
        *session = Some(PlaybackSession {
            command: command_text.clone(),
            frame_sources,
            pid,
            preview_dir: Some(preview_dir),
            preview_server,
            processes,
        });
        PlaybackStatusResponse {
            state: PlaybackProcessState::Playing,
            pid: Some(pid),
            command: Some(command_text),
            message: Some("Playback Pipeline을 실행했습니다.".to_string()),
        }
    } else {
        if let Some(server) = preview_server.take() {
            drop(server);
        }
        let _ = fs::remove_dir_all(preview_dir);
        PlaybackStatusResponse {
            state: PlaybackProcessState::Error,
            pid: None,
            command: Some(command_text),
            message: Some("실행할 Playback 프로세스가 없습니다.".to_string()),
        }
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
    let mut errors = Vec::new();
    let mut already_stopped = true;
    for process in &mut existing.processes {
        match playback_process_status(process) {
            Ok(Some(_)) => {}
            Ok(None) => {
                already_stopped = false;
                if let Err(error) = kill_playback_process(process) {
                    errors.push(format!("{}: {error}", playback_process_command(process)));
                }
            }
            Err(error) => errors.push(format!("{}: {error}", playback_process_command(process))),
        }
    }

    if errors.is_empty() {
        PlaybackStatusResponse {
            state: PlaybackProcessState::Stopped,
            pid: Some(pid),
            command: Some(command),
            message: Some(
                if already_stopped {
                    "Playback Pipeline이 이미 종료되었습니다."
                } else {
                    "Playback Pipeline을 정지했습니다."
                }
                .to_string(),
            ),
        }
    } else {
        PlaybackStatusResponse {
            state: PlaybackProcessState::Error,
            pid: Some(pid),
            command: Some(command),
            message: Some(format!(
                "Playback Pipeline 종료에 실패했습니다: {}",
                errors.join("; ")
            )),
        }
    }
}

fn spawn_playback_process(
    command_path: &Path,
    step: &PlaybackLaunchStep,
    index: usize,
) -> Result<PlaybackProcessHandle, String> {
    match step.location {
        PlaybackLocation::Local => spawn_local_playback_process(command_path, &step.pipeline),
        PlaybackLocation::Remote => {
            let Some(request) = step.request.as_ref() else {
                return Err("Remote Playback 요청 정보가 없습니다.".to_string());
            };
            spawn_remote_playback_process(request, &step.pipeline, index)
        }
    }
}

fn spawn_local_playback_process(
    command_path: &Path,
    pipeline: &str,
) -> Result<PlaybackProcessHandle, String> {
    let args = gst_launch_args(pipeline)?;
    let log_path = env::temp_dir().join(format!(
        "gst-topology-playback-local-{}-{}.log",
        std::process::id(),
        chrono_like_timestamp()
    ));
    let log_file = fs::File::create(&log_path).ok();
    let mut command = gstreamer_command(command_path);
    command
        .args(&args)
        .stdin(Stdio::null())
        .stdout(
            log_file
                .as_ref()
                .and_then(|file| file.try_clone().ok())
                .map(Stdio::from)
                .unwrap_or_else(Stdio::null),
        )
        .stderr(log_file.map(Stdio::from).unwrap_or_else(Stdio::null));
    let child = command
        .spawn()
        .map_err(|error| format!("Local gst-launch 실행 실패: {error}"))?;
    let pid = child.id();

    Ok(PlaybackProcessHandle::Local(PlaybackProcess {
        child,
        command: format!("{} {}", command_path.display(), args.join(" ")),
        log_path: Some(log_path),
        pid,
    }))
}

fn spawn_remote_playback_process(
    request: &RemoteTargetRequest,
    pipeline: &str,
    index: usize,
) -> Result<PlaybackProcessHandle, String> {
    let args = gst_launch_args(pipeline)?;
    let log_path = format!(
        "/tmp/gst-topology-playback-{}-{}-{index}.log",
        std::process::id(),
        chrono_like_timestamp()
    );
    let remote_args = args
        .iter()
        .map(|arg| shell_escape(arg))
        .collect::<Vec<_>>()
        .join(" ");
    let command = format!(
        "command -v gst-launch-1.0 >/dev/null 2>&1 || {{ echo 'gst-launch-1.0 is not available on remote target.' >&2; exit 127; }}; nohup gst-launch-1.0 {remote_args} > {} 2>&1 < /dev/null & echo $!",
        shell_escape(&log_path)
    );
    let mut session = connect_remote(request)?;
    let output = exec_remote_command_output(&mut session, &command)?;
    if output.exit_status != 0 {
        return Err(output
            .stderr
            .if_empty("Remote gst-launch background 실행에 실패했습니다."));
    }
    let pid = output
        .stdout
        .lines()
        .find_map(|line| line.trim().parse::<u32>().ok())
        .ok_or_else(|| "Remote gst-launch PID를 확인하지 못했습니다.".to_string())?;

    Ok(PlaybackProcessHandle::Remote(RemotePlaybackProcess {
        command: format!("remote:{} gst-launch-1.0 {remote_args}", request.host),
        log_path,
        pid,
        request: request.clone(),
    }))
}

fn launch_step_display_command(command_path: &Path, step: &PlaybackLaunchStep) -> String {
    match step.location {
        PlaybackLocation::Local => match gst_launch_args(&step.pipeline) {
            Ok(args) => format!("{} {}", command_path.display(), args.join(" ")),
            Err(_) => format!("{} {}", command_path.display(), step.pipeline),
        },
        PlaybackLocation::Remote => {
            let host = step
                .request
                .as_ref()
                .map(|request| request.host.as_str())
                .unwrap_or("remote");
            match gst_launch_args(&step.pipeline) {
                Ok(args) => format!("remote:{host} gst-launch-1.0 {}", args.join(" ")),
                Err(_) => format!("remote:{host} gst-launch-1.0 {}", step.pipeline),
            }
        }
    }
}

fn playback_process_pid(process: &PlaybackProcessHandle) -> u32 {
    match process {
        PlaybackProcessHandle::Local(process) => process.pid,
        PlaybackProcessHandle::Remote(process) => process.pid,
    }
}

fn playback_process_command(process: &PlaybackProcessHandle) -> &str {
    match process {
        PlaybackProcessHandle::Local(process) => &process.command,
        PlaybackProcessHandle::Remote(process) => &process.command,
    }
}

fn playback_process_status(process: &mut PlaybackProcessHandle) -> Result<Option<String>, String> {
    match process {
        PlaybackProcessHandle::Local(process) => match process.child.try_wait() {
            Ok(None) => Ok(None),
            Ok(Some(status)) => {
                let log_tail = read_local_playback_log(process).unwrap_or_default();
                Ok(Some(format!(
                    "Local PID {} 종료{}{}",
                    process.pid,
                    status
                        .code()
                        .map(|code| format!(" (exit {code})"))
                        .unwrap_or_default(),
                    if log_tail.trim().is_empty() {
                        String::new()
                    } else {
                        format!(": {}", summarize_playback_failure_hint(log_tail.trim()))
                    }
                )))
            }
            Err(error) => Err(error.to_string()),
        },
        PlaybackProcessHandle::Remote(process) => {
            let mut session = connect_remote(&process.request)?;
            let command = format!("kill -0 {} >/dev/null 2>&1", process.pid);
            let output = exec_remote_command_output(&mut session, &command)?;
            if output.exit_status == 0 {
                Ok(None)
            } else {
                let log_tail = read_remote_playback_log(process).unwrap_or_default();
                Ok(Some(format!(
                    "Remote PID {} 종료{}",
                    process.pid,
                    if log_tail.trim().is_empty() {
                        String::new()
                    } else {
                        format!(": {}", summarize_playback_failure_hint(log_tail.trim()))
                    }
                )))
            }
        }
    }
}

fn kill_playback_process(process: &mut PlaybackProcessHandle) -> Result<(), String> {
    match process {
        PlaybackProcessHandle::Local(process) => match process.child.try_wait() {
            Ok(Some(_)) => {
                remove_local_playback_log(process);
                Ok(())
            }
            Ok(None) => {
                process.child.kill().map_err(|error| error.to_string())?;
                process.child.wait().map_err(|error| error.to_string())?;
                remove_local_playback_log(process);
                Ok(())
            }
            Err(error) => Err(error.to_string()),
        },
        PlaybackProcessHandle::Remote(process) => {
            let mut session = connect_remote(&process.request)?;
            let command = format!(
                "kill {} >/dev/null 2>&1 || true; sleep 0.2; kill -9 {} >/dev/null 2>&1 || true; rm -f {} >/dev/null 2>&1 || true",
                process.pid,
                process.pid,
                shell_escape(&process.log_path)
            );
            let output = exec_remote_command_output(&mut session, &command)?;
            if output.exit_status == 0 {
                Ok(())
            } else {
                Err(output.stderr.if_empty("Remote Playback process 종료 실패."))
            }
        }
    }
}

fn read_local_playback_log(process: &PlaybackProcess) -> Result<String, String> {
    let Some(log_path) = &process.log_path else {
        return Ok(String::new());
    };
    let text = fs::read_to_string(log_path).map_err(|error| error.to_string())?;
    let mut lines = text
        .lines()
        .rev()
        .take(20)
        .map(str::to_string)
        .collect::<Vec<_>>();
    lines.reverse();

    Ok(lines.join("\n"))
}

fn summarize_playback_failure_hint(log_tail: &str) -> String {
    let lower = log_tail.to_ascii_lowercase();
    let compact = log_tail
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" | ");
    let compact = if compact.len() > 700 {
        format!("{}...", &compact[..700])
    } else {
        compact
    };

    if lower.contains("no element")
        || lower.contains("missing plugin")
        || lower.contains("could not link")
        || lower.contains("erroneous pipeline")
    {
        return format!(
            "로컬 GStreamer plugin/decoder 문제 가능성이 있습니다. H264 Preview는 rtph264depay, h264parse, decodebin이 사용할 H264 decoder, jpegenc가 필요합니다. 원본 로그: {compact}"
        );
    }

    compact
}

fn remove_local_playback_log(process: &PlaybackProcess) {
    if let Some(log_path) = &process.log_path {
        let _ = fs::remove_file(log_path);
    }
}

fn read_remote_playback_log(process: &RemotePlaybackProcess) -> Result<String, String> {
    let mut session = connect_remote(&process.request)?;
    let command = format!(
        "tail -20 {} 2>/dev/null || true",
        shell_escape(&process.log_path)
    );
    let output = exec_remote_command_output(&mut session, &command)?;

    Ok(output.stdout)
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

    playback_session_status(&mut session)
}

fn playback_session_status(session: &mut Option<PlaybackSession>) -> PlaybackStatusResponse {
    let Some(existing) = session.as_mut() else {
        return PlaybackStatusResponse {
            state: PlaybackProcessState::Idle,
            pid: None,
            command: None,
            message: Some("Playback Pipeline이 실행 중이 아닙니다.".to_string()),
        };
    };

    let mut running_count = 0usize;
    let mut stopped_messages = Vec::new();
    let mut errors = Vec::new();

    for process in &mut existing.processes {
        match playback_process_status(process) {
            Ok(None) => running_count += 1,
            Ok(Some(message)) => stopped_messages.push(message),
            Err(error) => errors.push(format!(
                "PID {} 상태 확인 실패: {error}",
                playback_process_pid(process)
            )),
        }
    }

    if !errors.is_empty() {
        let command = existing.command.clone();
        let pid = existing.pid;
        for process in &mut existing.processes {
            let _ = kill_playback_process(process);
        }
        *session = None;
        return PlaybackStatusResponse {
            state: PlaybackProcessState::Error,
            pid: Some(pid),
            command: Some(command),
            message: Some(errors.join("; ")),
        };
    }

    if running_count == existing.processes.len() {
        return PlaybackStatusResponse {
            state: PlaybackProcessState::Playing,
            pid: Some(existing.pid),
            command: Some(existing.command.clone()),
            message: Some("Playback Pipeline이 실행 중입니다.".to_string()),
        };
    }

    let command = existing.command.clone();
    let pid = existing.pid;
    for process in &mut existing.processes {
        let _ = kill_playback_process(process);
    }
    *session = None;

    let state = if running_count == 0 {
        PlaybackProcessState::Stopped
    } else {
        PlaybackProcessState::Error
    };
    let prefix = if running_count == 0 {
        "Playback Pipeline이 종료되었습니다"
    } else {
        "일부 Playback 프로세스가 종료되어 전체 재생을 중단했습니다"
    };

    PlaybackStatusResponse {
        state,
        pid: Some(pid),
        command: Some(command),
        message: Some(format!("{prefix}: {}", stopped_messages.join("; "))),
    }
}

#[tauri::command]
pub fn get_local_playback_frame(
    stream_id: String,
    state: State<'_, PlaybackState>,
) -> PlaybackFrameResponse {
    let session = match state.session.lock() {
        Ok(session) => session,
        Err(_) => {
            return PlaybackFrameResponse {
                stream_id,
                available: false,
                data_url: None,
                stream_url: None,
                updated_at_millis: None,
                diagnostic: Some("Playback 상태 잠금을 가져오지 못했습니다.".to_string()),
            };
        }
    };

    let Some(existing) = session.as_ref() else {
        return PlaybackFrameResponse {
            stream_id,
            available: false,
            data_url: None,
            stream_url: None,
            updated_at_millis: None,
            diagnostic: Some("Playback Pipeline이 실행 중이 아닙니다.".to_string()),
        };
    };

    let Some(frame_source) = existing
        .frame_sources
        .iter()
        .find(|source| source.stream_id == stream_id)
    else {
        return PlaybackFrameResponse {
            stream_id,
            available: false,
            data_url: None,
            stream_url: None,
            updated_at_millis: None,
            diagnostic: Some("이 스트림에는 App preview frame source가 없습니다.".to_string()),
        };
    };

    latest_playback_frame(frame_source)
}

fn detect_playback_streams(raw_text: &str) -> Vec<PlaybackStream> {
    let mut streams = Vec::new();

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
            .receive_address
            .as_deref()
            .map(|host| format!(" address={host}"))
            .unwrap_or_default();
        let normalized_caps = candidate
            .caps
            .as_deref()
            .map(normalize_rtp_caps_for_receiver);
        let caps_part = normalized_caps
            .as_deref()
            .map(|caps| format!(" caps=\"{caps}\""))
            .unwrap_or_default();
        let playback_chain = rtp_playback_chain(normalized_caps.as_deref(), &media_kind, sink);
        let playback_pipeline =
            format!("udpsrc{host_part} port={port}{caps_part} ! {playback_chain}");

        streams.push(PlaybackStream {
            id: format!("rtp-{stream_index}"),
            protocol: PlaybackProtocol::Rtp,
            direction: match candidate.direction {
                RtpDirection::Receiver => PlaybackDirection::Receiver,
                RtpDirection::Sender => PlaybackDirection::Sender,
            },
            media_kind,
            uri: None,
            host: candidate.host,
            port: Some(port),
            caps: normalized_caps,
            source: candidate.source,
            playback_pipeline,
        });
    }

    streams
}

fn rtp_playback_chain(
    caps: Option<&str>,
    media_kind: &PlaybackMediaKind,
    fallback_sink: &str,
) -> String {
    let caps_lower = caps.unwrap_or_default().to_ascii_lowercase();

    if rtp_caps_has_encoding(&caps_lower, "raw") {
        return "rtpvrawdepay ! videoconvert ! autovideosink".to_string();
    }
    if rtp_caps_has_encoding(&caps_lower, "jpeg") {
        return "rtpjpegdepay ! jpegdec ! videoconvert ! autovideosink".to_string();
    }
    if rtp_caps_has_encoding(&caps_lower, "h264") {
        return "rtph264depay ! h264parse ! decodebin ! videoconvert ! autovideosink".to_string();
    }
    if rtp_caps_has_encoding(&caps_lower, "h265") || rtp_caps_has_encoding(&caps_lower, "hevc") {
        return "rtph265depay ! h265parse ! decodebin ! videoconvert ! autovideosink".to_string();
    }
    if rtp_caps_has_encoding(&caps_lower, "opus") {
        return "rtpopusdepay ! opusdec ! audioconvert ! audioresample ! autoaudiosink".to_string();
    }
    if rtp_caps_has_encoding(&caps_lower, "mpeg4-generic")
        || caps_lower.contains("media=audio")
        || caps_lower.contains("media=(string)audio")
    {
        return "rtpmp4gdepay ! decodebin ! audioconvert ! audioresample ! autoaudiosink"
            .to_string();
    }

    let fallback_chain = match media_kind {
        PlaybackMediaKind::Audio => "decodebin ! autoaudiosink",
        PlaybackMediaKind::Unknown | PlaybackMediaKind::Video => "decodebin ! autovideosink",
    };

    if fallback_sink.is_empty() {
        fallback_chain.to_string()
    } else {
        fallback_chain.replace(
            match media_kind {
                PlaybackMediaKind::Audio => "autoaudiosink",
                PlaybackMediaKind::Unknown | PlaybackMediaKind::Video => "autovideosink",
            },
            fallback_sink,
        )
    }
}

fn preview_pipeline_for_stream(stream: &PlaybackStream, preview_dir: Option<&Path>) -> String {
    if !supports_frame_preview(stream) {
        return stream.playback_pipeline.clone();
    }

    let frame_location = preview_frame_location_value(preview_dir, &stream.id);
    match stream.protocol {
        PlaybackProtocol::Rtp => {
            let Some(port) = stream.port else {
                return stream.playback_pipeline.clone();
            };
            let caps_part = stream
                .caps
                .as_deref()
                .map(|caps| format!(" caps=\"{}\"", escape_gst_value(caps)))
                .unwrap_or_default();
            let preview_chain = rtp_frame_preview_chain(
                stream.caps.as_deref(),
                &stream.media_kind,
                &frame_location,
            );

            format!("udpsrc port={port}{caps_part} ! {preview_chain}")
        }
    }
}

fn supports_frame_preview(stream: &PlaybackStream) -> bool {
    matches!(
        stream.media_kind,
        PlaybackMediaKind::Unknown | PlaybackMediaKind::Video
    )
}

fn rtp_frame_preview_chain(
    caps: Option<&str>,
    media_kind: &PlaybackMediaKind,
    frame_location: &str,
) -> String {
    let caps_lower = caps.unwrap_or_default().to_ascii_lowercase();
    let sink_chain = frame_sink_chain(frame_location);

    if rtp_caps_has_encoding(&caps_lower, "raw") {
        return format!(
            "rtpvrawdepay ! videoconvert ! videorate ! video/x-raw,framerate=24/1 ! jpegenc quality=82 ! {sink_chain}"
        );
    }
    if rtp_caps_has_encoding(&caps_lower, "jpeg") {
        return format!(
            "rtpjpegdepay ! jpegdec ! videoconvert ! videorate ! video/x-raw,framerate=24/1 ! jpegenc quality=82 ! {sink_chain}"
        );
    }
    if rtp_caps_has_encoding(&caps_lower, "h264") {
        return format!(
            "rtph264depay ! h264parse ! decodebin ! videoconvert ! videorate ! video/x-raw,framerate=24/1 ! jpegenc quality=82 ! {sink_chain}"
        );
    }
    if rtp_caps_has_encoding(&caps_lower, "h265") || rtp_caps_has_encoding(&caps_lower, "hevc") {
        return format!(
            "rtph265depay ! h265parse ! decodebin ! videoconvert ! videorate ! video/x-raw,framerate=24/1 ! jpegenc quality=82 ! {sink_chain}"
        );
    }

    match media_kind {
        PlaybackMediaKind::Audio => "decodebin ! autoaudiosink".to_string(),
        PlaybackMediaKind::Unknown | PlaybackMediaKind::Video => {
            format!(
                "decodebin ! videoconvert ! videorate ! video/x-raw,framerate=24/1 ! jpegenc quality=82 ! {sink_chain}"
            )
        }
    }
}

fn rtp_caps_has_encoding(caps_lower: &str, encoding: &str) -> bool {
    caps_lower.contains(&format!("encoding-name={encoding}"))
        || caps_lower.contains(&format!("encoding-name=(string){encoding}"))
}

fn frame_sink_chain(frame_location: &str) -> String {
    format!(
        "multifilesink location=\"{}\" max-files=32",
        escape_gst_value(frame_location)
    )
}

fn preview_frame_location(preview_dir: &Path, stream_id: &str) -> PathBuf {
    preview_dir.join(format!("{}-%05d.jpg", safe_stream_file_prefix(stream_id)))
}

fn preview_frame_location_value(preview_dir: Option<&Path>, stream_id: &str) -> String {
    match preview_dir {
        Some(preview_dir) => gst_path_value(&preview_frame_location(preview_dir, stream_id)),
        None => format!(
            "/tmp/gst-topology-playback-preview/{}-%05d.jpg",
            safe_stream_file_prefix(stream_id)
        ),
    }
}

fn test_sender_pipeline_for_stream(
    stream: &PlaybackStream,
    request: Option<&RemoteTargetRequest>,
    preview_dir: Option<&Path>,
    index: usize,
) -> Option<String> {
    if !supports_frame_preview(stream) || !rtp_stream_is_raw_video(stream) {
        return None;
    }

    let port = stream.port?;
    let payload = rtp_caps_value(stream.caps.as_deref()?, "payload")
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(96);
    let width = rtp_caps_value(stream.caps.as_deref()?, "width")
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(640);
    let height = rtp_caps_value(stream.caps.as_deref()?, "height")
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(360);
    let framerate =
        rtp_caps_value(stream.caps.as_deref()?, "a-framerate").unwrap_or_else(|| "30".to_string());
    let target_host = generated_sender_target_host(stream, request);
    let pattern = if index % 2 == 0 { "smpte" } else { "ball" };
    let caps = stream.caps.as_deref()?;
    let frame_location = preview_frame_location_value(preview_dir, &stream.id);
    let source_caps = format!(
        "video/x-raw,format=RGB,width={width},height={height},framerate={}/1",
        framerate.trim_end_matches("/1")
    );

    Some(format!(
        "videotestsrc is-live=true pattern={pattern} ! {source_caps} ! videoconvert ! tee name=t t. ! queue ! videoconvert ! jpegenc ! {} t. ! queue ! rtpvrawpay pt={payload} ! {} ! udpsink host={} port={port}",
        frame_sink_chain(&frame_location),
        caps,
        escape_gst_value(&target_host)
    ))
}

fn generated_sender_target_host(
    stream: &PlaybackStream,
    request: Option<&RemoteTargetRequest>,
) -> String {
    if let Some(host) = stream
        .host
        .as_deref()
        .filter(|host| is_multicast_host(host))
    {
        return host.to_string();
    }

    request
        .map(|request| request.host.clone())
        .unwrap_or_else(|| "127.0.0.1".to_string())
}

fn is_multicast_host(host: &str) -> bool {
    let Some(first_octet) = host
        .split('.')
        .next()
        .and_then(|value| value.parse::<u8>().ok())
    else {
        return false;
    };

    (224..=239).contains(&first_octet)
}

fn rtp_stream_is_raw_video(stream: &PlaybackStream) -> bool {
    matches!(
        stream.media_kind,
        PlaybackMediaKind::Unknown | PlaybackMediaKind::Video
    ) && stream
        .caps
        .as_deref()
        .map(|caps| rtp_caps_has_encoding(&caps.to_ascii_lowercase(), "raw"))
        .unwrap_or(false)
}

fn rtp_caps_value(caps: &str, key: &str) -> Option<String> {
    caps.split(',').find_map(|field| {
        let (candidate_key, value) = field.trim().split_once('=')?;
        if candidate_key.trim() != key {
            return None;
        }

        Some(strip_caps_type(value.trim()).to_string())
    })
}

fn strip_caps_type(value: &str) -> &str {
    if let Some((_, rest)) = value.split_once(')') {
        rest
    } else {
        value
    }
}

fn safe_stream_file_prefix(stream_id: &str) -> String {
    stream_id
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '-' || character == '_' {
                character
            } else {
                '-'
            }
        })
        .collect()
}

fn normalize_rtp_caps_for_receiver(caps: &str) -> String {
    caps.split(',')
        .map(|field| {
            let trimmed = field.trim();
            if trimmed.is_empty() || trimmed.contains("=(") {
                return trimmed.to_string();
            }

            let Some((key, value)) = trimmed.split_once('=') else {
                return trimmed.to_string();
            };
            let key = key.trim();
            let value = value.trim();
            let caps_type = match key.to_ascii_lowercase().as_str() {
                "payload" | "clock-rate" => "(int)",
                "ssrc" | "timestamp-offset" | "seqnum-offset" => "(uint)",
                "media" | "encoding-name" | "sampling" | "depth" | "width" | "height"
                | "colorimetry" | "a-framerate" => "(string)",
                _ => "",
            };

            if caps_type.is_empty() {
                trimmed.to_string()
            } else {
                format!("{key}={caps_type}{value}")
            }
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn gst_path_value(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn escape_gst_value(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn create_playback_preview_dir() -> Result<PathBuf, String> {
    let preview_dir = env::temp_dir().join(format!(
        "gst-topology-playback-{}-{}",
        std::process::id(),
        chrono_like_timestamp()
    ));
    fs::create_dir_all(&preview_dir).map_err(|error| {
        format!(
            "Playback preview 폴더를 만들지 못했습니다 `{}`: {error}",
            preview_dir.display()
        )
    })?;

    Ok(preview_dir)
}

fn playback_frame_sources(
    streams: &[PlaybackStream],
    preview_dir: &Path,
) -> Vec<PlaybackFrameSource> {
    streams
        .iter()
        .filter(|stream| supports_frame_preview(stream))
        .map(|stream| PlaybackFrameSource {
            folder: preview_dir.to_path_buf(),
            stream_id: stream.id.clone(),
            stream_url: None,
        })
        .collect()
}

fn with_mjpeg_stream_urls(
    frame_sources: &[PlaybackFrameSource],
    base_url: &str,
) -> Vec<PlaybackFrameSource> {
    frame_sources
        .iter()
        .map(|source| PlaybackFrameSource {
            folder: source.folder.clone(),
            stream_id: source.stream_id.clone(),
            stream_url: Some(format!("{base_url}/{}.mjpeg", source.stream_id)),
        })
        .collect()
}

fn start_mjpeg_preview_server(
    frame_sources: &[PlaybackFrameSource],
) -> Result<MjpegPreviewServer, String> {
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .map_err(|error| format!("Preview MJPEG server bind 실패: {error}"))?;
    listener
        .set_nonblocking(true)
        .map_err(|error| format!("Preview MJPEG server nonblocking 설정 실패: {error}"))?;
    let port = listener
        .local_addr()
        .map_err(|error| format!("Preview MJPEG server 주소 확인 실패: {error}"))?
        .port();
    let stop = Arc::new(AtomicBool::new(false));
    let thread_stop = Arc::clone(&stop);
    let sources = frame_sources.to_vec();
    let handle = thread::spawn(move || {
        while !thread_stop.load(Ordering::Relaxed) {
            match listener.accept() {
                Ok((stream, _)) => {
                    let client_sources = sources.clone();
                    let client_stop = Arc::clone(&thread_stop);
                    thread::spawn(move || {
                        serve_mjpeg_client(stream, client_sources, client_stop);
                    });
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(25));
                }
                Err(_) => break,
            }
        }
    });

    Ok(MjpegPreviewServer {
        base_url: format!("http://127.0.0.1:{port}"),
        handle: Some(handle),
        stop,
    })
}

fn serve_mjpeg_client(
    mut stream: TcpStream,
    frame_sources: Vec<PlaybackFrameSource>,
    stop: Arc<AtomicBool>,
) {
    let _ = stream.set_read_timeout(Some(Duration::from_millis(500)));
    let mut buffer = [0u8; 1024];
    let read_len = stream.read(&mut buffer).unwrap_or(0);
    let request = String::from_utf8_lossy(&buffer[..read_len]);
    let stream_id = request
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|path| path.trim_start_matches('/').strip_suffix(".mjpeg"))
        .map(ToOwned::to_owned);
    let Some(stream_id) = stream_id else {
        let _ = stream.write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n");
        return;
    };
    let Some(frame_source) = frame_sources
        .into_iter()
        .find(|source| source.stream_id == stream_id)
    else {
        let _ = stream.write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n");
        return;
    };

    if stream
        .write_all(
            b"HTTP/1.1 200 OK\r\nContent-Type: multipart/x-mixed-replace; boundary=frame\r\nCache-Control: no-cache, no-store, must-revalidate\r\nPragma: no-cache\r\nConnection: close\r\nAccess-Control-Allow-Origin: *\r\n\r\n",
        )
        .is_err()
    {
        return;
    }

    let mut last_sent: Option<SystemTime> = None;
    while !stop.load(Ordering::Relaxed) {
        match latest_preview_frame_path(&frame_source) {
            Ok(Some((path, modified))) if last_sent.is_none_or(|previous| modified > previous) => {
                match fs::read(path) {
                    Ok(bytes) => {
                        let header = format!(
                            "--frame\r\nContent-Type: image/jpeg\r\nContent-Length: {}\r\n\r\n",
                            bytes.len()
                        );
                        if stream.write_all(header.as_bytes()).is_err()
                            || stream.write_all(&bytes).is_err()
                            || stream.write_all(b"\r\n").is_err()
                            || stream.flush().is_err()
                        {
                            return;
                        }
                        last_sent = Some(modified);
                    }
                    Err(_) => thread::sleep(Duration::from_millis(40)),
                }
            }
            _ => thread::sleep(Duration::from_millis(40)),
        }
    }
}

fn latest_preview_frame_path(
    frame_source: &PlaybackFrameSource,
) -> Result<Option<(PathBuf, SystemTime)>, String> {
    let prefix = safe_stream_file_prefix(&frame_source.stream_id);
    let entries = fs::read_dir(&frame_source.folder)
        .map_err(|error| format!("Preview frame 폴더를 읽지 못했습니다: {error}"))?;
    let mut latest: Option<(PathBuf, SystemTime)> = None;
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if !file_name.starts_with(&prefix) || !file_name.ends_with(".jpg") {
            continue;
        }
        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        let Ok(modified) = metadata.modified() else {
            continue;
        };
        if latest
            .as_ref()
            .is_none_or(|(_, current_modified)| modified > *current_modified)
        {
            latest = Some((path, modified));
        }
    }

    Ok(latest)
}

fn latest_playback_frame(frame_source: &PlaybackFrameSource) -> PlaybackFrameResponse {
    let latest = match latest_preview_frame_path(frame_source) {
        Ok(latest) => latest,
        Err(error) => {
            return PlaybackFrameResponse {
                stream_id: frame_source.stream_id.clone(),
                available: false,
                data_url: None,
                stream_url: frame_source.stream_url.clone(),
                updated_at_millis: None,
                diagnostic: Some(error),
            };
        }
    };

    let Some((path, modified)) = latest else {
        return PlaybackFrameResponse {
            stream_id: frame_source.stream_id.clone(),
            available: frame_source.stream_url.is_some(),
            data_url: None,
            stream_url: frame_source.stream_url.clone(),
            updated_at_millis: None,
            diagnostic: Some(
                "Playback 프로세스는 실행 중입니다. 첫 preview frame을 기다리는 중입니다."
                    .to_string(),
            ),
        };
    };

    let updated_at_millis = modified
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|duration| u64::try_from(duration.as_millis()).ok());
    if frame_source.stream_url.is_some() {
        return PlaybackFrameResponse {
            stream_id: frame_source.stream_id.clone(),
            available: true,
            data_url: None,
            stream_url: frame_source.stream_url.clone(),
            updated_at_millis,
            diagnostic: None,
        };
    }

    let bytes = match fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) => {
            return PlaybackFrameResponse {
                stream_id: frame_source.stream_id.clone(),
                available: false,
                data_url: None,
                stream_url: None,
                updated_at_millis: None,
                diagnostic: Some(format!("Preview frame을 읽지 못했습니다: {error}")),
            };
        }
    };

    PlaybackFrameResponse {
        stream_id: frame_source.stream_id.clone(),
        available: true,
        data_url: Some(format!(
            "data:image/jpeg;base64,{}",
            general_purpose::STANDARD.encode(bytes)
        )),
        stream_url: None,
        updated_at_millis,
        diagnostic: None,
    }
}

struct RtpCandidate {
    caps: Option<String>,
    direction: RtpDirection,
    host: Option<String>,
    port: Option<u16>,
    receive_address: Option<String>,
    source: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RtpDirection {
    Receiver,
    Sender,
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
            direction: RtpDirection::Receiver,
            receive_address: host.clone(),
            host,
            port,
            source,
        });
        index = cursor.saturating_add(1);
    }

    let mut index = 0;
    while index < tokens.len() {
        if tokens[index] != "udpsink" {
            index += 1;
            continue;
        }

        let mut sink_tokens = vec![tokens[index].clone()];
        let mut cursor = index + 1;
        while cursor < tokens.len() && tokens[cursor] != "!" {
            sink_tokens.push(tokens[cursor].clone());
            cursor += 1;
        }

        let mut host = None;
        let mut port = None;
        for token in &sink_tokens {
            if let Some(value) = parse_token_value(token, "host")
                .or_else(|| parse_token_value(token, "address"))
                .or_else(|| parse_token_value(token, "multicast-group"))
            {
                host = Some(value);
            }
            if let Some(value) = parse_token_value(token, "port") {
                port = value.parse::<u16>().ok();
            }
        }

        let Some(caps) = find_nearest_rtp_caps_before(&tokens, index)
            .or_else(|| infer_rtp_caps_from_payloader_before(&tokens, index))
        else {
            index = cursor.saturating_add(1);
            continue;
        };

        let start = index.saturating_sub(12);
        candidates.push(RtpCandidate {
            caps: Some(caps),
            direction: RtpDirection::Sender,
            host,
            port,
            // Sender PLDs publish to a remote/local host. The playback side
            // should bind the UDP port locally instead of reusing that host.
            receive_address: None,
            source: tokens[start..cursor].join(" "),
        });
        index = cursor.saturating_add(1);
    }

    candidates
}

fn find_nearest_rtp_caps_before(tokens: &[String], index: usize) -> Option<String> {
    let mut cursor = index;
    while cursor > 0 {
        cursor -= 1;
        let token = &tokens[cursor];
        if token == "udpsrc" || token == "udpsink" || token == "rtspsrc" {
            return None;
        }
        if token.contains("application/x-rtp") {
            return Some(token.trim_matches('"').to_string());
        }
    }

    None
}

fn infer_rtp_caps_from_payloader_before(tokens: &[String], index: usize) -> Option<String> {
    let mut cursor = index;
    while cursor > 0 {
        cursor -= 1;
        let token = &tokens[cursor];
        if token == "udpsrc" || token == "udpsink" || token == "rtspsrc" {
            return None;
        }
        let factory = token.trim();
        let payload = find_payloader_payload(tokens, cursor, index).unwrap_or_else(|| "96".into());
        let caps = match factory {
            "rtph264pay" => {
                format!(
                    "application/x-rtp,media=video,encoding-name=H264,payload={payload},clock-rate=90000"
                )
            }
            "rtph265pay" | "rtphevcpay" => {
                format!(
                    "application/x-rtp,media=video,encoding-name=H265,payload={payload},clock-rate=90000"
                )
            }
            "rtpjpegpay" => {
                format!(
                    "application/x-rtp,media=video,encoding-name=JPEG,payload={payload},clock-rate=90000"
                )
            }
            "rtpvrawpay" => {
                format!(
                    "application/x-rtp,media=video,encoding-name=RAW,payload={payload},clock-rate=90000"
                )
            }
            "rtpopuspay" => {
                format!(
                    "application/x-rtp,media=audio,encoding-name=OPUS,payload={payload},clock-rate=48000"
                )
            }
            "rtpmp4gpay" => {
                format!(
                    "application/x-rtp,media=audio,encoding-name=MPEG4-GENERIC,payload={payload},clock-rate=48000"
                )
            }
            _ => continue,
        };

        return Some(caps);
    }

    None
}

fn find_payloader_payload(
    tokens: &[String],
    payloader_index: usize,
    sink_index: usize,
) -> Option<String> {
    let mut cursor = payloader_index + 1;
    while cursor < sink_index && tokens[cursor] != "!" {
        if let Some(value) = parse_token_value(&tokens[cursor], "pt")
            .or_else(|| parse_token_value(&tokens[cursor], "payload"))
        {
            return Some(value);
        }
        cursor += 1;
    }

    None
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
    fn playback_detection_ignores_rtsp_for_rtp_only_slice() {
        let streams = detect_playback_streams(
            "rtspsrc location=rtsp://192.168.0.10:8554/camera ! rtph264depay ! fakesink",
        );

        assert!(streams.is_empty());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn gstreamer_command_candidates_prefer_homebrew_before_conda() {
        let candidates = gstreamer_command_candidates("gst-inspect-1.0");
        let homebrew_index = candidates
            .iter()
            .position(|candidate| candidate == Path::new("/opt/homebrew/bin/gst-inspect-1.0"))
            .expect("Homebrew candidate should be included on macOS");
        let conda_index = candidates
            .iter()
            .position(|candidate| candidate.ends_with("anaconda3/bin/gst-inspect-1.0"))
            .expect("Anaconda candidate should be included for fallback");

        assert!(homebrew_index < conda_index);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn gstreamer_plugin_scanner_resolves_from_homebrew_symlink() {
        let scanner = gstreamer_plugin_scanner_for(Path::new("/opt/homebrew/bin/gst-inspect-1.0"));
        if Path::new("/opt/homebrew/bin/gst-inspect-1.0").exists() {
            assert!(scanner
                .as_ref()
                .is_some_and(|path| path.ends_with("libexec/gstreamer-1.0/gst-plugin-scanner")));
        }
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
        assert_eq!(streams[0].direction, PlaybackDirection::Receiver);
        assert_eq!(streams[0].media_kind, PlaybackMediaKind::Video);
        assert_eq!(streams[0].host.as_deref(), Some("239.0.0.1"));
        assert_eq!(streams[0].port, Some(5004));
        assert!(streams[0]
            .playback_pipeline
            .contains("caps=\"application/x-rtp,media=(string)video"));
        assert!(streams[0].playback_pipeline.contains("payload=(int)96"));
    }

    #[test]
    fn playback_detection_accepts_rtp_udp_sender_caps() {
        let streams = detect_playback_streams(
            "videotestsrc is-live=true ! videoconvert ! rtpvrawpay pt=96 ! application/x-rtp,media=video,encoding-name=RAW,payload=96,clock-rate=90000 ! udpsink host=127.0.0.1 port=5004",
        );

        assert_eq!(streams.len(), 1);
        assert_eq!(streams[0].protocol, PlaybackProtocol::Rtp);
        assert_eq!(streams[0].direction, PlaybackDirection::Sender);
        assert_eq!(streams[0].media_kind, PlaybackMediaKind::Video);
        assert_eq!(streams[0].host.as_deref(), Some("127.0.0.1"));
        assert_eq!(streams[0].port, Some(5004));
        assert!(streams[0].playback_pipeline.starts_with("udpsrc port=5004"));
        assert!(streams[0]
            .playback_pipeline
            .contains("caps=\"application/x-rtp,media=(string)video"));
        assert!(streams[0].playback_pipeline.contains("payload=(int)96"));
        assert!(streams[0].playback_pipeline.contains("rtpvrawdepay"));
        assert!(!streams[0]
            .playback_pipeline
            .contains("! decodebin ! autovideosink"));
    }

    #[test]
    fn playback_detection_infers_h264_caps_from_sender_payloader() {
        let raw_text = "qtiqmmfsrc camera=0 name=eocam0 ! video/x-raw(memory:GBM),format=NV12,framerate=30/1,width=1920,height=1080 ! qtic2venc name=eo_venc control-rate=2 idr-interval=60 target-bitrate=3000000 ! h264parse config-interval=-1 ! tee name=eoenc ! rtph264pay mtu=1350 config-interval=1 name=pay0 pt=96 ! udpsink host=192.168.100.112 port=15000";
        let streams = detect_playback_streams(raw_text);

        assert_eq!(streams.len(), 1);
        assert_eq!(streams[0].direction, PlaybackDirection::Sender);
        assert_eq!(streams[0].media_kind, PlaybackMediaKind::Video);
        assert_eq!(streams[0].port, Some(15000));
        assert!(streams[0]
            .caps
            .as_deref()
            .is_some_and(|caps| caps.contains("encoding-name=(string)H264")));
        assert!(streams[0]
            .caps
            .as_deref()
            .is_some_and(|caps| caps.contains("payload=(int)96")));

        let plan = build_playback_plan(raw_text, None, Some(Path::new("/tmp/gst-preview")));
        assert!(plan.playable);
        assert!(plan
            .counterpart_pipeline
            .as_deref()
            .is_some_and(|pipeline| pipeline.contains("rtph264depay ! h264parse ! decodebin")));
        assert!(plan
            .counterpart_pipeline
            .as_deref()
            .is_some_and(|pipeline| pipeline.contains("framerate=24/1")));
    }

    #[test]
    fn rtp_playback_chain_uses_depayloader_for_known_encodings() {
        assert_eq!(
            rtp_playback_chain(
                Some("application/x-rtp,media=video,encoding-name=RAW,payload=96"),
                &PlaybackMediaKind::Video,
                "autovideosink"
            ),
            "rtpvrawdepay ! videoconvert ! autovideosink"
        );
        assert_eq!(
            rtp_playback_chain(
                Some("application/x-rtp,media=video,encoding-name=JPEG,payload=96"),
                &PlaybackMediaKind::Video,
                "autovideosink"
            ),
            "rtpjpegdepay ! jpegdec ! videoconvert ! autovideosink"
        );
        assert!(rtp_playback_chain(
            Some("application/x-rtp,media=video,encoding-name=H264,payload=96"),
            &PlaybackMediaKind::Video,
            "autovideosink"
        )
        .starts_with("rtph264depay"));
    }

    #[test]
    fn rtp_caps_are_normalized_for_receiver_preview() {
        assert_eq!(
            normalize_rtp_caps_for_receiver(
                "application/x-rtp,media=video,encoding-name=RAW,payload=96,clock-rate=90000"
            ),
            "application/x-rtp,media=(string)video,encoding-name=(string)RAW,payload=(int)96,clock-rate=(int)90000"
        );

        assert_eq!(
            normalize_rtp_caps_for_receiver(
                "application/x-rtp,media=(string)video,payload=(int)96"
            ),
            "application/x-rtp,media=(string)video,payload=(int)96"
        );
    }

    #[test]
    fn playback_plan_generates_test_sender_for_receiver_pld() {
        let raw_text = "udpsrc port=5008 caps=\"application/x-rtp,media=video,encoding-name=RAW,payload=98,clock-rate=90000,width=640,height=360\" ! rtpvrawdepay ! fakesink";
        let plan = build_playback_plan(raw_text, None, Some(Path::new("/tmp/gst-preview")));

        assert!(plan.playable);
        assert_eq!(plan.source_role, PlaybackSourceRole::Receiver);
        assert!(plan
            .counterpart_pipeline
            .as_deref()
            .is_some_and(|pipeline| pipeline.contains("videotestsrc is-live=true")));
        assert!(plan
            .counterpart_pipeline
            .as_deref()
            .is_some_and(|pipeline| pipeline.contains("udpsink host=127.0.0.1 port=5008")));
        assert!(plan
            .counterpart_pipeline
            .as_deref()
            .is_some_and(|pipeline| pipeline.contains("/tmp/gst-preview/rtp-1-%05d.jpg")));
    }

    #[test]
    fn playback_plan_sends_local_counterpart_to_remote_receiver() {
        let raw_text = "udpsrc port=5008 caps=\"application/x-rtp,media=video,encoding-name=RAW,payload=98,clock-rate=90000,width=640,height=360\" ! rtpvrawdepay ! fakesink";
        let request = RemoteTargetRequest {
            host: "192.168.0.55".to_string(),
            port: Some(22),
            username: "root".to_string(),
            password: "pw".to_string(),
        };
        let plan = build_playback_plan(
            raw_text,
            Some(&request),
            Some(Path::new("/tmp/gst-preview")),
        );

        assert!(plan.playable);
        assert_eq!(plan.source_location, PlaybackLocation::Remote);
        assert!(plan
            .counterpart_pipeline
            .as_deref()
            .is_some_and(|pipeline| pipeline.contains("udpsink host=192.168.0.55 port=5008")));
    }

    #[test]
    fn playback_plan_remaps_local_sender_ports_for_preview() {
        let raw_text = "videotestsrc is-live=true ! rtpvrawpay pt=96 ! application/x-rtp,media=video,encoding-name=RAW,payload=96,clock-rate=90000 ! udpsink host=127.0.0.1 port=5004";
        let plan = build_playback_plan(raw_text, None, Some(Path::new("/tmp/gst-preview")));

        assert!(plan.playable);
        assert_eq!(plan.source_role, PlaybackSourceRole::Sender);
        assert!(plan
            .counterpart_pipeline
            .as_deref()
            .is_some_and(|pipeline| pipeline.contains("udpsrc port=17000")));
        assert!(plan
            .generated_pipeline
            .as_deref()
            .is_some_and(|pipeline| pipeline.contains("udpsink host=127.0.0.1 port=17000")));
        assert!(!plan
            .generated_pipeline
            .as_deref()
            .unwrap_or_default()
            .contains("udpsrc port=5004"));
    }

    #[test]
    fn preview_pipeline_for_rtp_stream_writes_app_frames() {
        let stream = PlaybackStream {
            id: "rtp-1".to_string(),
            protocol: PlaybackProtocol::Rtp,
            direction: PlaybackDirection::Sender,
            media_kind: PlaybackMediaKind::Video,
            uri: None,
            host: Some("127.0.0.1".to_string()),
            port: Some(5004),
            caps: Some(
                "application/x-rtp,media=video,encoding-name=RAW,payload=96,clock-rate=90000"
                    .to_string(),
            ),
            source: "udpsink host=127.0.0.1 port=5004".to_string(),
            playback_pipeline: "udpsrc port=5004 ! autovideosink".to_string(),
        };
        let pipeline = preview_pipeline_for_stream(&stream, Some(Path::new("/tmp/gst-preview")));

        assert!(pipeline.contains("rtpvrawdepay"));
        assert!(pipeline.contains("jpegenc"));
        assert!(pipeline.contains("multifilesink"));
        assert!(pipeline.contains("/tmp/gst-preview/rtp-1-%05d.jpg"));
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
    }

    #[test]
    fn playback_detection_does_not_accept_shell_injection_suffix() {
        let streams = detect_playback_streams(
            "videotestsrc ! rtpvrawpay pt=96 ! application/x-rtp,media=video,encoding-name=RAW,payload=96,clock-rate=90000 ! udpsink host=127.0.0.1 port=5004;touch /tmp/owned",
        );

        assert!(streams.is_empty());
    }
}
