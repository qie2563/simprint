import { useCallback, useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Badge } from '@/components/ui/badge';
import { FingerprintPreset } from '../../types/fingerprint';
import { useTranslation } from 'react-i18next';
import { CheckCircle2 } from 'lucide-react';

interface PresetSelectorProps {
  presets: FingerprintPreset[];
  onSelectPreset: (preset: FingerprintPreset, profileName: string) => Promise<void>;
  loading?: boolean;
}

/**
 * 指纹预设选择器组件
 * 提供内置预设选择和快速创建
 */
export function PresetSelector({
  presets,
  onSelectPreset,
  loading = false,
}: PresetSelectorProps) {
  const { t } = useTranslation('fingerprint');
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [newProfileName, setNewProfileName] = useState('');

  const handleCreate = useCallback(
    async (presetId: string) => {
      if (!newProfileName.trim()) {
        alert(t('fingerprint:profile_name_required'));
        return;
      }
      const preset = presets.find((p) => p.id === presetId);
      if (preset) {
        await onSelectPreset(preset, newProfileName);
        setNewProfileName('');
        setSelectedId(null);
      }
    },
    [newProfileName, presets, onSelectPreset, t]
  );

  return (
    <div className="space-y-4">
      <ScrollArea className="h-[400px] w-full rounded-md border">
        <div className="p-4 space-y-3">
          {presets.map((preset) => (
            <Card
              key={preset.id}
              className={
                selectedId === preset.id
                  ? 'border-primary bg-primary/5'
                  : 'cursor-pointer hover:bg-muted/50'
              }
              onClick={() => setSelectedId(preset.id)}
            >
              <CardHeader className="pb-3">
                <div className="flex items-center justify-between">
                  <div>
                    <CardTitle className="text-sm">
                      {preset.name}
                    </CardTitle>
                    <CardDescription className="text-xs">
                      {preset.description}
                    </CardDescription>
                  </div>
                  {selectedId === preset.id && (
                    <CheckCircle2 className="h-5 w-5 text-primary" />
                  )}
                </div>
              </CardHeader>
              <CardContent className="pb-3">
                <div className="flex flex-wrap gap-1">
                  <Badge variant="secondary" className="text-xs">
                    {preset.platform}
                  </Badge>
                  <Badge variant="secondary" className="text-xs">
                    {preset.brand}
                  </Badge>
                  {preset.timezone && (
                    <Badge variant="secondary" className="text-xs">
                      {preset.timezone}
                    </Badge>
                  )}
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      </ScrollArea>

      {selectedId && (
        <div className="space-y-3 border-t pt-4">
          <div>
            <Label>{t('fingerprint:new_profile_name')}</Label>
            <Input
              value={newProfileName}
              onChange={(e) => setNewProfileName(e.target.value)}
              placeholder={`${presets.find((p) => p.id === selectedId)?.name} Copy`}
            />
          </div>
          <Button
            onClick={() => handleCreate(selectedId)}
            disabled={loading || !newProfileName.trim()}
            className="w-full"
          >
            {loading
              ? t('common:loading')
              : t('fingerprint:create_from_preset')}
          </Button>
        </div>
      )}
    </div>
  );
}
