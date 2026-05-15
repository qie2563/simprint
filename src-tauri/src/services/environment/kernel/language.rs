//! 语言自动检测模块

use super::types::{ProxyConfig, TIMEZONE_DETECTION_TIMEOUT_SECS};

/// 检测语言
///
/// # 逻辑
/// - 有代理配置：先用代理检测，失败则直连检测
/// - 无代理配置：使用系统代理（reqwest 自动检测）
///
/// # 返回
/// - `Some(language)`: 检测成功
/// - `None`: 所有检测都失败
pub async fn detect_language(proxy: Option<&ProxyConfig>) -> Option<String> {
    if let Some(proxy_cfg) = proxy {
        detect_with_proxy(proxy_cfg).await
    } else {
        detect_with_system_proxy().await
    }
}

async fn detect_with_proxy(proxy_cfg: &ProxyConfig) -> Option<String> {
    let infra_proxy_cfg = proxy_cfg.to_infrastructure_proxy_config();
    let proxy_client = match crate::infrastructure::proxy::client::build_proxy_client(
        &infra_proxy_cfg,
        Some(TIMEZONE_DETECTION_TIMEOUT_SECS),
    ) {
        Ok(client) => client,
        Err(_) => {
            return detect_with_direct_connection().await;
        }
    };

    let proxy_result = crate::infrastructure::proxy::detector::detect_ip_with_timeout(
        proxy_client,
        TIMEZONE_DETECTION_TIMEOUT_SECS,
    )
    .await;

    if proxy_result.success {
        let ip_info = proxy_result.ip_info.as_ref()?;
        infer_language_from_country_code(ip_info.country_code.as_str())
    } else {
        detect_with_direct_connection().await
    }
}

async fn detect_with_system_proxy() -> Option<String> {
    let result = crate::infrastructure::proxy::detector::detect_with_system_proxy(
        TIMEZONE_DETECTION_TIMEOUT_SECS,
    )
    .await;

    if result.success {
        let ip_info = result.ip_info.as_ref()?;
        infer_language_from_country_code(ip_info.country_code.as_str())
    } else {
        None
    }
}

async fn detect_with_direct_connection() -> Option<String> {
    let local_result = crate::infrastructure::proxy::detector::detect_direct_ip_with_timeout(
        TIMEZONE_DETECTION_TIMEOUT_SECS,
    )
    .await;

    if local_result.success {
        let ip_info = local_result.ip_info.as_ref()?;
        infer_language_from_country_code(ip_info.country_code.as_str())
    } else {
        None
    }
}

fn infer_language_from_country_code(country_code: &str) -> Option<String> {
    let normalized = country_code.trim().to_ascii_uppercase();
    if normalized.is_empty() {
        return None;
    }

    let language = match normalized.as_str() {
        "CN" => "zh-CN",
        "TW" => "zh-TW",
        "HK" => "zh-HK",
        "US" => "en-US",
        "GB" => "en-GB",
        "CA" => "en-CA",
        "AU" => "en-AU",
        "JP" => "ja-JP",
        "KR" => "ko-KR",
        "FR" => "fr-FR",
        "DE" => "de-DE",
        "ES" => "es-ES",
        "IT" => "it-IT",
        "RU" => "ru-RU",
        "BR" => "pt-BR",
        "MX" => "es-MX",
        "IN" => "en-IN",
        "SG" => "en-SG",
        _ => "en-US",
    };

    Some(language.to_string())
}
