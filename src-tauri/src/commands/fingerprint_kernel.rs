//! Fingerprint Chromium 浏览器内核 Tauri 命令接口
//! 提供与前端通信的 RPC 接口

use crate::modules::kernel_config::{FingerprintProfile, InstalledKernel, KernelVersion};
use crate::modules::kernel_manager::KernelManager;
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tauri::State;

/// 全局内核管理器状态
pub struct KernelManagerState(pub Arc<RwLock<KernelManager>>);

/// 列出所有已安装的内核版本
#[tauri::command]
pub async fn list_fingerprint_kernels(
    manager: State<'_, KernelManagerState>,
) -> Result<Vec<InstalledKernel>, String> {
    let manager = manager.0.read().await;
    manager.list_installed_kernels().await
}

/// 获取默认内核
#[tauri::command]
pub async fn get_default_fingerprint_kernel(
    manager: State<'_, KernelManagerState>,
) -> Result<Option<InstalledKernel>, String> {
    let manager = manager.0.read().await;
    manager.get_default_kernel().await
}

/// 注册新内核（从外部路径）
#[tauri::command]
pub async fn register_fingerprint_kernel(
    version: String,
    path: String,
    manager: State<'_, KernelManagerState>,
) -> Result<(), String> {
    let version = KernelVersion::from_string(&version)?;
    let path = PathBuf::from(&path);
    let manager = manager.0.write().await;
    manager.register_kernel(version, path).await
}

/// 设置默认内核
#[tauri::command]
pub async fn set_default_fingerprint_kernel(
    version: String,
    manager: State<'_, KernelManagerState>,
) -> Result<(), String> {
    let version = KernelVersion::from_string(&version)?;
    let manager = manager.0.write().await;
    manager.set_default_kernel(&version).await
}

/// 卸载内核
#[tauri::command]
pub async fn uninstall_fingerprint_kernel(
    version: String,
    manager: State<'_, KernelManagerState>,
) -> Result<(), String> {
    let version = KernelVersion::from_string(&version)?;
    let manager = manager.0.write().await;
    manager.uninstall_kernel(&version).await
}

// ==================== 指纹配置管理 ====================

/// 列出所有指纹配置
#[tauri::command]
pub async fn list_fingerprint_profiles(
    manager: State<'_, KernelManagerState>,
) -> Result<Vec<FingerprintProfile>, String> {
    let manager = manager.0.read().await;
    manager.list_profiles().await
}

/// 获取指纹配置
#[tauri::command]
pub async fn get_fingerprint_profile(
    profile_id: String,
    manager: State<'_, KernelManagerState>,
) -> Result<Option<FingerprintProfile>, String> {
    let manager = manager.0.read().await;
    manager.get_profile(&profile_id).await
}

/// 创建指纹配置
#[tauri::command]
pub async fn create_fingerprint_profile(
    name: String,
    seed: u32,
    platform: String,
    brand: String,
    manager: State<'_, KernelManagerState>,
) -> Result<String, String> {
    let manager = manager.0.write().await;
    manager
        .create_profile(name, seed, platform, brand)
        .await
}

/// 更新指纹配置
#[tauri::command]
pub async fn update_fingerprint_profile(
    profile: FingerprintProfile,
    manager: State<'_, KernelManagerState>,
) -> Result<(), String> {
    let manager = manager.0.write().await;
    manager.update_profile(profile).await
}

/// 删除指纹配置
#[tauri::command]
pub async fn delete_fingerprint_profile(
    profile_id: String,
    manager: State<'_, KernelManagerState>,
) -> Result<(), String> {
    let manager = manager.0.write().await;
    manager.delete_profile(&profile_id).await
}

/// 克隆指纹配置
#[tauri::command]
pub async fn clone_fingerprint_profile(
    profile_id: String,
    new_name: String,
    manager: State<'_, KernelManagerState>,
) -> Result<String, String> {
    let manager = manager.0.write().await;
    manager.clone_profile(&profile_id, new_name).await
}

// ==================== 预设配置 ====================

/// 预设指纹配置模板
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct FingerprintPreset {
    pub id: String,
    pub name: String,
    pub description: String,
    pub seed: u32,
    pub platform: String,
    pub platform_version: Option<String>,
    pub brand: String,
    pub brand_version: Option<String>,
    pub hardware_concurrency: Option<u32>,
    pub language: Option<String>,
    pub timezone: Option<String>,
    pub disable_spoofing: Vec<String>,
}

/// 获取所有预设配置
#[tauri::command]
pub async fn list_fingerprint_presets() -> Result<Vec<FingerprintPreset>, String> {
    Ok(vec![
        // 美国游客模式
        FingerprintPreset {
            id: "preset_us_visitor".to_string(),
            name: "美国游客".to_string(),
            description: "模拟美国 Chrome 浏览器环境".to_string(),
            seed: 2024,
            platform: "windows".to_string(),
            platform_version: Some("10.0".to_string()),
            brand: "Chrome".to_string(),
            brand_version: Some("131.0".to_string()),
            hardware_concurrency: Some(8),
            language: Some("en-US".to_string()),
            timezone: Some("America/New_York".to_string()),
            disable_spoofing: vec![],
        },
        // 新加坡住宅代理模式
        FingerprintPreset {
            id: "preset_sg_residential".to_string(),
            name: "新加坡住宅".to_string(),
            description: "模拟新加坡住宅 IP Edge 浏览器".to_string(),
            seed: 1688,
            platform: "windows".to_string(),
            platform_version: Some("11.0".to_string()),
            brand: "Edge".to_string(),
            brand_version: Some("131.0".to_string()),
            hardware_concurrency: Some(4),
            language: Some("en-SG".to_string()),
            timezone: Some("Asia/Singapore".to_string()),
            disable_spoofing: vec!["gpu".to_string()],
        },
        // macOS 高端用户模式
        FingerprintPreset {
            id: "preset_macos_premium".to_string(),
            name: "macOS 高端用户".to_string(),
            description: "模拟 macOS Safari 高端用户".to_string(),
            seed: 3333,
            platform: "macos".to_string(),
            platform_version: Some("14.2".to_string()),
            brand: "Safari".to_string(),
            brand_version: Some("17.2".to_string()),
            hardware_concurrency: Some(16),
            language: Some("en-US".to_string()),
            timezone: Some("America/Los_Angeles".to_string()),
            disable_spoofing: vec![],
        },
        // Linux 开发者模式
        FingerprintPreset {
            id: "preset_linux_dev".to_string(),
            name: "Linux 开发者".to_string(),
            description: "模拟 Linux Chromium 开发者环境".to_string(),
            seed: 5555,
            platform: "linux".to_string(),
            platform_version: None,
            brand: "Chromium".to_string(),
            brand_version: None,
            hardware_concurrency: Some(32),
            language: Some("en-US".to_string()),
            timezone: Some("UTC".to_string()),
            disable_spoofing: vec![],
        },
    ])
}

/// 从预设创建配置
#[tauri::command]
pub async fn create_profile_from_preset(
    preset_id: String,
    name: String,
    manager: State<'_, KernelManagerState>,
) -> Result<String, String> {
    let presets = list_fingerprint_presets().await?;
    let preset = presets
        .iter()
        .find(|p| p.id == preset_id)
        .ok_or(format!("Preset {} not found", preset_id))?
        .clone();

    let mut profile = FingerprintProfile::new(
        name,
        preset.seed,
        preset.platform,
        preset.brand,
    );
    profile.platform_version = preset.platform_version;
    profile.brand_version = preset.brand_version;
    profile.hardware_concurrency = preset.hardware_concurrency;
    profile.language = preset.language;
    profile.timezone = preset.timezone;
    profile.disable_spoofing = preset.disable_spoofing;

    let profile_id = profile.id.clone();
    let manager = manager.0.write().await;
    manager.add_profile(profile).map_err(|e| e.to_string())?;
    Ok(profile_id)
}
