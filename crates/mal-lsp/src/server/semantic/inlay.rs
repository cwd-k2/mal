use serde::Deserialize;
use serde_json::{Value, json};

use super::{Position, SemanticRequest, Server, error, success};

#[derive(Deserialize)]
struct RangeParams {
    range: Range,
}

#[derive(Deserialize)]
struct Range {
    start: Position,
    end: Position,
}

/// Marks where control leaves its result block, so the branch that ends a path can be told from the ones that
/// continue without reading each one. The label names the result binders the path transfers to.
pub(super) fn hints(server: &mut Server, id: Value, params: Value) -> Value {
    let Ok(RangeParams { range }) = serde_json::from_value::<RangeParams>(params.clone()) else {
        return error(id, -32602, "invalid inlay hint parameters");
    };
    let (source, semantic) = match server.document_request(&params) {
        SemanticRequest::Ready(value) => value,
        SemanticRequest::Unavailable => return success(id, json!([])),
        SemanticRequest::Invalid => {
            return error(id, -32602, "invalid parameters or document is not open");
        }
    };
    let within = |line: usize, character: usize| {
        (line, character) >= (range.start.line, range.start.character)
            && (line, character) <= (range.end.line, range.end.character)
    };
    let hints = semantic
        .exits()
        .filter_map(|exit| {
            Some((
                source.utf16_position(exit.span.end())?,
                label(&exit.targets),
            ))
        })
        .filter(|(position, _)| within(position.line, position.character))
        .map(|(position, label)| {
            json!({
                "position": {"line": position.line, "character": position.character},
                "label": label,
                "paddingLeft": true
            })
        })
        .collect::<Vec<_>>();
    success(id, json!(hints))
}

/// `→ return`, or `→ ok, fail` when the units transfer to several binders. A unit that names none never returns.
fn label(targets: &[String]) -> String {
    if targets.is_empty() {
        "never returns".to_owned()
    } else {
        format!("→ {}", targets.join(", "))
    }
}
