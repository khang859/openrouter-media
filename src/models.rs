use serde_json::Value;

use crate::cli::Kind;

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

/// A speech model, or a transcribe model, which has no voices.
fn describe_audio_model(model: &Value) -> Vec<String> {
    let mut lines = Vec::new();
    let voices = model["supported_voices"].as_array();
    if let Some(voices) = voices.filter(|voices| !voices.is_empty()) {
        lines.push(format!("voice: {}", join(voices)));
    }
    // Shown raw: one model prices a character, another a token.
    for (sku, price) in model["pricing"].as_object().into_iter().flatten() {
        if plain(price) != "0" {
            lines.push(format!("price {sku}: {}", plain(price)));
        }
    }
    lines
}

/// A model as a few lines of text: its id and name, then one option per line.
pub fn describe_model(kind: Kind, model: &Value) -> String {
    let lines = match kind {
        Kind::Image => describe_image_model(model),
        Kind::Video => describe_video_model(model),
        Kind::Speech | Kind::Transcribe => describe_audio_model(model),
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
            describe_model(Kind::Image, &model),
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
            describe_model(Kind::Video, &model),
            "v/x (X)\n  duration: 4, 8 s\n  audio\n  price duration_seconds: 0.1"
        );
    }

    #[test]
    fn a_speech_model_lists_its_voices_and_the_prices_that_are_not_zero() {
        let model = json!({
            "id": "s/y",
            "name": "Y",
            "supported_voices": ["ada", "ben"],
            "pricing": { "prompt": "0.000015", "completion": "0" }
        });
        assert_eq!(
            describe_model(Kind::Speech, &model),
            "s/y (Y)\n  voice: ada, ben\n  price prompt: 0.000015"
        );
        let bare = json!({ "id": "s/z", "name": "Z", "supported_voices": null });
        assert_eq!(describe_model(Kind::Speech, &bare), "s/z (Z)");
        assert_eq!(
            describe_model(Kind::Transcribe, &model),
            "s/y (Y)\n  voice: ada, ben\n  price prompt: 0.000015"
        );
    }
}
