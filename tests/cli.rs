//! The real program, run the way a user runs it. Nothing here reaches the network.

use std::path::Path;
use std::process::{Command, Output};

use serde_json::{Value, json};

struct Sandbox(tempfile::TempDir);

impl Sandbox {
    fn new() -> Self {
        Self(tempfile::tempdir().unwrap())
    }

    fn command(&self, args: &[&str]) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_ormedia"));
        command.args(args).env("XDG_CONFIG_HOME", self.0.path());
        // No test may reach GitHub. The cache is in the sandbox too, in case one tries.
        command.env("XDG_CACHE_HOME", self.0.path().join("cache"));
        command.env("ORMEDIA_UPDATE_CHECK", "false");
        for variable in [
            "OPENROUTER_API_KEY",
            "ORMEDIA_IMAGE_MODEL",
            "ORMEDIA_VIDEO_MODEL",
        ] {
            command.env_remove(variable);
        }
        command
    }

    fn run(&self, args: &[&str]) -> Output {
        self.command(args).output().unwrap()
    }

    fn config_file(&self) -> std::path::PathBuf {
        self.0.path().join("openrouter-media").join("config.json")
    }
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).unwrap()
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).unwrap()
}

#[test]
fn help_is_one_page_and_no_arguments_shows_it() {
    let sandbox = Sandbox::new();
    let help = sandbox.run(&["--help"]);
    assert!(help.status.success());
    assert!(
        stdout(&help)
            .starts_with("ormedia - generate images and videos with OpenRouter\n\nUsage\n")
    );
    assert!(stdout(&help).contains("--open | --no-open"));
    assert_eq!(stdout(&sandbox.run(&[])), stdout(&help));
    assert_eq!(stdout(&sandbox.run(&["help"])), stdout(&help));
    assert!(stdout(&sandbox.run(&["image", "--help"])).contains("--aspect-ratio <r>"));
}

#[test]
fn the_version_is_the_same_everywhere() {
    let version = env!("CARGO_PKG_VERSION");
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let manifest =
        std::fs::read_to_string(root.join(".claude-plugin").join("plugin.json")).unwrap();
    let manifest: Value = serde_json::from_str(&manifest).unwrap();
    assert_eq!(
        manifest["version"],
        json!(version),
        "plugin.json and Cargo.toml must agree"
    );
    assert_eq!(
        stdout(&Sandbox::new().run(&["--version"])),
        format!("ormedia {version}\n")
    );
}

#[test]
fn config_saves_shows_and_removes_a_setting() {
    let sandbox = Sandbox::new();
    let file = sandbox.config_file().display().to_string();

    let saved = sandbox.run(&["config", "set", "image-model", "acme/image-1"]);
    assert_eq!(stdout(&saved), format!("Saved image-model in {file}\n"));
    assert_eq!(stderr(&saved), "");

    let shown = stdout(&sandbox.run(&["config"]));
    assert!(
        shown.starts_with("image-model  acme/image-1  (saved)\n"),
        "{shown}"
    );
    assert!(shown.contains("\nout          not set\n"), "{shown}");
    assert!(
        shown.ends_with(&format!("\n\nSaved settings are in {file}\n")),
        "{shown}"
    );

    let shown: Value = serde_json::from_str(&stdout(&sandbox.run(&["config", "--json"]))).unwrap();
    assert_eq!(shown["path"], json!(file));
    assert_eq!(
        shown["settings"]["image-model"],
        json!({ "value": "acme/image-1", "source": file })
    );
    assert_eq!(
        shown["settings"]["out"],
        json!({ "value": null, "source": "default" })
    );

    let removed = sandbox.run(&["config", "unset", "image-model"]);
    assert_eq!(
        stdout(&removed),
        format!("Removed image-model from {file}\n")
    );
    let shown: Value = serde_json::from_str(&stdout(&sandbox.run(&["config", "--json"]))).unwrap();
    assert_eq!(shown["settings"]["image-model"]["source"], json!("default"));
}

#[test]
fn config_set_says_when_an_environment_variable_still_wins() {
    let sandbox = Sandbox::new();
    let with_variable = |args: &[&str]| {
        sandbox
            .command(args)
            .env("ORMEDIA_VIDEO_MODEL", "env/video")
            .output()
            .unwrap()
    };
    let saved = with_variable(&["config", "set", "video-model", "acme/video-1"]);
    assert_eq!(
        stderr(&saved),
        "$ORMEDIA_VIDEO_MODEL is set, and it wins over the saved video-model.\n"
    );
    let shown = stdout(&with_variable(&["config"]));
    assert!(
        shown.contains("video-model  env/video  ($ORMEDIA_VIDEO_MODEL)\n"),
        "{shown}"
    );
}

#[test]
fn a_call_that_cannot_work_fails_with_a_reason() {
    let sandbox = Sandbox::new();
    let failure = |args: &[&str]| {
        let output = sandbox.run(args);
        assert!(!output.status.success(), "{args:?} must fail");
        assert_eq!(stdout(&output), "", "{args:?} must print nothing to stdout");
        stderr(&output)
    };
    assert!(
        failure(&["config", "set", "colour", "red"])
            .starts_with("ormedia: Unknown setting \"colour\"")
    );
    assert_eq!(
        failure(&["config", "set", "open", "yes"]),
        "ormedia: open must be true or false, got \"yes\"\n"
    );
    assert!(!sandbox.config_file().exists());
    for usage in [
        &["config", "set", "open"][..],
        &["config", "get", "open"],
        &["paint"],
        &["models"],
    ] {
        assert!(failure(usage).contains("Usage:"), "{usage:?}");
    }
    assert_eq!(failure(&["image"]), "ormedia: An image needs a --prompt\n");
    assert_eq!(
        failure(&["video"]),
        "ormedia: A video needs a --prompt or a frame image\n"
    );
    assert_eq!(
        failure(&["image", "--prompt", "a boat"]),
        "ormedia: No API key. Run `ormedia login`, or set OPENROUTER_API_KEY.\n"
    );
}

#[test]
fn a_broken_config_stops_a_command_before_it_can_cost_money() {
    let sandbox = Sandbox::new();
    std::fs::create_dir_all(sandbox.config_file().parent().unwrap()).unwrap();
    std::fs::write(sandbox.config_file(), "{bad").unwrap();
    for args in [
        &["image", "--prompt", "a boat"][..],
        &["video", "--prompt", "a boat"],
        &["job", "abc"],
    ] {
        let output = sandbox
            .command(args)
            .env("OPENROUTER_API_KEY", "sk-test")
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1));
        assert!(
            stderr(&output).contains("config.json is not valid JSON"),
            "{args:?}"
        );
    }
}

#[test]
fn status_without_a_key_says_so_and_fails() {
    let sandbox = Sandbox::new();
    let plain = sandbox.run(&["status"]);
    assert_eq!(stdout(&plain), "Not signed in. Run `ormedia login`.\n");
    assert_eq!(plain.status.code(), Some(1));
    assert_eq!(
        stdout(&sandbox.run(&["status", "--json"])),
        "{\"signedIn\":false}\n"
    );
    assert_eq!(stdout(&sandbox.run(&["logout"])), "Stored key deleted.\n");
}

#[test]
fn the_update_check_can_be_turned_off_and_runs_at_most_once_a_day() {
    let sandbox = Sandbox::new();
    let state = sandbox
        .0
        .path()
        .join("cache")
        .join("openrouter-media")
        .join("update-check.json");

    // Off, from the variable that every test in this file sets.
    sandbox.run(&["config"]);
    assert!(!state.exists());

    // On, but a check already ran today, so GitHub is not asked and nothing is said.
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap();
    let checked = json!({ "checked_at": now.as_secs() }).to_string();
    std::fs::create_dir_all(state.parent().unwrap()).unwrap();
    std::fs::write(&state, &checked).unwrap();
    let output = sandbox
        .command(&["config"])
        .env_remove("ORMEDIA_UPDATE_CHECK")
        .output()
        .unwrap();
    assert!(
        stdout(&output).contains("update-check true  (default)\n"),
        "{}",
        stdout(&output)
    );
    assert_eq!(stderr(&output), "");
    assert_eq!(std::fs::read_to_string(&state).unwrap(), checked);

    // Off, from the saved setting.
    std::fs::remove_file(&state).unwrap();
    sandbox.run(&["config", "set", "update-check", "false"]);
    sandbox
        .command(&["config"])
        .env_remove("ORMEDIA_UPDATE_CHECK")
        .output()
        .unwrap();
    assert!(!state.exists());
}
