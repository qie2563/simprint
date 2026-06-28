/**
 * 指纹浏览器类型定义
 */

export interface FingerprintProfile {
  id: string;
  name: string;
  seed: number;
  platform: string; // "windows", "linux", "macos"
  platform_version?: string;
  brand: string; // "Chrome", "Edge", "Opera", "Vivaldi", "Chromium"
  brand_version?: string;
  hardware_concurrency?: number;
  language?: string;
  accept_languages?: string;
  timezone?: string;
  proxy_server?: string;
  disable_spoofing: string[];
  created_at: string;
  updated_at: string;
  description?: string;
}

export interface InstalledKernel {
  version: KernelVersion;
  install_path: string;
  binary_path: string;
  installed_at: string;
  is_default: boolean;
}

export interface KernelVersion {
  major: number;
  minor: number;
  patch: number;
}

export interface BrowserInstance {
  instance_id: string;
  env_uuid: string;
  profile_id: string;
  pid?: number;
  status: string; // "Created" | "Starting" | "Running" | "Stopping" | "Stopped" | "Error"
  created_at: string;
  started_at?: string;
  stopped_at?: string;
}

export interface FingerprintPreset {
  id: string;
  name: string;
  description: string;
  seed: number;
  platform: string;
  platform_version?: string;
  brand: string;
  brand_version?: string;
  hardware_concurrency?: number;
  language?: string;
  timezone?: string;
  disable_spoofing: string[];
}

export interface EnvironmentFingerprintBinding {
  env_uuid: string;
  fingerprint_profile_id: string;
  kernel_version: string;
  created_at: string;
  updated_at: string;
}
