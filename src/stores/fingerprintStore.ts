import { create } from 'zustand';
import { devtools } from 'zustand/middleware';
import { FingerprintProfile } from '../types/fingerprint';

interface FingerprintStore {
  // 状态
  selectedProfileId: string | null;
  previewConfig: FingerprintProfile | null;
  isConfiguring: boolean;
  resourceUsage: ResourceUsage | null;

  // 操作
  setSelectedProfileId: (id: string | null) => void;
  setPreviewConfig: (config: FingerprintProfile | null) => void;
  setIsConfiguring: (value: boolean) => void;
  setResourceUsage: (usage: ResourceUsage | null) => void;
}

export interface ResourceUsage {
  instanceId: string;
  cpu: number; // 0-100
  memory: number; // 字节
  network: {
    uploadMbps: number;
    downloadMbps: number;
  };
  timestamp: string;
}

export const useFingerprintStore = create<FingerprintStore>(
  devtools((set) => ({
    selectedProfileId: null,
    previewConfig: null,
    isConfiguring: false,
    resourceUsage: null,

    setSelectedProfileId: (id) => set({ selectedProfileId: id }),
    setPreviewConfig: (config) => set({ previewConfig: config }),
    setIsConfiguring: (value) => set({ isConfiguring: value }),
    setResourceUsage: (usage) => set({ resourceUsage: usage }),
  })),
  { name: 'fingerprint-store' }
);
