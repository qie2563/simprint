//! 浏览器内核实例池管理
//! 支持同时运行多个 fingerprint-chromium 实例

use super::kernel_config::FingerprintProfile;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Child;
use tokio::sync::RwLock;
use std::sync::Arc;

/// 单个浏览器实例信息
#[derive(Clone)]
pub struct BrowserInstance {
    /// 实例 ID（唯一标识）
    pub instance_id: String,
    /// 关联的环境 UUID
    pub env_uuid: String,
    /// 关联的指纹配置 ID
    pub profile_id: String,
    /// 进程 ID
    pub pid: Option<u32>,
    /// 用户数据目录
    pub user_data_dir: PathBuf,
    /// 状态
    pub status: BrowserInstanceStatus,
    /// 创建时间
    pub created_at: String,
    /// 启动时间
    pub started_at: Option<String>,
    /// 停止时间
    pub stopped_at: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum BrowserInstanceStatus {
    /// 已创建，未启动
    Created,
    /// 启动中
    Starting,
    /// 运行中
    Running,
    /// 停止中
    Stopping,
    /// 已停止
    Stopped,
    /// 错误
    Error(String),
}

/// 浏览器实例池 - 管理所有运行的 fingerprint-chromium 实例
pub struct BrowserInstancePool {
    /// 实例映射：instance_id -> BrowserInstance
    instances: Arc<RwLock<HashMap<String, BrowserInstance>>>,
    /// 进程映射：instance_id -> Child
    processes: Arc<RwLock<HashMap<String, Child>>>,
}

impl BrowserInstancePool {
    pub fn new() -> Self {
        Self {
            instances: Arc::new(RwLock::new(HashMap::new())),
            processes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 创建新实例
    pub async fn create_instance(
        &self,
        instance_id: String,
        env_uuid: String,
        profile_id: String,
        user_data_dir: PathBuf,
    ) -> Result<String, String> {
        let now = chrono::Local::now().to_rfc3339();
        let instance = BrowserInstance {
            instance_id: instance_id.clone(),
            env_uuid,
            profile_id,
            pid: None,
            user_data_dir,
            status: BrowserInstanceStatus::Created,
            created_at: now,
            started_at: None,
            stopped_at: None,
        };

        let mut instances = self.instances.write().await;
        instances.insert(instance_id.clone(), instance);
        Ok(instance_id)
    }

    /// 更新实例状态
    pub async fn update_instance_status(
        &self,
        instance_id: &str,
        status: BrowserInstanceStatus,
    ) -> Result<(), String> {
        let mut instances = self.instances.write().await;
        if let Some(instance) = instances.get_mut(instance_id) {
            instance.status = status.clone();
            if status == BrowserInstanceStatus::Running {
                instance.started_at = Some(chrono::Local::now().to_rfc3339());
            }
            if status == BrowserInstanceStatus::Stopped {
                instance.stopped_at = Some(chrono::Local::now().to_rfc3339());
            }
            Ok(())
        } else {
            Err(format!("Instance {} not found", instance_id))
        }
    }

    /// 设置进程 ID
    pub async fn set_instance_pid(
        &self,
        instance_id: &str,
        pid: u32,
    ) -> Result<(), String> {
        let mut instances = self.instances.write().await;
        if let Some(instance) = instances.get_mut(instance_id) {
            instance.pid = Some(pid);
            Ok(())
        } else {
            Err(format!("Instance {} not found", instance_id))
        }
    }

    /// 获取实例信息
    pub async fn get_instance(&self, instance_id: &str) -> Result<Option<BrowserInstance>, String> {
        let instances = self.instances.read().await;
        Ok(instances.get(instance_id).cloned())
    }

    /// 获取环境关联的所有实例
    pub async fn get_env_instances(
        &self,
        env_uuid: &str,
    ) -> Result<Vec<BrowserInstance>, String> {
        let instances = self.instances.read().await;
        Ok(instances
            .values()
            .filter(|i| i.env_uuid == env_uuid)
            .cloned()
            .collect())
    }

    /// 获取运行中的实例
    pub async fn get_running_instances(&self) -> Result<Vec<BrowserInstance>, String> {
        let instances = self.instances.read().await;
        Ok(instances
            .values()
            .filter(|i| i.status == BrowserInstanceStatus::Running)
            .cloned()
            .collect())
    }

    /// 删除实例记录
    pub async fn remove_instance(&self, instance_id: &str) -> Result<(), String> {
        let mut instances = self.instances.write().await;
        let mut processes = self.processes.write().await;
        instances.remove(instance_id);
        processes.remove(instance_id);
        Ok(())
    }

    /// 存储进程引用
    pub async fn store_process(
        &self,
        instance_id: String,
        process: Child,
    ) -> Result<(), String> {
        let mut processes = self.processes.write().await;
        processes.insert(instance_id, process);
        Ok(())
    }

    /// 获取进程引用并删除
    pub async fn take_process(
        &self,
        instance_id: &str,
    ) -> Result<Option<Child>, String> {
        let mut processes = self.processes.write().await;
        Ok(processes.remove(instance_id))
    }

    /// 统计运行中的实例数
    pub async fn count_running(&self) -> Result<usize, String> {
        let instances = self.instances.read().await;
        Ok(instances
            .values()
            .filter(|i| i.status == BrowserInstanceStatus::Running)
            .count())
    }
}

impl Default for BrowserInstancePool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_instance_pool() {
        let pool = BrowserInstancePool::new();

        let instance_id = pool
            .create_instance(
                "inst-1".to_string(),
                "env-uuid-1".to_string(),
                "profile-1".to_string(),
                PathBuf::from("/tmp/browser-1"),
            )
            .await
            .unwrap();

        let instance = pool.get_instance(&instance_id).await.unwrap();
        assert!(instance.is_some());
        assert_eq!(instance.unwrap().env_uuid, "env-uuid-1");

        pool.update_instance_status(&instance_id, BrowserInstanceStatus::Running)
            .await
            .unwrap();

        let running_count = pool.count_running().await.unwrap();
        assert_eq!(running_count, 1);
    }
}
