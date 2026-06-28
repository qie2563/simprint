use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 内核版本信息
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct KernelVersion {
    /// 主版本号
    pub major: u32,
    /// 次版本号
    pub minor: u32,
    /// 补丁版本号
    pub patch: u32,
}

impl KernelVersion {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    pub fn to_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }

    pub fn from_string(version_str: &str) -> Result<Self, String> {
        let parts: Vec<&str> = version_str.split('.').collect();
        if parts.len() != 3 {
            return Err(format!("Invalid version format: {}", version_str));
        }
        Ok(Self {
            major: parts[0].parse().map_err(|e| format!("Invalid major version: {}", e))?,
            minor: parts[1].parse().map_err(|e| format!("Invalid minor version: {}", e))?,
            patch: parts[2].parse().map_err(|e| format!("Invalid patch version: {}", e))?,
        })
    }
}

/// 内核发行版信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelRelease {
    /// 版本
    pub version: KernelVersion,
    /// 发行日期
    pub release_date: String,
    /// 支持的平台列表
    pub platforms: Vec<String>, // "windows-x64", "linux-x64", "macos-x64", "macos-arm64"
    /// 校验和
    pub checksums: std::collections::HashMap<String, String>,
}

impl KernelRelease {
    pub fn supports_platform(&self, platform: &str) -> bool {
        self.platforms.contains(&platform.to_string())
    }
}

/// 已安装内核信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledKernel {
    /// 版本
    pub version: KernelVersion,
    /// 安装路径
    pub install_path: PathBuf,
    /// 可执行文件路径
    pub binary_path: PathBuf,
    /// 安装时间
    pub installed_at: String,
    /// 是否为默认内核
    pub is_default: bool,
}

impl InstalledKernel {
    pub fn new(
        version: KernelVersion,
        install_path: PathBuf,
        is_default: bool,
    ) -> Self {
        let binary_path = Self::get_binary_path(&install_path);
        Self {
            version,
            install_path,
            binary_path,
            installed_at: chrono::Local::now().to_rfc3339(),
            is_default,
        }
    }

    fn get_binary_path(install_path: &PathBuf) -> PathBuf {
        #[cfg(target_os = "windows")]
        {
            install_path.join("chrome.exe")
        }
        #[cfg(target_os = "macos")]
        {
            install_path.join("Chromium.app/Contents/MacOS/Chromium")
        }
        #[cfg(target_os = "linux")]
        {
            install_path.join("chrome")
        }
    }
}

/// 指纹配置选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FingerprintProfile {
    /// 配置 ID
    pub id: String,
    /// 配置名称
    pub name: String,
    /// 指纹种子
    pub seed: u32,
    /// 操作系统
    pub platform: String, // "windows", "linux", "macos"
    /// 操作系统版本
    pub platform_version: Option<String>,
    /// 浏览器品牌
    pub brand: String, // "Chrome", "Edge", "Opera", "Vivaldi", "Chromium"
    /// 品牌版本
    pub brand_version: Option<String>,
    /// CPU 核心数
    pub hardware_concurrency: Option<u32>,
    /// 语言
    pub language: Option<String>,
    /// 接受的语言列表
    pub accept_languages: Option<String>,
    /// 时区
    pub timezone: Option<String>,
    /// 代理服务器
    pub proxy_server: Option<String>,
    /// 禁用的指纹伪装功能
    #[serde(default)]
    pub disable_spoofing: Vec<String>, // ["font", "audio", "canvas", "clientrects", "gpu"]
    /// 创建时间
    pub created_at: String,
    /// 更新时间
    pub updated_at: String,
    /// 描述
    pub description: Option<String>,
}

impl FingerprintProfile {
    pub fn new(name: String, seed: u32, platform: String, brand: String) -> Self {
        let now = chrono::Local::now().to_rfc3339();
        Self {
            id: uuid::Uuid::v4().to_string(),
            name,
            seed,
            platform,
            platform_version: None,
            brand,
            brand_version: None,
            hardware_concurrency: None,
            language: None,
            accept_languages: None,
            timezone: None,
            proxy_server: None,
            disable_spoofing: vec![],
            created_at: now.clone(),
            updated_at: now,
            description: None,
        }
    }

    /// 转换为命令行参数
    pub fn to_cli_args(&self) -> Vec<String> {
        let mut args = vec![];

        args.push(format!("--fingerprint={}", self.seed));
        args.push(format!("--fingerprint-platform={}", self.platform));
        args.push(format!("--fingerprint-brand={}", self.brand));

        if let Some(version) = &self.platform_version {
            args.push(format!("--fingerprint-platform-version={}", version));
        }

        if let Some(version) = &self.brand_version {
            args.push(format!("--fingerprint-brand-version={}", version));
        }

        if let Some(concurrency) = self.hardware_concurrency {
            args.push(format!("--fingerprint-hardware-concurrency={}", concurrency));
        }

        if let Some(lang) = &self.language {
            args.push(format!("--lang={}", lang));
        }

        if let Some(langs) = &self.accept_languages {
            args.push(format!("--accept-lang={}", langs));
        }

        if let Some(tz) = &self.timezone {
            args.push(format!("--timezone={}", tz));
        }

        if let Some(proxy) = &self.proxy_server {
            args.push(format!("--proxy-server={}", proxy));
        }

        if !self.disable_spoofing.is_empty() {
            args.push(format!("--disable-spoofing={}", self.disable_spoofing.join(",")));
        }

        args
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kernel_version_parse() {
        let version = KernelVersion::from_string("148.0.7778").unwrap();
        assert_eq!(version.major, 148);
        assert_eq!(version.minor, 0);
        assert_eq!(version.patch, 7778);
    }

    #[test]
    fn test_fingerprint_profile_cli_args() {
        let mut profile = FingerprintProfile::new(
            "测试指纹".to_string(),
            1000,
            "windows".to_string(),
            "Edge".to_string(),
        );
        profile.timezone = Some("America/New_York".to_string());
        profile.disable_spoofing = vec!["gpu".to_string()];

        let args = profile.to_cli_args();
        assert!(args.contains(&"--fingerprint=1000".to_string()));
        assert!(args.contains(&"--disable-spoofing=gpu".to_string()));
    }
}
