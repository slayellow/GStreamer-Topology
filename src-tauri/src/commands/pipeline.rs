use std::env;
use std::fs;
use std::io::Read;
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use base64::{engine::general_purpose, Engine as _};
use ssh2::Session;

use crate::models::{
    ElementMetadataResponse, ElementPadTemplateMetadata, ElementPropertyMetadata,
    GStreamerProbeResponse, MetadataAuthority, PipelineDocument, RemoteProbeResponse,
    RemoteTargetRequest, SourceKind,
};
use crate::parser::{normalize_text, parse_document};

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

fn push_unique_candidate(candidates: &mut Vec<PathBuf>, path: PathBuf) {
    if !candidates.iter().any(|candidate| candidate == &path) {
        candidates.push(path);
    }
}

fn gst_inspect_command_candidates() -> Vec<PathBuf> {
    let executable_name = gst_inspect_executable_name();
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
        match Command::new(candidate).arg("--version").output() {
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

fn run_gst_inspect(args: &[&str]) -> Result<(PathBuf, Output), String> {
    let command_path = resolve_gst_inspect_command()?;
    let output = Command::new(&command_path)
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

    if exit_status != 0 {
        return Err(format!(
            "remote command `{command}` failed with status {exit_status}: {}",
            stderr.trim()
        ));
    }

    Ok(stdout)
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

    for line in raw_output.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        match trimmed {
            "Factory Details:" => {
                flush_pad_template(&mut metadata.pad_templates, &mut current_pad);
                section = "factory";
                continue;
            }
            "Plugin Details:" => {
                flush_pad_template(&mut metadata.pad_templates, &mut current_pad);
                section = "plugin";
                continue;
            }
            "Pad Templates:" => {
                section = "pads";
                continue;
            }
            "Element Properties:" => {
                flush_pad_template(&mut metadata.pad_templates, &mut current_pad);
                section = "properties";
                continue;
            }
            "Element Signals:"
            | "Element Actions:"
            | "Children:"
            | "Pads:"
            | "Clocking Interaction:" => {
                flush_pad_template(&mut metadata.pad_templates, &mut current_pad);
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
                    current_pad = Some(ElementPadTemplateMetadata {
                        direction,
                        name,
                        presence: None,
                    });
                } else if let Some(value) = parse_gst_field(trimmed, "Availability") {
                    if let Some(pad) = &mut current_pad {
                        pad.presence = Some(value);
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
}
