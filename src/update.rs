use std::fs;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};

use crate::env::Env;

const LATEST_RELEASE_URL: &str =
    "https://api.github.com/repos/khang859/openrouter-media/releases/latest";
const CHECK_EVERY: Duration = Duration::from_secs(24 * 60 * 60);

/// `1.2.3` or `v1.2.3` as numbers. Anything else, such as a pre-release, is not a version to offer.
fn parse(version: &str) -> Option<[u64; 3]> {
    let mut parts = version
        .strip_prefix('v')
        .unwrap_or(version)
        .split('.')
        .map(str::parse);
    let numbers = [
        parts.next()?.ok()?,
        parts.next()?.ok()?,
        parts.next()?.ok()?,
    ];
    parts.next().is_none().then_some(numbers)
}

fn is_newer(latest: &str, current: &str) -> bool {
    matches!((parse(latest), parse(current)), (Some(latest), Some(current)) if latest > current)
}

/// The tag of the newest release on GitHub. Quiet on any failure: an update
/// check must never break, or slow down for long, the command the user ran.
pub fn latest_release() -> Option<String> {
    let config = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(3)))
        .user_agent(concat!("ormedia/", env!("CARGO_PKG_VERSION")))
        .build();
    let agent: ureq::Agent = config.into();
    let mut response = agent
        .get(LATEST_RELEASE_URL)
        .header("Accept", "application/vnd.github+json")
        .call()
        .ok()?;
    let release: Value = response.body_mut().read_json().ok()?;
    release["tag_name"].as_str().map(String::from)
}

/// True when no check ran in the last day. Records this one first, so a
/// computer with no network pays for one slow check a day, not one per command.
fn claim_check(env: &Env, now: SystemTime) -> bool {
    let Some(dir) = env.cache_dir() else {
        return false;
    };
    let file = dir.join("update-check.json");
    let now = now
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_secs());
    let checked_at = fs::read_to_string(&file)
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
        .and_then(|state| state["checked_at"].as_u64());
    let due = checked_at.is_none_or(|at| at > now || now - at >= CHECK_EVERY.as_secs());
    // A cache that cannot be written means no check: there is no way to remember it ran.
    due && fs::create_dir_all(&dir)
        .and_then(|()| fs::write(&file, json!({ "checked_at": now }).to_string()))
        .is_ok()
}

/// The line to show the user, when a check is due and a newer release exists.
pub fn notice(
    env: &Env,
    now: SystemTime,
    latest: impl FnOnce() -> Option<String>,
) -> Option<String> {
    let current = env!("CARGO_PKG_VERSION");
    let latest = claim_check(env, now).then(latest)??;
    is_newer(&latest, current).then(|| {
        let latest = latest.trim_start_matches('v');
        format!(
            "ormedia {latest} is out, and this is {current}. \
             Update with: claude plugin update openrouter-media@openrouter-media"
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    const NEWER: &str = "v999.0.0";

    fn at(seconds: u64) -> SystemTime {
        UNIX_EPOCH + Duration::from_secs(seconds)
    }

    fn release(tag: &str) -> impl FnOnce() -> Option<String> {
        move || Some(tag.to_string())
    }

    #[test]
    fn versions_compare_as_numbers() {
        assert!(is_newer("v0.10.0", "0.9.9"));
        assert!(is_newer("1.0.0", "0.99.99"));
        assert!(!is_newer("v0.2.0", "0.2.0"));
        assert!(!is_newer("v0.1.9", "0.2.0"));
        for odd in ["", "v1", "1.2", "1.2.3.4", "v1.2.3-beta.1", "latest"] {
            assert!(!is_newer(odd, "0.0.1"), "{odd}");
        }
    }

    #[test]
    fn a_newer_release_is_announced_with_how_to_update() {
        let (env, _dir) = Env::temp();
        let notice = notice(&env, at(1_000_000), release(NEWER)).unwrap();
        assert!(
            notice.starts_with("ormedia 999.0.0 is out, and this is "),
            "{notice}"
        );
        assert!(
            notice.ends_with("claude plugin update openrouter-media@openrouter-media"),
            "{notice}"
        );
    }

    #[test]
    fn the_same_or_an_older_release_is_not_announced() {
        for tag in [concat!("v", env!("CARGO_PKG_VERSION")), "v0.0.1", "nightly"] {
            let (env, _dir) = Env::temp();
            assert_eq!(notice(&env, at(1_000_000), release(tag)), None, "{tag}");
        }
    }

    #[test]
    fn github_is_asked_at_most_once_a_day() {
        let (env, _dir) = Env::temp();
        let asked = Cell::new(0);
        let ask = |now| {
            notice(&env, at(now), || {
                asked.set(asked.get() + 1);
                Some(NEWER.to_string())
            })
        };
        let day = CHECK_EVERY.as_secs();
        assert!(ask(1_000_000).is_some());
        assert_eq!(ask(1_000_000 + day - 1), None);
        assert_eq!(asked.get(), 1);
        assert!(ask(1_000_000 + day).is_some());
        // A clock that went back must not silence the check for good.
        assert!(ask(500).is_some());
        assert_eq!(asked.get(), 3);
    }

    #[test]
    fn a_check_that_fails_is_quiet_and_still_counts_for_the_day() {
        let (env, _dir) = Env::temp();
        assert_eq!(notice(&env, at(1_000_000), || None), None);
        assert_eq!(
            notice(&env, at(1_000_001), || panic!("asked twice in one day")),
            None
        );
    }

    #[test]
    fn a_cache_that_cannot_be_used_means_no_check() {
        let (env, dir) = Env::temp();
        fs::write(
            dir.path().join("cache"),
            "a file where the folder should be",
        )
        .unwrap();
        assert_eq!(
            notice(&env, at(1_000_000), || panic!("asked with no cache")),
            None
        );

        let (env, dir) = Env::temp();
        let state = dir
            .path()
            .join("cache")
            .join("openrouter-media")
            .join("update-check.json");
        fs::create_dir_all(state.parent().unwrap()).unwrap();
        fs::write(&state, "not json").unwrap();
        assert!(
            notice(&env, at(1_000_000), release(NEWER)).is_some(),
            "a damaged state file is replaced"
        );
        assert_eq!(
            fs::read_to_string(&state).unwrap(),
            "{\"checked_at\":1000000}"
        );
    }
}
