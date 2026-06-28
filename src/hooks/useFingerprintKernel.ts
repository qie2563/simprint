import { useCallback, useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { FingerprintProfile } from '../types/fingerprint';

/**
 * 指纹内核管理 Hook
 * 负责内核版本管理、指纹配置 CRUD 操作
 */
export function useFingerprintKernel() {
  const [kernels, setKernels] = useState<any[]>([]);
  const [profiles, setProfiles] = useState<FingerprintProfile[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // 加载所有内核
  const loadKernels = useCallback(async () => {
    setLoading(true);
    try {
      const result = await invoke<any[]>('list_fingerprint_kernels');
      setKernels(result);
      setError(null);
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      setError(`加载内核失败: ${errorMsg}`);
    } finally {
      setLoading(false);
    }
  }, []);

  // 加载所有指纹配置
  const loadProfiles = useCallback(async () => {
    setLoading(true);
    try {
      const result = await invoke<FingerprintProfile[]>(
        'list_fingerprint_profiles'
      );
      setProfiles(result);
      setError(null);
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      setError(`加载配置失败: ${errorMsg}`);
    } finally {
      setLoading(false);
    }
  }, []);

  // 创建指纹配置
  const createProfile = useCallback(
    async (
      name: string,
      seed: number,
      platform: string,
      brand: string
    ): Promise<string> => {
      try {
        const profileId = await invoke<string>(
          'create_fingerprint_profile',
          { name, seed, platform, brand }
        );
        await loadProfiles();
        return profileId;
      } catch (err) {
        const errorMsg = err instanceof Error ? err.message : String(err);
        throw new Error(`创建配置失败: ${errorMsg}`);
      }
    },
    [loadProfiles]
  );

  // 更新指纹配置
  const updateProfile = useCallback(
    async (profile: FingerprintProfile) => {
      try {
        await invoke('update_fingerprint_profile', { profile });
        await loadProfiles();
      } catch (err) {
        const errorMsg = err instanceof Error ? err.message : String(err);
        throw new Error(`更新配置失败: ${errorMsg}`);
      }
    },
    [loadProfiles]
  );

  // 删除指纹配置
  const deleteProfile = useCallback(
    async (profileId: string) => {
      try {
        await invoke('delete_fingerprint_profile', { profile_id: profileId });
        await loadProfiles();
      } catch (err) {
        const errorMsg = err instanceof Error ? err.message : String(err);
        throw new Error(`删除配置失败: ${errorMsg}`);
      }
    },
    [loadProfiles]
  );

  // 克隆指纹配置
  const cloneProfile = useCallback(
    async (profileId: string, newName: string): Promise<string> => {
      try {
        const newProfileId = await invoke<string>('clone_fingerprint_profile', {
          profile_id: profileId,
          new_name: newName,
        });
        await loadProfiles();
        return newProfileId;
      } catch (err) {
        const errorMsg = err instanceof Error ? err.message : String(err);
        throw new Error(`克隆配置失败: ${errorMsg}`);
      }
    },
    [loadProfiles]
  );

  // 从预设创建配置
  const createFromPreset = useCallback(
    async (presetId: string, name: string): Promise<string> => {
      try {
        const profileId = await invoke<string>(
          'create_profile_from_preset',
          { preset_id: presetId, name }
        );
        await loadProfiles();
        return profileId;
      } catch (err) {
        const errorMsg = err instanceof Error ? err.message : String(err);
        throw new Error(`从预设创建失败: ${errorMsg}`);
      }
    },
    [loadProfiles]
  );

  useEffect(() => {
    loadKernels();
    loadProfiles();
  }, [loadKernels, loadProfiles]);

  return {
    kernels,
    profiles,
    loading,
    error,
    loadKernels,
    loadProfiles,
    createProfile,
    updateProfile,
    deleteProfile,
    cloneProfile,
    createFromPreset,
  };
}
