use super::kernel_config::{FingerprintProfile, InstalledKernel, KernelVersion};
use super::kernel_store::KernelStore;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::process::Child;
use std::collections::HashMap;

/// 内核进程实例
pub struct KernelInstance {
    pub version: KernelVersion,
    pub process: Option<Child>,
    pub pid: Option<u32>,
    pub profile: FingerprintProfile,
}

/// 内核管理器 - 负责版本管理、进程控制
pub struct KernelManager {
    store: Arc<KernelStore>,
    running_instances: Arc<RwLock<HashMap<String, KernelInstance>>>,
    data_dir: PathBuf,
}

impl KernelManager {
    pub fn new(data_dir: PathBuf) -> Self {
        let store = Arc::new(KernelStore::new(data_dir.clone()));
        Self {
            store,
            running_instances: Arc::new(RwLock::new(HashMap::new())),
            data_dir,
        }
    }

    // ==================== 版本管理 ====================

    /// 获取所有已安装的内核版本
    pub async fn list_installed_kernels(&self) -> Result<Vec<InstalledKernel>, String> {
        self.store.load_kernels()
    }

    /// 获取默认内核
    pub async fn get_default_kernel(&self) -> Result<Option<InstalledKernel>, String> {
        self.store.get_default_kernel()
    }

    /// 注册新内核
    pub async fn register_kernel(
        &self,
        version: KernelVersion,
        install_path: PathBuf,
    ) -> Result<(), String> {
        let kernel = InstalledKernel::new(version, install_path, false);
        self.store.add_kernel(kernel)
    }

    /// 设置默认内核
    pub async fn set_default_kernel(&self, version: &KernelVersion) -> Result<(), String> {
        self.store.set_default_kernel(version)
    }

    /// 卸载内核
    pub async fn uninstall_kernel(&self, version: &KernelVersion) -> Result<(), String> {
        // 获取内核信息
        let kernels = self.store.load_kernels()?;
        if let Some(kernel) = kernels.iter().find(|k| k.version == *version) {
            // 删除文件系统中的内核
            if kernel.install_path.exists() {
                std::fs::remove_dir_all(&kernel.install_path)
                    .map_err(|e| format!("Failed to delete kernel files: {}", e))?
            }
        }
        self.store.remove_kernel(version)
    }

    // ==================== 指纹配置管理 ====================

    /// 获取所有指纹配置
    pub async fn list_profiles(&self) -> Result<Vec<FingerprintProfile>, String> {
        self.store.load_profiles()
    }

    /// 获取指纹配置
    pub async fn get_profile(&self, profile_id: &str) -> Result<Option<FingerprintProfile>, String> {
        self.store.get_profile(profile_id)
    }

    /// 创建指纹配置
    pub async fn create_profile(
        &self,
        name: String,
        seed: u32,
        platform: String,
        brand: String,
    ) -> Result<String, String> {
        let profile = FingerprintProfile::new(name, seed, platform, brand);
        let profile_id = profile.id.clone();
        self.store.add_profile(profile)?;
        Ok(profile_id)
    }

    /// 更新指纹配置
    pub async fn update_profile(&self, profile: FingerprintProfile) -> Result<(), String> {
        self.store.update_profile(profile)
    }

    /// 删除指纹配置
    pub async fn delete_profile(&self, profile_id: &str) -> Result<(), String> {
        self.store.remove_profile(profile_id)
    }

    /// 克隆指纹配置
    pub async fn clone_profile(&self, profile_id: &str, new_name: String) -> Result<String, String> {
        let profile = self
            .store
            .get_profile(profile_id)?
            .ok_or(format!("Profile {} not found", profile_id))?;

        let mut cloned = profile.clone();
        cloned.id = uuid::Uuid::v4().to_string();
        cloned.name = new_name;
        let new_id = cloned.id.clone();
        let now = chrono::Local::now().to_rfc3339();
        cloned.created_at = now.clone();
        cloned.updated_at = now;

        self.store.add_profile(cloned)?;
        Ok(new_id)
    }
}
