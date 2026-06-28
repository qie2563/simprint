//! Fingerprint Chromium 浏览器实例命令接口
//! 支持同时运行多个 fingerprint-chromium 实例

use crate::modules::kernel_instance_pool::{BrowserInstancePool, BrowserInstanceStatus};
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tauri::State;
use uuid::Uuid;

pub struct BrowserInstancePoolState(pub Arc<RwLock<BrowserInstancePool>>);

/// 启动浏览器实例
#[tauri::command]
pub async fn start_browser_instance(
    env_uuid: String,
    profile_id: String,
    user_data_dir: Option<String>,
    pool: State<'_, BrowserInstancePoolState>,
) -> Result<String, String> {
    let instance_id = format!("inst-{}", Uuid::v4());
    let user_data_dir = PathBuf::from(
        user_data_dir
            .unwrap_or_else(|| format!("/tmp/simprint/browser/{}", Uuid::v4())),
    );

    let pool = pool.0.write().await;
    pool.create_instance(instance_id.clone(), env_uuid, profile_id, user_data_dir)
        .await
}

/// 获取实例状态
#[tauri::command]
pub async fn get_browser_instance_status(
    instance_id: String,
    pool: State<'_, BrowserInstancePoolState>,
) -> Result<Option<serde_json::Value>, String> {
    let pool = pool.0.read().await;
    let instance = pool.get_instance(&instance_id).await?;
    Ok(instance.map(|inst| {
        json!({
            "instance_id": inst.instance_id,
            "env_uuid": inst.env_uuid,
            "profile_id": inst.profile_id,
            "pid": inst.pid,
            "status": format!("{:?}", inst.status),
            "created_at": inst.created_at,
            "started_at": inst.started_at,
            "stopped_at": inst.stopped_at,
        })
    }))
}

/// 获取环境下的所有实例
#[tauri::command]
pub async fn get_env_browser_instances(
    env_uuid: String,
    pool: State<'_, BrowserInstancePoolState>,
) -> Result<Vec<serde_json::Value>, String> {
    let pool = pool.0.read().await;
    let instances = pool.get_env_instances(&env_uuid).await?;
    Ok(instances
        .into_iter()
        .map(|inst| {
            json!({
                "instance_id": inst.instance_id,
                "env_uuid": inst.env_uuid,
                "profile_id": inst.profile_id,
                "pid": inst.pid,
                "status": format!("{:?}", inst.status),
                "created_at": inst.created_at,
                "started_at": inst.started_at,
                "stopped_at": inst.stopped_at,
            })
        })
        .collect())
}

/// 获取所有运行中的实例
#[tauri::command]
pub async fn get_running_browser_instances(
    pool: State<'_, BrowserInstancePoolState>,
) -> Result<Vec<serde_json::Value>, String> {
    let pool = pool.0.read().await;
    let instances = pool.get_running_instances().await?;
    Ok(instances
        .into_iter()
        .map(|inst| {
            json!({
                "instance_id": inst.instance_id,
                "env_uuid": inst.env_uuid,
                "profile_id": inst.profile_id,
                "pid": inst.pid,
                "created_at": inst.created_at,
                "started_at": inst.started_at,
            })
        })
        .collect())
}

/// 停止实例
#[tauri::command]
pub async fn stop_browser_instance(
    instance_id: String,
    pool: State<'_, BrowserInstancePoolState>,
) -> Result<(), String> {
    let pool = pool.0.write().await;
    pool.update_instance_status(&instance_id, BrowserInstanceStatus::Stopping)
        .await?;
    // 这里应该调用实际的进程杀死逻辑
    pool.update_instance_status(&instance_id, BrowserInstanceStatus::Stopped)
        .await
}

/// 删除实例记录
#[tauri::command]
pub async fn remove_browser_instance(
    instance_id: String,
    pool: State<'_, BrowserInstancePoolState>,
) -> Result<(), String> {
    let pool = pool.0.write().await;
    pool.remove_instance(&instance_id).await
}

/// 获取运行中的实例数
#[tauri::command]
pub async fn count_running_browser_instances(
    pool: State<'_, BrowserInstancePoolState>,
) -> Result<usize, String> {
    let pool = pool.0.read().await;
    pool.count_running().await
}
