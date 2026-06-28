use serde::{Deserialize, Serialize};
use std::process::{Child, Command};
use std::path::PathBuf;
use tokio::sync::RwLock;
use std::sync::Arc;

/// 指纹配置选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FingerprintConfig {
    /// 指纹种子 (32位整数)
    pub seed: u32,
    
    /// 操作系统类型
    #[serde(rename = "platform")]
    pub os_platform: String, // "windows", "linux", "macos"
    
    /// 操作系统版本
    pub platform_version: Option<String>,
    
    /// 浏览器品牌
    pub brand: String, // "Chrome", "Edge", "Opera", "Vivaldi"
    
    /// 品牌版本号
    pub brand_version: Option<String>,
    
    /// CPU 核心数
    pub hardware_concurrency: Option<u32>,
    
    /// 禁用特定指纹伪装 (Chrome 144+)
    #[serde(default)]
    pub disable_spoofing: Vec<String>, // ["font", "audio", "canvas", "clientrects", "gpu"]
}

impl FingerprintConfig {
    /// 将配置转换为命令行参数
    pub fn to_cli_args(&self) -> Vec<String> {
        let mut args = vec![];
        
        args.push(format!("--fingerprint={}", self.seed));
        args.push(format!("--fingerprint-platform={}", self.os_platform));
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
        
        if !self.disable_spoofing.is_empty() {
            args.push(format!("--disable-spoofing={}", self.disable_spoofing.join(",")));
        }
        
        args
    }
}

/// 浏览器内核进程管理器
pub struct FingerprintChromiumKernel {
    process: Arc<RwLock<Option<Child>>>,
    config: FingerprintConfig,
    binary_path: PathBuf,
    user_data_dir: PathBuf,
}

impl FingerprintChromiumKernel {
    pub fn new(
        binary_path: PathBuf,
        user_data_dir: PathBuf,
        config: FingerprintConfig,
    ) -> Self {
        Self {
            process: Arc::new(RwLock::new(None)),
            config,
            binary_path,
            user_data_dir,
        }
    }
    
    /// 启动浏览器进程
    pub async fn start(&self) -> Result<u32, String> {
        let mut args = self.config.to_cli_args();
        args.push(format!("--user-data-dir={}", self.user_data_dir.display()));
        
        let mut cmd = Command::new(&self.binary_path);
        cmd.args(&args);
        
        match cmd.spawn() {
            Ok(child) => {
                let pid = child.id();
                let mut proc = self.process.write().await;
                *proc = Some(child);
                Ok(pid)
            }
            Err(e) => Err(format!("Failed to start fingerprint-chromium: {}", e)),
        }
    }
    
    /// 停止浏览器进程
    pub async fn stop(&self) -> Result<(), String> {
        let mut proc = self.process.write().await;
        if let Some(mut child) = proc.take() {
            child.kill().map_err(|e| format!("Failed to kill process: {}", e))?;
            child.wait().map_err(|e| format!("Failed to wait for process: {}", e))?;
        }
        Ok(())
    }
    
    /// 获取进程 ID
    pub async fn pid(&self) -> Option<u32> {
        let proc = self.process.read().await;
        proc.as_ref().map(|p| p.id())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fingerprint_config_to_cli_args() {
        let config = FingerprintConfig {
            seed: 1000,
            os_platform: "macos".to_string(),
            platform_version: Some("15.2.0".to_string()),
            brand: "Edge".to_string(),
            brand_version: Some("131.0".to_string()),
            hardware_concurrency: Some(8),
            disable_spoofing: vec!["gpu".to_string(), "font".to_string()],
        };
        
        let args = config.to_cli_args();
        assert!(args.contains(&"--fingerprint=1000".to_string()));
        assert!(args.contains(&"--fingerprint-platform=macos".to_string()));
        assert!(args.contains(&"--fingerprint-brand=Edge".to_string()));
        assert!(args.contains(&"--disable-spoofing=gpu,font".to_string()));
    }
}
