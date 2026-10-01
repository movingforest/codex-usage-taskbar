use windows::core::{PCWSTR, PWSTR};
use windows::Win32::Foundation::{GlobalFree, HGLOBAL};
use windows::Win32::Networking::WinHttp::*;

use crate::native_interop::wide_str;

/// Resolve the current user's Windows proxy/PAC rules for the target URL.
/// ureq's environment-variable proxy support does not read these settings.
pub fn for_url(url: &str) -> Option<String> {
    unsafe {
        let mut config = WINHTTP_CURRENT_USER_IE_PROXY_CONFIG::default();
        WinHttpGetIEProxyConfigForCurrentUser(&mut config).ok()?;
        let static_proxy = read_wide(config.lpszProxy);
        let mut resolved = None;
        if !config.lpszAutoConfigUrl.is_null() || config.fAutoDetect.as_bool() {
            let agent = wide_str("CodexUsageTaskbar");
            let session = WinHttpOpen(
                PCWSTR(agent.as_ptr()),
                WINHTTP_ACCESS_TYPE_NO_PROXY,
                PCWSTR::null(),
                PCWSTR::null(),
                0,
            );
            if !session.is_null() {
                let _ = WinHttpSetTimeouts(session, 5_000, 5_000, 5_000, 5_000);
                let mut options = WINHTTP_AUTOPROXY_OPTIONS::default();
                if !config.lpszAutoConfigUrl.is_null() {
                    options.dwFlags = WINHTTP_AUTOPROXY_CONFIG_URL;
                    options.lpszAutoConfigUrl = PCWSTR(config.lpszAutoConfigUrl.0);
                } else {
                    options.dwFlags = WINHTTP_AUTOPROXY_AUTO_DETECT;
                    options.dwAutoDetectFlags =
                        WINHTTP_AUTO_DETECT_TYPE_DHCP | WINHTTP_AUTO_DETECT_TYPE_DNS_A;
                }
                let mut proxy = WINHTTP_PROXY_INFO::default();
                let target = wide_str(url);
                if WinHttpGetProxyForUrl(session, PCWSTR(target.as_ptr()), &mut options, &mut proxy)
                    .is_ok()
                {
                    // An empty string records a successful DIRECT decision.
                    resolved = Some(read_wide(proxy.lpszProxy).unwrap_or_default());
                }
                free_wide(proxy.lpszProxy);
                free_wide(proxy.lpszProxyBypass);
                let _ = WinHttpCloseHandle(session);
            }
        }
        free_wide(config.lpszAutoConfigUrl);
        free_wide(config.lpszProxy);
        free_wide(config.lpszProxyBypass);
        select_proxy(resolved.as_deref().or(static_proxy.as_deref())?, url)
    }
}

unsafe fn read_wide(value: PWSTR) -> Option<String> {
    if value.is_null() {
        None
    } else {
        value.to_string().ok()
    }
}

unsafe fn free_wide(value: PWSTR) {
    if !value.is_null() {
        let _ = GlobalFree(HGLOBAL(value.0.cast()));
    }
}

fn select_proxy(list: &str, url: &str) -> Option<String> {
    let scheme = if url.starts_with("https:") {
        "https"
    } else {
        "http"
    };
    let mut generic = None;
    for entry in list.split([';', ' ']).filter(|entry| !entry.is_empty()) {
        let proxy = if let Some((protocol, address)) = entry.split_once('=') {
            if protocol.eq_ignore_ascii_case(scheme) {
                Some(address)
            } else {
                None
            }
        } else {
            generic.get_or_insert(entry);
            None
        };
        if let Some(proxy) = proxy {
            return proxy_url(proxy);
        }
    }
    generic.and_then(proxy_url)
}

fn proxy_url(proxy: &str) -> Option<String> {
    if proxy.is_empty() || proxy.eq_ignore_ascii_case("DIRECT") {
        return None;
    }
    Some(if proxy.contains("://") {
        proxy.to_string()
    } else {
        format!("http://{proxy}")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects_https_proxy_in_protocol_specific_windows_list() {
        assert_eq!(
            select_proxy(
                "http=localhost:8000;https=localhost:9000",
                "https://chatgpt.com"
            ),
            Some("http://localhost:9000".into())
        );
        assert_eq!(
            select_proxy("localhost:49100", "https://chatgpt.com"),
            Some("http://localhost:49100".into())
        );
        assert_eq!(select_proxy("DIRECT", "https://chatgpt.com"), None);
    }
}
