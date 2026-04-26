use std::fs;
use std::io::Read;
use std::net::TcpStream;
use std::path::Path;
use std::process::Command;

use ssh2::Session;

use crate::models::{
    ElementMetadataResponse, ElementPadTemplateMetadata, ElementPropertyMetadata,
    GStreamerProbeResponse, MetadataAuthority, NormalizationResult, PipelineDocument,
    RemoteProbeResponse, RemoteTargetRequest, SourceKind,
};
use crate::parser::{normalize_text, parse_document};

#[tauri::command]
pub fn normalize_rtf_text(raw_text: String) -> NormalizationResult {
    normalize_text(&raw_text)
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
pub fn probe_local_gstreamer() -> GStreamerProbeResponse {
    match Command::new("gst-inspect-1.0").arg("--version").output() {
        Ok(output) if output.status.success() => GStreamerProbeResponse {
            available: true,
            authority: MetadataAuthority::Local,
            version_output: Some(String::from_utf8_lossy(&output.stdout).trim().to_string()),
            diagnostic: None,
        },
        Ok(output) => GStreamerProbeResponse {
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
            diagnostic: Some(format!("gst-inspect-1.0 is not available: {error}")),
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

    match Command::new("gst-inspect-1.0").arg(&factory_name).output() {
        Ok(output) if output.status.success() => {
            let raw_output = String::from_utf8_lossy(&output.stdout).into_owned();
            parse_gst_inspect_output(MetadataAuthority::Local, factory_name, raw_output)
        }
        Ok(output) => unavailable_element_metadata(
            MetadataAuthority::Local,
            factory_name,
            String::from_utf8_lossy(&output.stderr)
                .trim()
                .to_string()
                .if_empty("gst-inspect-1.0 could not inspect this element."),
        ),
        Err(error) => unavailable_element_metadata(
            MetadataAuthority::Local,
            factory_name,
            format!("gst-inspect-1.0 is not available: {error}"),
        ),
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
        .map(str::trim)
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
        || name.contains(char::is_whitespace)
        || matches!(name, "flags" | "Enum" | "Default")
    {
        return None;
    }

    Some(ElementPropertyMetadata {
        name: name.to_string(),
        description: Some(description.trim().to_string()).filter(|value| !value.is_empty()),
    })
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
