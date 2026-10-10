use serde_json::Value;

/// A JSON value as a person writes it: text without the quotes.
pub fn plain(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}

fn join(values: &[Value]) -> String {
    values.iter().map(plain).collect::<Vec<_>>().join(", ")
}

/// Set, and not to "no": the API marks a missing feature as `null` or `false`.
fn supported(value: &Value) -> bool {
    !matches!(value, Value::Null | Value::Bool(false))
}

/// One image option as text: `resolution: 1K, 2K, 4K`, `n: 1-4`, or just `seed`.
fn describe_parameter(name: &str, spec: &Value) -> String {
    match spec["type"].as_str() {
        Some("enum") => format!(
            "{name}: {}",
            join(spec["values"].as_array().map_or(&[], Vec::as_slice))
        ),
        Some("range") => format!("{name}: {}-{}", plain(&spec["min"]), plain(&spec["max"])),
        _ => name.to_string(),
    }
}

fn describe_image_model(model: &Value) -> Vec<String> {
    let parameters = model["supported_parameters"].as_object();
    parameters
        .into_iter()
        .flatten()
        .map(|(name, spec)| describe_parameter(name, spec))
        .collect()
}

fn describe_video_model(model: &Value) -> Vec<String> {
    let mut lines = Vec::new();
    let mut list = |label: &str, field: &str, unit: &str| {
        if let Some(values) = model[field].as_array().filter(|values| !values.is_empty()) {
            lines.push(format!("{label}: {}{unit}", join(values)));
        }
    };
    list("duration", "supported_durations", " s");
    list("resolution", "supported_resolutions", "");
    list("aspect_ratio", "supported_aspect_ratios", "");
    list("frames", "supported_frame_images", "");
    for feature in ["audio", "seed"] {
        let field = if feature == "audio" {
            "generate_audio"
        } else {
            feature
        };
        if supported(&model[field]) {
            lines.push(feature.to_string());
        }
    }
    // Shown raw: the price keys differ per provider and say their own unit.
    for (sku, price) in model["pricing_skus"].as_object().into_iter().flatten() {
        lines.push(format!("price {sku}: {}", plain(price)));
    }
    lines
}

/// A model as a few lines of text: its id and name, then one option per line.
pub fn describe_model(video: bool, model: &Value) -> String {
    let lines = if video {
        describe_video_model(model)
    } else {
        describe_image_model(model)
    };
    let mut text = format!("{} ({})", plain(&model["id"]), plain(&model["name"]));
    for line in lines {
        text.push_str("\n  ");
        text.push_str(&line);
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn an_image_model_lists_each_option_in_its_own_shape() {
        let model = json!({
            "id": "a/b",
            "name": "B",
            "supported_parameters": {
                "resolution": { "type": "enum", "values": ["1K", "2K"] },
                "n": { "type": "range", "min": 1, "max": 4 },
                "seed": { "type": "boolean" }
            }
        });
        assert_eq!(
            describe_model(false, &model),
            "a/b (B)\n  resolution: 1K, 2K\n  n: 1-4\n  seed"
        );
    }

    #[test]
    fn a_video_model_skips_what_it_does_not_support() {
        let model = json!({
            "id": "v/x",
            "name": "X",
            "supported_durations": [4, 8],
            "supported_resolutions": null,
            "generate_audio": true,
            "seed": null,
            "pricing_skus": { "duration_seconds": "0.1" }
        });
        assert_eq!(
            describe_model(true, &model),
            "v/x (X)\n  duration: 4, 8 s\n  audio\n  price duration_seconds: 0.1"
        );
    }
}
