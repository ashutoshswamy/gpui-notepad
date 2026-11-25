//! Update check: asks GitHub for the latest release and compares versions.
//! Uses the system `curl` (present on macOS, Windows 10+, most Linux) to avoid an HTTP dependency.
use std::process::Command;

const REPO: &str = "ashutoshswamy/gpui-notepad";
pub const CURRENT: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone)]
pub struct Update {
    pub version: String,
    pub url: String,
}

/// Blocking; call off the UI thread. `Ok(None)` means up to date.
pub fn fetch_latest() -> Result<Option<Update>, String> {
    let out = Command::new("curl")
        .args(["-sfL", "-m", "10", "-H", "Accept: application/vnd.github+json", "-H", "User-Agent: gpui-notepad"])
        .arg(format!("https://api.github.com/repos/{REPO}/releases/latest"))
        .output()
        .map_err(|e| format!("curl: {e}"))?;
    if !out.status.success() {
        return Err("could not reach GitHub".into());
    }
    let body = String::from_utf8_lossy(&out.stdout);
    let tag = json_str(&body, "tag_name").ok_or("unexpected response")?;
    let version = tag.trim_start_matches('v').to_string();
    let url = json_str(&body, "html_url").unwrap_or_else(|| format!("https://github.com/{REPO}/releases/latest"));
    Ok(newer(&version, CURRENT).then_some(Update { version, url }))
}

// ponytail: string scan instead of a JSON parser; the two fields we read are top-level strings
fn json_str(body: &str, key: &str) -> Option<String> {
    let pat = format!("\"{key}\"");
    let rest = body[body.find(&pat)? + pat.len()..].trim_start().strip_prefix(':')?.trim_start().strip_prefix('"')?;
    Some(rest[..rest.find('"')?].to_string())
}

fn parse(v: &str) -> Vec<u32> {
    v.split('.').map(|p| p.chars().take_while(char::is_ascii_digit).collect::<String>().parse().unwrap_or(0)).collect()
}

/// True when `a` is a higher version than `b`.
pub fn newer(a: &str, b: &str) -> bool {
    let (mut a, mut b) = (parse(a), parse(b));
    let n = a.len().max(b.len());
    a.resize(n, 0);
    b.resize(n, 0);
    a > b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_versions() {
        assert!(newer("0.1.2", "0.1.1"));
        assert!(newer("0.2", "0.1.9"));
        assert!(newer("1.0.0", "0.9.9"));
        assert!(!newer("0.1.1", "0.1.1"));
        assert!(!newer("0.1.0", "0.1.1"));
        assert!(newer("0.10.0", "0.9.0"));
    }

    #[test]
    fn extracts_json_strings() {
        let b = "{\n  \"url\": \"x\",\n  \"tag_name\": \"v0.1.2\",\n  \"html_url\" : \"https://e.com/r\"\n}";
        assert_eq!(json_str(b, "tag_name").as_deref(), Some("v0.1.2"));
        assert_eq!(json_str(b, "html_url").as_deref(), Some("https://e.com/r"));
        assert_eq!(json_str(b, "nope"), None);
    }
}
