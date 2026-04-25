use std::fs;
use std::io::Read;
use std::net::TcpStream;
use std::path::Path;

use ssh2::Session;

use crate::models::{
    NormalizationResult, PipelineDocument, RemoteProbeResponse, RemoteTargetRequest, SourceKind,
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
