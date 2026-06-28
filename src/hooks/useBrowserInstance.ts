import { useCallback, useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { BrowserInstance } from '../types/fingerprint';

/**
 * 浏览器实例管理 Hook
 * 负责浏览器实例的生命周期管理和状态监控
 */
export function useBrowserInstance() {
  const [instances, setInstances] = useState<BrowserInstance[]>([]);
  const [runningCount, setRunningCount] = useState(0);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // 启动浏览器实例
  const startInstance = useCallback(
    async (
      envUuid: string,
      profileId: string,
      userDataDir?: string
    ): Promise<string> => {
      try {
        const instanceId = await invoke<string>('start_browser_instance', {
          env_uuid: envUuid,
          profile_id: profileId,
          user_data_dir: userDataDir,
        });
        await refreshInstances();
        return instanceId;
      } catch (err) {
        const errorMsg = err instanceof Error ? err.message : String(err);
        throw new Error(`启动实例失败: ${errorMsg}`);
      }
    },
    []
  );

  // 停止浏览器实例
  const stopInstance = useCallback(async (instanceId: string) => {
    try {
      await invoke('stop_browser_instance', { instance_id: instanceId });
      await refreshInstances();
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      throw new Error(`停止实例失败: ${errorMsg}`);
    }
  }, []);

  // 删除实例记录
  const removeInstance = useCallback(async (instanceId: string) => {
    try {
      await invoke('remove_browser_instance', { instance_id: instanceId });
      await refreshInstances();
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      throw new Error(`删除实例失败: ${errorMsg}`);
    }
  }, []);

  // 获取单个实例状态
  const getInstanceStatus = useCallback(
    async (instanceId: string): Promise<BrowserInstance | null> => {
      try {
        const result = await invoke<any>('get_browser_instance_status', {
          instance_id: instanceId,
        });
        return result
          ? {
              instance_id: result.instance_id,
              env_uuid: result.env_uuid,
              profile_id: result.profile_id,
              pid: result.pid,
              status: result.status,
              created_at: result.created_at,
              started_at: result.started_at,
              stopped_at: result.stopped_at,
            }
          : null;
      } catch (err) {
        const errorMsg = err instanceof Error ? err.message : String(err);
        throw new Error(`获取实例状态失败: ${errorMsg}`);
      }
    },
    []
  );

  // 获取环境下的所有实例
  const getEnvInstances = useCallback(
    async (envUuid: string): Promise<BrowserInstance[]> => {
      try {
        const results = await invoke<any[]>('get_env_browser_instances', {
          env_uuid: envUuid,
        });
        return results.map((r) => ({
          instance_id: r.instance_id,
          env_uuid: r.env_uuid,
          profile_id: r.profile_id,
          pid: r.pid,
          status: r.status,
          created_at: r.created_at,
          started_at: r.started_at,
          stopped_at: r.stopped_at,
        }));
      } catch (err) {
        const errorMsg = err instanceof Error ? err.message : String(err);
        throw new Error(`获取环境实例失败: ${errorMsg}`);
      }
    },
    []
  );

  // 刷新所有实例信息
  const refreshInstances = useCallback(async () => {
    setLoading(true);
    try {
      const results = await invoke<any[]>('get_running_browser_instances');
      const formattedInstances: BrowserInstance[] = results.map((r) => ({
        instance_id: r.instance_id,
        env_uuid: r.env_uuid,
        profile_id: r.profile_id,
        pid: r.pid,
        status: r.status,
        created_at: r.created_at,
        started_at: r.started_at,
      }));
      setInstances(formattedInstances);
      setRunningCount(formattedInstances.length);
      setError(null);
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      setError(`刷新实例失败: ${errorMsg}`);
    } finally {
      setLoading(false);
    }
  }, []);

  // 获取运行中实例数
  const getRunningCount = useCallback(async (): Promise<number> => {
    try {
      const count = await invoke<number>('count_running_browser_instances');
      setRunningCount(count);
      return count;
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      throw new Error(`获取运行实例数失败: ${errorMsg}`);
    }
  }, []);

  // 初始加载和定时刷新
  useEffect(() => {
    refreshInstances();
    // 每 5 秒刷新一次实例状态
    const interval = setInterval(refreshInstances, 5000);
    return () => clearInterval(interval);
  }, [refreshInstances]);

  return {
    instances,
    runningCount,
    loading,
    error,
    startInstance,
    stopInstance,
    removeInstance,
    getInstanceStatus,
    getEnvInstances,
    refreshInstances,
    getRunningCount,
  };
}
