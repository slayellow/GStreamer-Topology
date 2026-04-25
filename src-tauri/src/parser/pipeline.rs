use std::collections::HashMap;

use crate::models::{
    ParseDiagnostic, PipelineDocument, PipelineEdge, PipelineGraph, PipelineNode, PipelineNodeKind,
    PipelinePort, PipelinePortKind, PipelineProperty, SourceKind, SourceSpan,
};

#[derive(Debug, Clone)]
enum ChainItem {
    Node(NodeEndpoint),
    Reference(ReferenceEndpoint),
    Caps(CapsSegment),
}

#[derive(Debug, Clone)]
struct NodeEndpoint {
    node_id: String,
    span: SourceSpan,
}

#[derive(Debug, Clone)]
struct ReferenceEndpoint {
    name: String,
    pad: Option<String>,
    span: SourceSpan,
}

#[derive(Debug, Clone)]
struct CapsSegment {
    label: String,
    span: SourceSpan,
}

#[derive(Debug, Clone)]
struct PendingEdge {
    from: Endpoint,
    to: Endpoint,
    caps_label: Option<String>,
    span: SourceSpan,
}

#[derive(Debug, Clone)]
enum Endpoint {
    Node(NodeEndpoint),
    Reference(ReferenceEndpoint),
}

#[derive(Debug)]
struct ParsedElement {
    factory_name: String,
    instance_name: Option<String>,
    properties: Vec<PipelineProperty>,
    loose_tokens: Vec<String>,
}

pub fn parse_document(
    raw_text: String,
    normalized_text: String,
    source_kind: SourceKind,
    path: Option<String>,
    source_name: Option<String>,
    normalization_diagnostics: Vec<ParseDiagnostic>,
) -> PipelineDocument {
    let (graph, mut parser_diagnostics) = parse_graph(&normalized_text);
    let mut diagnostics = normalization_diagnostics;
    diagnostics.append(&mut parser_diagnostics);

    PipelineDocument {
        source_kind,
        path,
        source_name,
        raw_text,
        normalized_text,
        diagnostics,
        graph,
    }
}

pub fn parse_graph(normalized_text: &str) -> (PipelineGraph, Vec<ParseDiagnostic>) {
    let mut graph = PipelineGraph::default();
    let mut diagnostics = Vec::new();
    let mut instance_map = HashMap::new();
    let mut pending_edges = Vec::new();

    for statement in split_statements(normalized_text) {
        let mut items = Vec::new();
        for component in split_components(normalized_text, &statement) {
            if component.1.trim().is_empty() {
                continue;
            }

            if let Some(reference) = parse_reference_component(component.1, component.0.clone()) {
                items.push(ChainItem::Reference(reference));
                continue;
            }

            if let Some(caps) = parse_caps_component(component.1, component.0.clone()) {
                items.push(ChainItem::Caps(caps));
                continue;
            }

            match parse_element_component(component.1) {
                Some(element) => {
                    let node_id = format!("node-{}", graph.nodes.len());
                    if let Some(instance_name) = &element.instance_name {
                        if instance_map
                            .insert(instance_name.clone(), node_id.clone())
                            .is_some()
                        {
                            diagnostics.push(ParseDiagnostic::warning(
                                "duplicate-instance-name",
                                format!(
                                    "The named element `{instance_name}` was declared more than once."
                                ),
                                Some(component.0.clone()),
                            ));
                        }
                    }

                    if !element.loose_tokens.is_empty() {
                        diagnostics.push(ParseDiagnostic::warning(
                            "unparsed-element-token",
                            format!(
                                "Ignored tokens on element `{}`: {}",
                                element.factory_name,
                                element.loose_tokens.join(", ")
                            ),
                            Some(component.0.clone()),
                        ));
                    }

                    graph.nodes.push(PipelineNode {
                        id: node_id.clone(),
                        factory_name: element.factory_name,
                        instance_name: element.instance_name,
                        kind: PipelineNodeKind::Element,
                        properties: element.properties,
                        source_span: component.0.clone(),
                    });
                    items.push(ChainItem::Node(NodeEndpoint {
                        node_id,
                        span: component.0,
                    }));
                }
                None => diagnostics.push(ParseDiagnostic::warning(
                    "empty-component",
                    "Encountered an empty pipeline component.",
                    Some(component.0),
                )),
            }
        }

        pending_edges.extend(build_pending_edges(items, &mut diagnostics));
    }

    resolve_pending_edges(&mut graph, &instance_map, pending_edges, &mut diagnostics);

    (graph, diagnostics)
}

fn split_statements(text: &str) -> Vec<SourceSpan> {
    let mut statements = Vec::new();
    let mut current_start = None;
    let mut offset = 0usize;

    for line in text.split_inclusive('\n') {
        let line_start = offset;
        let line_end = offset + line.len();
        if line.trim().is_empty() {
            if let Some(start) = current_start.take() {
                statements.push(SourceSpan::new(start, line_start));
            }
        } else if current_start.is_none() {
            current_start = Some(line_start);
        }
        offset = line_end;
    }

    if let Some(start) = current_start {
        statements.push(SourceSpan::new(start, text.len()));
    }

    statements
}

fn split_components<'a>(text: &'a str, statement: &SourceSpan) -> Vec<(SourceSpan, &'a str)> {
    let mut components = Vec::new();
    let source = &text[statement.start..statement.end];
    let mut segment_start = 0usize;
    let mut in_quotes = false;
    let mut angle_depth = 0usize;
    let mut paren_depth = 0usize;
    let mut brace_depth = 0usize;

    for (index, ch) in source.char_indices() {
        match ch {
            '"' => in_quotes = !in_quotes,
            '<' if !in_quotes => angle_depth += 1,
            '>' if !in_quotes && angle_depth > 0 => angle_depth -= 1,
            '(' if !in_quotes => paren_depth += 1,
            ')' if !in_quotes && paren_depth > 0 => paren_depth -= 1,
            '{' if !in_quotes => brace_depth += 1,
            '}' if !in_quotes && brace_depth > 0 => brace_depth -= 1,
            '!' if !in_quotes && angle_depth == 0 && paren_depth == 0 && brace_depth == 0 => {
                if let Some(component) = trim_slice(
                    text,
                    statement.start + segment_start,
                    statement.start + index,
                ) {
                    components.push(component);
                }
                segment_start = index + ch.len_utf8();
            }
            _ => {}
        }
    }

    if let Some(component) = trim_slice(text, statement.start + segment_start, statement.end) {
        components.push(component);
    }

    components
}

fn trim_slice(text: &str, start: usize, end: usize) -> Option<(SourceSpan, &str)> {
    if start >= end {
        return None;
    }

    let slice = &text[start..end];
    let leading = slice.len() - slice.trim_start().len();
    let trailing = slice.len() - slice.trim_end().len();
    let trimmed_start = start + leading;
    let trimmed_end = end.saturating_sub(trailing);

    if trimmed_start >= trimmed_end {
        return None;
    }

    Some((
        SourceSpan::new(trimmed_start, trimmed_end),
        &text[trimmed_start..trimmed_end],
    ))
}

fn parse_reference_component(text: &str, span: SourceSpan) -> Option<ReferenceEndpoint> {
    let candidate = text.trim();
    if candidate.is_empty() || candidate.contains(char::is_whitespace) || candidate.contains('=') {
        return None;
    }

    if let Some(name) = candidate.strip_suffix('.') {
        if !name.is_empty() {
            return Some(ReferenceEndpoint {
                name: name.to_string(),
                pad: None,
                span,
            });
        }
    }

    let (name, pad) = candidate.split_once('.')?;
    if name.is_empty() || pad.is_empty() {
        return None;
    }

    Some(ReferenceEndpoint {
        name: name.to_string(),
        pad: Some(pad.to_string()),
        span,
    })
}

fn parse_caps_component(text: &str, span: SourceSpan) -> Option<CapsSegment> {
    let candidate = text.trim();
    if candidate.is_empty() || candidate.contains(char::is_whitespace) {
        return None;
    }

    if !candidate.contains('/') {
        return None;
    }

    Some(CapsSegment {
        label: candidate.to_string(),
        span,
    })
}

fn parse_element_component(text: &str) -> Option<ParsedElement> {
    let tokens = split_whitespace_tokens(text);
    let (factory_span, factory_name) = tokens.first()?.clone();
    let _ = factory_span;

    let mut properties = Vec::new();
    let mut loose_tokens = Vec::new();
    let mut instance_name = None;

    for (_, token) in tokens.into_iter().skip(1) {
        if let Some((key, value)) = token.split_once('=') {
            let property = PipelineProperty {
                key: key.to_string(),
                value: value.to_string(),
            };
            if property.key == "name" {
                instance_name = Some(property.value.clone());
            }
            properties.push(property);
        } else {
            loose_tokens.push(token);
        }
    }

    Some(ParsedElement {
        factory_name,
        instance_name,
        properties,
        loose_tokens,
    })
}

fn split_whitespace_tokens(text: &str) -> Vec<(SourceSpan, String)> {
    let mut tokens = Vec::new();
    let mut token_start = None;
    let mut in_quotes = false;
    let mut angle_depth = 0usize;
    let mut paren_depth = 0usize;
    let mut brace_depth = 0usize;

    for (index, ch) in text.char_indices() {
        match ch {
            '"' => {
                if token_start.is_none() {
                    token_start = Some(index);
                }
                in_quotes = !in_quotes;
            }
            '<' if !in_quotes => {
                token_start.get_or_insert(index);
                angle_depth += 1;
            }
            '>' if !in_quotes && angle_depth > 0 => angle_depth -= 1,
            '(' if !in_quotes => {
                token_start.get_or_insert(index);
                paren_depth += 1;
            }
            ')' if !in_quotes && paren_depth > 0 => paren_depth -= 1,
            '{' if !in_quotes => {
                token_start.get_or_insert(index);
                brace_depth += 1;
            }
            '}' if !in_quotes && brace_depth > 0 => brace_depth -= 1,
            whitespace
                if whitespace.is_whitespace()
                    && !in_quotes
                    && angle_depth == 0
                    && paren_depth == 0
                    && brace_depth == 0 =>
            {
                if let Some(start) = token_start.take() {
                    tokens.push((
                        SourceSpan::new(start, index),
                        text[start..index].to_string(),
                    ));
                }
            }
            _ => {
                token_start.get_or_insert(index);
            }
        }
    }

    if let Some(start) = token_start {
        tokens.push((
            SourceSpan::new(start, text.len()),
            text[start..].to_string(),
        ));
    }

    tokens
}

fn build_pending_edges(
    items: Vec<ChainItem>,
    diagnostics: &mut Vec<ParseDiagnostic>,
) -> Vec<PendingEdge> {
    let mut pending_edges = Vec::new();
    let mut previous: Option<Endpoint> = None;
    let mut caps = Vec::new();
    let mut caps_span: Option<SourceSpan> = None;

    for item in items {
        match item {
            ChainItem::Caps(segment) => {
                caps.push(segment.label);
                caps_span = Some(match caps_span.take() {
                    Some(existing) => existing.merge(&segment.span),
                    None => segment.span,
                });
            }
            ChainItem::Node(node) => {
                let endpoint = Endpoint::Node(node);
                if let Some(edge) =
                    connect_endpoints(previous.take(), endpoint.clone(), &caps, caps_span.as_ref())
                {
                    pending_edges.push(edge);
                }
                previous = Some(endpoint);
                caps.clear();
                caps_span = None;
            }
            ChainItem::Reference(reference) => {
                let endpoint = Endpoint::Reference(reference);
                if let Some(edge) =
                    connect_endpoints(previous.take(), endpoint.clone(), &caps, caps_span.as_ref())
                {
                    pending_edges.push(edge);
                }
                previous = Some(endpoint);
                caps.clear();
                caps_span = None;
            }
        }
    }

    if previous.is_none() && !caps.is_empty() {
        diagnostics.push(ParseDiagnostic::warning(
            "dangling-caps",
            "Encountered caps syntax that was not connected to any pipeline segment.",
            caps_span,
        ));
    }

    pending_edges
}

fn connect_endpoints(
    previous: Option<Endpoint>,
    current: Endpoint,
    caps: &[String],
    caps_span: Option<&SourceSpan>,
) -> Option<PendingEdge> {
    let previous = previous?;
    let span = endpoint_span(&previous)
        .merge(endpoint_span(&current))
        .merge(caps_span.unwrap_or(endpoint_span(&current)));

    Some(PendingEdge {
        from: previous,
        to: current,
        caps_label: if caps.is_empty() {
            None
        } else {
            Some(caps.join("\n"))
        },
        span,
    })
}

fn endpoint_span(endpoint: &Endpoint) -> &SourceSpan {
    match endpoint {
        Endpoint::Node(node) => &node.span,
        Endpoint::Reference(reference) => &reference.span,
    }
}

fn resolve_pending_edges(
    graph: &mut PipelineGraph,
    instance_map: &HashMap<String, String>,
    pending_edges: Vec<PendingEdge>,
    diagnostics: &mut Vec<ParseDiagnostic>,
) {
    let mut unresolved_nodes = HashMap::new();

    for edge in pending_edges {
        let (source_node_id, source_port) = resolve_endpoint(
            graph,
            instance_map,
            &mut unresolved_nodes,
            diagnostics,
            &edge.from,
            true,
        );
        let (target_node_id, target_port) = resolve_endpoint(
            graph,
            instance_map,
            &mut unresolved_nodes,
            diagnostics,
            &edge.to,
            false,
        );

        graph.edges.push(PipelineEdge {
            id: format!("edge-{}", graph.edges.len()),
            source_node_id,
            source_port,
            target_node_id,
            target_port,
            caps_label: edge.caps_label,
            source_span: edge.span,
        });
    }
}

fn resolve_endpoint(
    graph: &mut PipelineGraph,
    instance_map: &HashMap<String, String>,
    unresolved_nodes: &mut HashMap<String, String>,
    diagnostics: &mut Vec<ParseDiagnostic>,
    endpoint: &Endpoint,
    is_source: bool,
) -> (String, Option<PipelinePort>) {
    match endpoint {
        Endpoint::Node(node) => (node.node_id.clone(), None),
        Endpoint::Reference(reference) => {
            let node_id = if let Some(node_id) = instance_map.get(&reference.name) {
                node_id.clone()
            } else if let Some(node_id) = unresolved_nodes.get(&reference.name) {
                node_id.clone()
            } else {
                let node_id = format!("node-{}", graph.nodes.len());
                graph.nodes.push(PipelineNode {
                    id: node_id.clone(),
                    factory_name: reference.name.clone(),
                    instance_name: Some(reference.name.clone()),
                    kind: PipelineNodeKind::Unknown,
                    properties: Vec::new(),
                    source_span: reference.span.clone(),
                });
                unresolved_nodes.insert(reference.name.clone(), node_id.clone());
                diagnostics.push(ParseDiagnostic::warning(
                    "unresolved-reference",
                    format!(
                        "Could not resolve named reference `{}` to a declared element.",
                        reference.name
                    ),
                    Some(reference.span.clone()),
                ));
                node_id
            };

            let port_name = reference.pad.clone().unwrap_or_else(|| {
                if is_source {
                    "src".into()
                } else {
                    "sink".into()
                }
            });
            let port_kind = if reference.pad.is_none() {
                if is_source {
                    PipelinePortKind::Src
                } else {
                    PipelinePortKind::Sink
                }
            } else if !is_source && port_name.starts_with("sink_") {
                PipelinePortKind::Request
            } else {
                PipelinePortKind::Named
            };

            let port = PipelinePort {
                id: format!(
                    "{node_id}:{port_name}:{}",
                    if is_source { "src" } else { "sink" }
                ),
                node_id: node_id.clone(),
                port_kind,
                port_name,
            };

            (node_id, Some(port))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use super::parse_graph;
    use crate::parser::normalize_text;

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("src-tauri should live under the repo root")
            .to_path_buf()
    }

    #[test]
    fn normalizes_rtf_without_rich_text_markup() {
        let sample_path = repo_root().join("26_release_record_smoothing.pld.rtf");
        let raw = fs::read_to_string(sample_path).expect("sample fixture should exist");

        let normalized = normalize_text(&raw);

        assert!(!normalized.normalized_text.contains("{\\rtf"));
        assert!(normalized
            .normalized_text
            .contains("qtiqmmfsrc camera=0 name=eocam0 !"));
        assert!(normalized
            .normalized_text
            .contains("eo_sr_in.src_1 ! queue"));
    }

    #[test]
    fn parses_sample_rtf_files_without_crashing() {
        for sample_name in ["26_release_record_smoothing.pld.rtf", "27_pipmux.pld.rtf"] {
            let sample_path = repo_root().join(sample_name);
            let raw = fs::read_to_string(sample_path).expect("sample fixture should exist");
            let normalized = normalize_text(&raw);
            let (graph, diagnostics) = parse_graph(&normalized.normalized_text);

            assert!(
                graph.nodes.len() > 10,
                "expected multiple nodes from sample {sample_name}"
            );
            assert!(
                graph.edges.len() > 10,
                "expected multiple edges from sample {sample_name}"
            );
            assert!(
                graph
                    .nodes
                    .iter()
                    .any(|node| node.instance_name.as_deref() == Some("eoraw")
                        || node.instance_name.as_deref() == Some("mixer")),
                "expected named elements from sample {sample_name}"
            );
            assert!(
                diagnostics
                    .iter()
                    .all(|diagnostic| diagnostic.code != "panic"),
                "parser should report warnings, not crash, for sample {sample_name}"
            );
        }
    }
}
