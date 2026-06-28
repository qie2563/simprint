use super::kernel_config::{FingerprintProfile, InstalledKernel, KernelVersion};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::io::Write;

/// 内核存储管理器 - 负责读写 JSON 配置文件
pub struct KernelStore {
    data_dir: PathBuf,
}

impl KernelStore {
    pub fn new(data_dir: PathBuf) -> Self {
        Self { data_dir }
    }

    /// 确保目录存在
    fn ensure_dir(&self) -> Result<(), String> {
        fs::create_dir_all(&self.data_dir)
            .map_err(|e| format!("Failed to create data directory: {}", e))
    }

    fn get_kernels_file(&self) -> PathBuf {
        self.data_dir.join("installed_kernels.json")
    }

    fn get_profiles_file(&self) -> PathBuf {
        self.data_dir.join("fingerprint_profiles.json")
    }

    fn get_metadata_file(&self) -> PathBuf {
        self.data_dir.join("kernel_metadata.json")
    }

    // ==================== 内核管理 ====================

    /// 加载所有已安装的内核
    pub fn load_kernels(&self) -> Result<Vec<InstalledKernel>, String> {
        self.ensure_dir()?;
        let path = self.get_kernels_file();

        if !path.exists() {
            return Ok(vec![]);
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read kernels file: {}", e))?;
        let kernels: Vec<InstalledKernel> = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse kernels JSON: {}", e))?;

        Ok(kernels)
    }

    /// 保存内核信息
    pub fn save_kernels(&self, kernels: &[InstalledKernel]) -> Result<(), String> {
        self.ensure_dir()?;
        let path = self.get_kernels_file();
        let content = serde_json::to_string_pretty(kernels)
            .map_err(|e| format!("Failed to serialize kernels: {}", e))?;

        let mut file = fs::File::create(&path)
            .map_err(|e| format!("Failed to create kernels file: {}", e))?;
        file.write_all(content.as_bytes())
            .map_err(|e| format!("Failed to write kernels file: {}", e))?;

        Ok(())
    }

    /// 添加新内核
    pub fn add_kernel(&self, kernel: InstalledKernel) -> Result<(), String> {
        let mut kernels = self.load_kernels()?;
        kernels.push(kernel);
        self.save_kernels(&kernels)
    }

    /// 获取默认内核
    pub fn get_default_kernel(&self) -> Result<Option<InstalledKernel>, String> {
        let kernels = self.load_kernels()?;
        Ok(kernels.into_iter().find(|k| k.is_default))
    }

    /// 设置默认内核
    pub fn set_default_kernel(&self, version: &KernelVersion) -> Result<(), String> {
        let mut kernels = self.load_kernels()?;
        for kernel in &mut kernels {
            kernel.is_default = kernel.version == *version;
        }
        self.save_kernels(&kernels)
    }

    /// 删除内核
    pub fn remove_kernel(&self, version: &KernelVersion) -> Result<(), String> {
        let mut kernels = self.load_kernels()?;
        kernels.retain(|k| k.version != *version);
        self.save_kernels(&kernels)
    }

    // ==================== 指纹配置管理 ====================

    /// 加载所有指纹配置
    pub fn load_profiles(&self) -> Result<Vec<FingerprintProfile>, String> {
        self.ensure_dir()?;
        let path = self.get_profiles_file();

        if !path.exists() {
            return Ok(vec![]);
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read profiles file: {}", e))?;
        let profiles: Vec<FingerprintProfile> = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse profiles JSON: {}", e))?;

        Ok(profiles)
    }

    /// 保存指纹配置
    pub fn save_profiles(&self, profiles: &[FingerprintProfile]) -> Result<(), String> {
        self.ensure_dir()?;
        let path = self.get_profiles_file();
        let content = serde_json::to_string_pretty(profiles)
            .map_err(|e| format!("Failed to serialize profiles: {}", e))?;

        let mut file = fs::File::create(&path)
            .map_err(|e| format!("Failed to create profiles file: {}", e))?;
        file.write_all(content.as_bytes())
            .map_err(|e| format!("Failed to write profiles file: {}", e))?;

        Ok(())
    }

    /// 添加指纹配置
    pub fn add_profile(&self, profile: FingerprintProfile) -> Result<(), String> {
        let mut profiles = self.load_profiles()?;
        profiles.push(profile);
        self.save_profiles(&profiles)
    }

    /// 更新指纹配置
    pub fn update_profile(&self, profile: FingerprintProfile) -> Result<(), String> {
        let mut profiles = self.load_profiles()?;
        let pos = profiles.iter().position(|p| p.id == profile.id);
        if let Some(index) = pos {
            profiles[index] = profile;
            self.save_profiles(&profiles)?;
            Ok(())
        } else {
            Err(format!("Profile {} not found", profile.id))
        }
    }

    /// 删除指纹配置
    pub fn remove_profile(&self, profile_id: &str) -> Result<(), String> {
        let mut profiles = self.load_profiles()?;
        profiles.retain(|p| p.id != profile_id);
        self.save_profiles(&profiles)
    }

    /// 获取指纹配置
    pub fn get_profile(&self, profile_id: &str) -> Result<Option<FingerprintProfile>, String> {
        let profiles = self.load_profiles()?;
        Ok(profiles.into_iter().find(|p| p.id == profile_id))
    }

    // ==================== 元数据管理 ====================

    /// 保存元数据（可用的内核版本列表等）
    pub fn save_metadata(&self, metadata: Value) -> Result<(), String> {
        self.ensure_dir()?;
        let path = self.get_metadata_file();
        let content = serde_json::to_string_pretty(&metadata)
            .map_err(|e| format!("Failed to serialize metadata: {}", e))?;

        let mut file = fs::File::create(&path)
            .map_err(|e| format!("Failed to create metadata file: {}", e))?;
        file.write_all(content.as_bytes())
            .map_err(|e| format!("Failed to write metadata file: {}", e))?;

        Ok(())
    }

    /// 加载元数据
    pub fn load_metadata(&self) -> Result<Value, String> {
        self.ensure_dir()?;
        let path = self.get_metadata_file();

        if !path.exists() {
            return Ok(json!({}));
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read metadata file: {}", e))?;
        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse metadata JSON: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_kernel_store_basic() {
        let temp_dir = TempDir::new().unwrap();
        let store = KernelStore::new(temp_dir.path().to_path_buf());

        // 加载空内核列表
        let kernels = store.load_kernels().unwrap();
        assert_eq!(kernels.len(), 0);

        // 添加内核
        let kernel = InstalledKernel::new(
            KernelVersion::new(148, 0, 7778),
            temp_dir.path().join("148.0.7778"),
            true,
        );
        store.add_kernel(kernel.clone()).unwrap();

        // 加载并验证
        let kernels = store.load_kernels().unwrap();
        assert_eq!(kernels.len(), 1);
        assert_eq!(kernels[0].version, KernelVersion::new(148, 0, 7778));
    }

    #[test]
    fn test_fingerprint_profile_store() {
        let temp_dir = TempDir::new().unwrap();
        let store = KernelStore::new(temp_dir.path().to_path_buf());

        let profile = FingerprintProfile::new(
            "测试配置".to_string(),
            1000,
            "windows".to_string(),
            "Edge".to_string(),
        );
        let profile_id = profile.id.clone();

        store.add_profile(profile).unwrap();

        let loaded = store.get_profile(&profile_id).unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().name, "测试配置");
    }
}
