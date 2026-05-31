//! Application-level HTTP proxy configuration.
//!
//! On startup we snapshot `HTTP_PROXY`/`HTTPS_PROXY` (and lowercase variants)
//! into a process-global cache and **remove them from the environment** so
//! that the Tauri WebView (webkit2gtk on Linux) does not try to route its
//! own traffic — including the Vite dev server at `http://localhost:1420` —
//! through the proxy.
//!
//! Internal Rust HTTP requests still honour the proxy via [`apply_proxy`],
//! which reads the cached values and configures `reqwest` explicitly.

use std::sync::OnceLock;
use std::sync::RwLock;

use reqwest::{ClientBuilder, Proxy};
use tracing::{debug, warn};

#[derive(Default, Clone, Debug)]
struct ProxyConfig {
    http: Option<String>,
    https: Option<String>,
}

static PROXY_CONFIG: OnceLock<ProxyConfig> = OnceLock::new();

/// User-selected proxy from application settings. Takes precedence over the
/// env-captured proxy when set. Updated at startup and whenever the active
/// proxy setting changes. An empty/`None` value means "direct connection".
static RUNTIME_PROXY: RwLock<Option<String>> = RwLock::new(None);

/// Set (or clear) the active proxy chosen by the user in Settings.
///
/// Pass `None` or an empty string to disable the runtime proxy and fall back
/// to env-captured proxies (if any). New HTTP/LLM/search clients built after
/// this call will use the updated value.
pub fn set_runtime_proxy(url: Option<String>) {
    let normalized = url
        .map(|u| u.trim().to_string())
        .filter(|u| !u.is_empty());
    if let Ok(mut guard) = RUNTIME_PROXY.write() {
        *guard = normalized;
    }
}

fn runtime_proxy() -> Option<String> {
    RUNTIME_PROXY.read().ok().and_then(|g| g.clone())
}


fn read_env(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

/// Capture proxy env vars and unset them so the WebView doesn't inherit them.
///
/// Must be called once, very early in `main`, before Tauri spawns the WebView
/// and before any `reqwest::Client` is constructed.
pub fn init_and_strip_env() {
    let https = read_env("HTTPS_PROXY").or_else(|| read_env("https_proxy"));
    let http = read_env("HTTP_PROXY").or_else(|| read_env("http_proxy"));

    let config = ProxyConfig { http, https };

    if let Some(ref url) = config.https {
        debug!(target: "http.proxy", proxy = %url, "Captured HTTPS proxy from env");
    }
    if let Some(ref url) = config.http {
        debug!(target: "http.proxy", proxy = %url, "Captured HTTP proxy from env");
    }

    // Strip every proxy-related env var so the WebView and any child processes
    // do not pick them up. We re-apply them per-client via `apply_proxy`.
    for var in [
        "HTTP_PROXY",
        "HTTPS_PROXY",
        "ALL_PROXY",
        "http_proxy",
        "https_proxy",
        "all_proxy",
    ] {
        std::env::remove_var(var);
    }

    let _ = PROXY_CONFIG.set(config);
}

fn config() -> ProxyConfig {
    PROXY_CONFIG.get().cloned().unwrap_or_default()
}

/// Apply the captured HTTP/HTTPS proxy settings to a `reqwest::ClientBuilder`.
///
/// Always disables reqwest's automatic env-based proxy detection (`no_proxy`)
/// and then re-adds proxies explicitly from the cached configuration. This is
/// deterministic regardless of the current process environment.
pub fn apply_proxy(mut builder: ClientBuilder) -> ClientBuilder {
    builder = builder.no_proxy();

    // A proxy explicitly chosen in Settings takes precedence over env vars and
    // applies to every scheme (http/https/socks).
    if let Some(url) = runtime_proxy() {
        match Proxy::all(&url) {
            Ok(p) => {
                debug!(target: "http.proxy", proxy = %url, "Using user-selected proxy");
                return builder.proxy(p);
            }
            Err(e) => warn!(target: "http.proxy", error = %e, url = %url, "Invalid proxy URL from settings"),
        }
    }

    let cfg = config();

    if let Some(url) = cfg.https {
        match Proxy::https(&url) {
            Ok(p) => {
                debug!(target: "http.proxy", proxy = %url, "Using HTTPS proxy");
                builder = builder.proxy(p);
            }
            Err(e) => warn!(target: "http.proxy", error = %e, url = %url, "Invalid HTTPS proxy URL"),
        }
    }

    if let Some(url) = cfg.http {
        match Proxy::http(&url) {
            Ok(p) => {
                debug!(target: "http.proxy", proxy = %url, "Using HTTP proxy");
                builder = builder.proxy(p);
            }
            Err(e) => warn!(target: "http.proxy", error = %e, url = %url, "Invalid HTTP proxy URL"),
        }
    }

    builder
}
