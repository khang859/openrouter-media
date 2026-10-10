use std::fs;
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;

use crate::Result;
use crate::env::Env;

/// One generated file, before it has a name.
#[derive(Debug, PartialEq)]
pub struct Output {
    pub data: Vec<u8>,
    pub mime_type: String,
}

fn image_mime(path: &Path) -> Option<&'static str> {
    match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
        "png" => Some("image/png"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "webp" => Some("image/webp"),
        "gif" => Some("image/gif"),
        _ => None,
    }
}

/// A reference image as the API wants it: a URL as is, a local file as a data URL.
pub fn to_image_url(reference: &str) -> Result<String> {
    let lower = reference.to_ascii_lowercase();
    if ["http:", "https:", "data:"]
        .iter()
        .any(|scheme| lower.starts_with(scheme))
    {
        return Ok(reference.to_string());
    }
    let mime = image_mime(Path::new(reference)).ok_or_else(|| {
        format!("Unsupported image type: {reference} (use png, jpg, webp, or gif)")
    })?;
    let data = fs::read(reference).map_err(|error| format!("Cannot read {reference}: {error}"))?;
    Ok(format!("data:{mime};base64,{}", STANDARD.encode(data)))
}

pub fn extension_for(mime_type: &str) -> &'static str {
    let media_type = mime_type
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    match media_type.as_str() {
        "image/png" => ".png",
        "image/jpeg" => ".jpg",
        "image/webp" => ".webp",
        "image/gif" => ".gif",
        "image/svg+xml" => ".svg",
        "video/mp4" => ".mp4",
        "video/webm" => ".webm",
        "video/quicktime" => ".mov",
        _ => ".bin",
    }
}

/// A short, safe file name from a prompt.
pub fn slug(text: &str) -> String {
    let mut words = String::new();
    for character in text.chars() {
        if character.is_ascii_alphanumeric() {
            words.push(character.to_ascii_lowercase());
        } else if !words.is_empty() && !words.ends_with('-') {
            words.push('-');
        }
    }
    words.truncate(40);
    let words = words.trim_end_matches('-');
    if words.is_empty() {
        "media".to_string()
    } else {
        words.to_string()
    }
}

/// `path` without `.` and `..` parts, so a saved file prints as a clean path.
fn tidy(path: &Path) -> PathBuf {
    let mut tidy = PathBuf::new();
    for part in path.components() {
        match part {
            Component::CurDir => {}
            Component::ParentDir => {
                tidy.pop();
            }
            other => tidy.push(other),
        }
    }
    tidy
}

/// Where output number `index` goes. `out` with an extension names the file,
/// anything else names a folder. The extension always comes from what was
/// actually returned, so a `.png` name never hides a JPEG.
pub fn output_path(out: Option<&str>, stem: &str, index: usize, ext: &str, cwd: &Path) -> PathBuf {
    let suffix = if index == 0 {
        String::new()
    } else {
        format!("-{}", index + 1)
    };
    let out = Path::new(out.unwrap_or("."));
    match (out.extension(), out.file_stem()) {
        (Some(_), Some(base)) => {
            let name = format!("{}{suffix}{ext}", base.to_string_lossy());
            tidy(&cwd.join(out.with_file_name(name)))
        }
        _ => tidy(&cwd.join(out).join(format!("{stem}{suffix}{ext}"))),
    }
}

/// The path itself, or the first `name-2`, `name-3`... that is not taken.
pub fn free_path(path: PathBuf, exists: impl Fn(&Path) -> bool) -> PathBuf {
    if !exists(&path) {
        return path;
    }
    let base = path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    let ext = path
        .extension()
        .map(|ext| format!(".{}", ext.to_string_lossy()))
        .unwrap_or_default();
    (2..)
        .map(|n| path.with_file_name(format!("{base}-{n}{ext}")))
        .find(|candidate| !exists(candidate))
        .expect("an unused number always exists")
}

/// Writes each output and returns where it went. Never overwrites a file.
pub fn save_outputs(
    outputs: &[Output],
    out: Option<&str>,
    stem: &str,
    cwd: &Path,
) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    for (index, output) in outputs.iter().enumerate() {
        let wanted = output_path(out, stem, index, extension_for(&output.mime_type), cwd);
        let path = free_path(wanted, Path::exists);
        let failed = |error: std::io::Error| format!("Cannot save {}: {error}", path.display());
        if let Some(folder) = path.parent() {
            fs::create_dir_all(folder).map_err(failed)?;
        }
        // create_new refuses a file that appeared since the check above.
        let mut file = fs::File::create_new(&path).map_err(failed)?;
        file.write_all(&output.data).map_err(failed)?;
        paths.push(path);
    }
    Ok(paths)
}

/// A folder only the user can open. It holds the API key.
pub fn create_private_dir(dir: &Path) -> std::io::Result<()> {
    let mut builder = fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    std::os::unix::fs::DirBuilderExt::mode(&mut builder, 0o700);
    builder.create(dir)
}

/// Fire and forget: a viewer that fails to start must not fail the command.
fn launch(command: &str, args: &[&str]) {
    let _ = Command::new(command)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
}

pub fn open_url(url: &str) {
    if cfg!(target_os = "macos") {
        launch("open", &[url]);
    } else if cfg!(windows) {
        launch("cmd", &["/c", "start", "", url]);
    } else {
        launch("xdg-open", &[url]);
    }
}

/// Inside a Fleet pane, show images in a Fleet tab. Fleet has no video viewer.
pub fn open_in_fleet(paths: &[PathBuf], env: &Env) {
    let images: Vec<&str> = paths
        .iter()
        .filter(|path| image_mime(path).is_some())
        .filter_map(|path| path.to_str())
        .collect();
    if env.get("FLEET_SESSION") != Some("1") || images.is_empty() {
        return;
    }
    launch("fleet", &[&["open"], images.as_slice()].concat());
}

/// `20261010-130509`, in UTC: sortable and safe in a file name.
pub fn timestamp(now: SystemTime) -> String {
    let seconds = now
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_secs());
    let (days, rest) = (seconds / 86_400, seconds % 86_400);
    // Days since 1970 to a calendar date, after Howard Hinnant's civil_from_days.
    let shifted = days + 719_468;
    let (era, day_of_era) = (shifted / 146_097, shifted % 146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    };
    let year = year_of_era + era * 400 + u64::from(month <= 2);
    let (hour, minute, second) = (rest / 3_600, rest % 3_600 / 60, rest % 60);
    format!("{year:04}{month:02}{day:02}-{hour:02}{minute:02}{second:02}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::time::Duration;

    fn png(text: &str) -> Vec<Output> {
        vec![Output {
            data: text.into(),
            mime_type: "image/png".into(),
        }]
    }

    #[test]
    fn to_image_url_passes_urls_through_and_inlines_local_files() {
        assert_eq!(to_image_url("https://x/a.png").unwrap(), "https://x/a.png");
        assert_eq!(
            to_image_url("DATA:image/png;base64,AA").unwrap(),
            "DATA:image/png;base64,AA"
        );
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("ref.JPG");
        fs::write(&file, "pixels").unwrap();
        assert_eq!(
            to_image_url(file.to_str().unwrap()).unwrap(),
            "data:image/jpeg;base64,cGl4ZWxz"
        );
    }

    #[test]
    fn to_image_url_rejects_a_file_that_is_not_an_image() {
        assert!(
            to_image_url("notes.txt")
                .unwrap_err()
                .starts_with("Unsupported image type")
        );
        assert!(
            to_image_url("missing.png")
                .unwrap_err()
                .starts_with("Cannot read missing.png")
        );
    }

    #[test]
    fn extension_for_reads_the_media_type_with_or_without_parameters() {
        assert_eq!(extension_for("image/jpeg"), ".jpg");
        assert_eq!(extension_for("video/mp4; codecs=avc1"), ".mp4");
        assert_eq!(extension_for("application/x-unknown"), ".bin");
    }

    #[test]
    fn slug_makes_a_short_safe_file_name() {
        assert_eq!(slug("A Cat, in SPACE!"), "a-cat-in-space");
        assert_eq!(slug("  --Été 2026"), "t-2026");
        assert_eq!(slug("!!!"), "media");
        let long = slug(&"word ".repeat(40));
        assert!(long.len() <= 40 && !long.ends_with('-'));
    }

    #[test]
    fn output_path_treats_a_name_with_an_extension_as_a_file_anything_else_as_a_folder() {
        let cwd = Path::new("/w");
        let at = |out, index| output_path(out, "s", index, ".jpg", cwd);
        assert_eq!(at(None, 0), cwd.join("s.jpg"));
        assert_eq!(at(Some("art"), 0), cwd.join("art").join("s.jpg"));
        assert_eq!(at(Some("art"), 1), cwd.join("art").join("s-2.jpg"));
        assert_eq!(
            at(Some("./art/../media"), 0),
            cwd.join("media").join("s.jpg")
        );
        // The real extension wins over the one that was asked for.
        assert_eq!(
            at(Some("art/hero.png"), 0),
            cwd.join("art").join("hero.jpg")
        );
        assert_eq!(
            at(Some("art/hero.png"), 1),
            cwd.join("art").join("hero-2.jpg")
        );
    }

    #[cfg(unix)]
    #[test]
    fn output_path_keeps_an_absolute_path() {
        let path = output_path(Some("/abs/hero.png"), "s", 1, ".jpg", Path::new("/w"));
        assert_eq!(path, Path::new("/abs/hero-2.jpg"));
    }

    #[test]
    fn free_path_steps_past_names_that_are_taken() {
        let dir = Path::new("w");
        let taken: HashSet<_> = [dir.join("a.png"), dir.join("a-2.png")].into();
        let exists = |path: &Path| taken.contains(path);
        assert_eq!(free_path(dir.join("b.png"), exists), dir.join("b.png"));
        assert_eq!(free_path(dir.join("a.png"), exists), dir.join("a-3.png"));
    }

    #[test]
    fn save_outputs_writes_into_a_new_folder_and_never_overwrites() {
        let dir = tempfile::tempdir().unwrap();
        let first = save_outputs(&png("first"), Some("nested"), "pic", dir.path()).unwrap();
        let second = save_outputs(&png("second"), Some("nested"), "pic", dir.path()).unwrap();
        assert_eq!(first, [dir.path().join("nested").join("pic.png")]);
        assert_eq!(second, [dir.path().join("nested").join("pic-2.png")]);
        assert_eq!(fs::read_to_string(&first[0]).unwrap(), "first");
        assert_eq!(fs::read_to_string(&second[0]).unwrap(), "second");
    }

    #[test]
    fn timestamp_is_sortable_and_file_name_safe() {
        let at = |seconds| timestamp(UNIX_EPOCH + Duration::from_secs(seconds));
        assert_eq!(at(1_791_637_509), "20261010-130509");
        assert_eq!(at(0), "19700101-000000");
        assert_eq!(at(1_709_251_199), "20240229-235959");
    }
}
