import { useState, useCallback } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Badge } from '@/components/ui/badge';
import { ScrollArea } from '@/components/ui/scroll-area';
import { FingerprintProfile } from '../../types/fingerprint';
import { useTranslation } from 'react-i18next';
import { Settings2, Copy, Trash2, Play, Square } from 'lucide-react';

interface FingerprintConfigEditorProps {
  profile: FingerprintProfile;
  onSave: (profile: FingerprintProfile) => Promise<void>;
  onDelete: (profileId: string) => Promise<void>;
  onClone: (profileId: string, newName: string) => Promise<void>;
  onStart?: (profileId: string) => Promise<void>;
  onStop?: () => Promise<void>;
  isRunning?: boolean;
}

/**
 * 指纹配置编辑器组件
 * 提供完整的指纹参数配置界面
 */
export function FingerprintConfigEditor({
  profile,
  onSave,
  onDelete,
  onClone,
  onStart,
  onStop,
  isRunning = false,
}: FingerprintConfigEditorProps) {
  const { t } = useTranslation('fingerprint');
  const [editing, setEditing] = useState(false);
  const [cloneName, setCloneName] = useState('');
  const [formData, setFormData] = useState<FingerprintProfile>(profile);

  const handleSave = useCallback(async () => {
    await onSave(formData);
    setEditing(false);
  }, [formData, onSave]);

  const handleClone = useCallback(async () => {
    if (cloneName.trim()) {
      await onClone(profile.id, cloneName);
      setCloneName('');
    }
  }, [cloneName, profile.id, onClone]);

  const handleDelete = useCallback(async () => {
    if (confirm(t('common:confirm_delete'))) {
      await onDelete(profile.id);
    }
  }, [profile.id, onDelete, t]);

  const updateField = (field: keyof FingerprintProfile, value: any) => {
    setFormData((prev) => ({
      ...prev,
      [field]: value,
    }));
  };

  const updateArrayField = (field: 'disable_spoofing', value: string) => {
    const arr = formData[field] as string[];
    if (arr.includes(value)) {
      updateField(field, arr.filter((v) => v !== value));
    } else {
      updateField(field, [...arr, value]);
    }
  };

  return (
    <Card className="w-full">
      <CardHeader>
        <div className="flex items-center justify-between">
          <div>
            <CardTitle>{formData.name}</CardTitle>
            <CardDescription>
              {t('fingerprint:seed')}: {formData.seed}
            </CardDescription>
          </div>
          <div className="flex gap-2">
            {isRunning ? (
              <Button
                size="sm"
                variant="destructive"
                onClick={onStop}
                className="gap-2"
              >
                <Square className="h-4 w-4" />
                {t('common:stop')}
              </Button>
            ) : (
              <Button
                size="sm"
                variant="default"
                onClick={() => onStart?.(profile.id)}
                className="gap-2"
              >
                <Play className="h-4 w-4" />
                {t('common:start')}
              </Button>
            )}
            <Button
              size="sm"
              variant="outline"
              onClick={() => setEditing(!editing)}
            >
              <Settings2 className="h-4 w-4" />
            </Button>
          </div>
        </div>
      </CardHeader>

      <CardContent>
        <Tabs defaultValue="basic">
          <TabsList className="grid w-full grid-cols-3">
            <TabsTrigger value="basic">{t('fingerprint:basic')}</TabsTrigger>
            <TabsTrigger value="advanced">{t('fingerprint:advanced')}</TabsTrigger>
            <TabsTrigger value="spoofing">{t('fingerprint:spoofing')}</TabsTrigger>
          </TabsList>

          {/* 基本配置 */}
          <TabsContent value="basic" className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div>
                <Label>{t('fingerprint:platform')}</Label>
                <Select
                  value={formData.platform}
                  onValueChange={(value) =>
                    updateField('platform', value)
                  }
                  disabled={!editing}
                >
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="windows">Windows</SelectItem>
                    <SelectItem value="macos">macOS</SelectItem>
                    <SelectItem value="linux">Linux</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div>
                <Label>{t('fingerprint:brand')}</Label>
                <Select
                  value={formData.brand}
                  onValueChange={(value) => updateField('brand', value)}
                  disabled={!editing}
                >
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="Chrome">Chrome</SelectItem>
                    <SelectItem value="Edge">Edge</SelectItem>
                    <SelectItem value="Opera">Opera</SelectItem>
                    <SelectItem value="Vivaldi">Vivaldi</SelectItem>
                    <SelectItem value="Chromium">Chromium</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>
          </TabsContent>

          {/* 高级配置 */}
          <TabsContent value="advanced" className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div>
                <Label>{t('fingerprint:timezone')}</Label>
                <Input
                  value={formData.timezone || ''}
                  onChange={(e) =>
                    updateField('timezone', e.target.value || null)
                  }
                  placeholder="e.g., America/New_York"
                  disabled={!editing}
                />
              </div>
              <div>
                <Label>{t('fingerprint:language')}</Label>
                <Input
                  value={formData.language || ''}
                  onChange={(e) =>
                    updateField('language', e.target.value || null)
                  }
                  placeholder="e.g., en-US"
                  disabled={!editing}
                />
              </div>
              <div>
                <Label>{t('fingerprint:hardware_concurrency')}</Label>
                <Input
                  type="number"
                  value={formData.hardware_concurrency || ''}
                  onChange={(e) =>
                    updateField(
                      'hardware_concurrency',
                      e.target.value ? parseInt(e.target.value) : null
                    )
                  }
                  disabled={!editing}
                />
              </div>
              <div>
                <Label>{t('fingerprint:proxy_server')}</Label>
                <Input
                  value={formData.proxy_server || ''}
                  onChange={(e) =>
                    updateField('proxy_server', e.target.value || null)
                  }
                  placeholder="e.g., socks5://127.0.0.1:1080"
                  disabled={!editing}
                />
              </div>
            </div>
          </TabsContent>

          {/* 伪装选项 */}
          <TabsContent value="spoofing" className="space-y-4">
            <div className="space-y-2">
              <Label>{t('fingerprint:disable_spoofing')}</Label>
              <div className="flex flex-wrap gap-2">
                {['font', 'audio', 'canvas', 'clientrects', 'gpu'].map(
                  (option) => (
                    <Badge
                      key={option}
                      variant={
                        formData.disable_spoofing.includes(option)
                          ? 'default'
                          : 'outline'
                      }
                      className="cursor-pointer"
                      onClick={() => {
                        if (editing) {
                          updateArrayField('disable_spoofing', option);
                        }
                      }}
                    >
                      {option}
                    </Badge>
                  )
                )}
              </div>
            </div>
          </TabsContent>
        </Tabs>

        {/* 操作按钮 */}
        <div className="mt-6 flex gap-2 border-t pt-4">
          {editing ? (
            <>
              <Button onClick={handleSave} variant="default">
                {t('common:save')}
              </Button>
              <Button
                onClick={() => setEditing(false)}
                variant="outline"
              >
                {t('common:cancel')}
              </Button>
            </>
          ) : (
            <>
              <Button
                onClick={() => setEditing(true)}
                variant="outline"
              >
                {t('common:edit')}
              </Button>
              <Button
                onClick={handleClone}
                variant="outline"
                className="gap-2"
              >
                <Copy className="h-4 w-4" />
                {t('common:clone')}
              </Button>
              <Button
                onClick={handleDelete}
                variant="destructive"
                className="ml-auto gap-2"
              >
                <Trash2 className="h-4 w-4" />
                {t('common:delete')}
              </Button>
            </>
          )}
        </div>
      </CardContent>
    </Card>
  );
}
