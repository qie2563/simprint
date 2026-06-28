import { useEffect, useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { ScrollArea } from '@/components/ui/scroll-area';
import { BrowserInstance } from '../../types/fingerprint';
import { useTranslation } from 'react-i18next';
import { Activity, Cpu, HardDrive, Zap } from 'lucide-react';
import { Progress } from '@/components/ui/progress';

interface BrowserInstanceMonitorProps {
  instances: BrowserInstance[];
  loading?: boolean;
  onRefresh?: () => void;
}

interface InstanceMetrics {
  instanceId: string;
  cpu: number;
  memory: number;
  network: {
    upload: number;
    download: number;
  };
}

/**
 * 浏览器实例监控组件
 * 实时显示运行中的浏览器实例及其资源使用情况
 */
export function BrowserInstanceMonitor({
  instances,
  loading = false,
  onRefresh,
}: BrowserInstanceMonitorProps) {
  const { t } = useTranslation('fingerprint');
  const [metrics, setMetrics] = useState<Map<string, InstanceMetrics>>(
    new Map()
  );

  // 模拟资源使用情况（生产环境应从后端获取）
  useEffect(() => {
    const updateMetrics = () => {
      const newMetrics = new Map<string, InstanceMetrics>();
      instances.forEach((instance) => {
        if (instance.status === 'Running') {
          newMetrics.set(instance.instance_id, {
            instanceId: instance.instance_id,
            cpu: Math.random() * 30 + 10,
            memory: Math.random() * 500 + 200,
            network: {
              upload: Math.random() * 5,
              download: Math.random() * 10,
            },
          });
        }
      });
      setMetrics(newMetrics);
    };

    updateMetrics();
    const interval = setInterval(updateMetrics, 2000);
    return () => clearInterval(interval);
  }, [instances]);

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'Running':
        return 'bg-emerald-500';
      case 'Starting':
        return 'bg-amber-500';
      case 'Stopping':
        return 'bg-orange-500';
      case 'Stopped':
        return 'bg-gray-500';
      case 'Error':
        return 'bg-red-500';
      default:
        return 'bg-blue-500';
    }
  };

  const getStatusLabel = (status: string): string => {
    return t(`fingerprint:status.${status.toLowerCase()}`, status);
  };

  const runningCount = instances.filter(
    (i) => i.status === 'Running'
  ).length;

  return (
    <div className="space-y-4">
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle className="flex items-center gap-2">
                <Activity className="h-5 w-5" />
                {t('fingerprint:instance_monitor')}
              </CardTitle>
              <CardDescription>
                {t('fingerprint:running_instances')}: {runningCount}
              </CardDescription>
            </div>
          </div>
        </CardHeader>
      </Card>

      <ScrollArea className="h-[600px] w-full rounded-md border">
        <div className="p-4 space-y-3">
          {instances.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-8 text-muted-foreground">
              <Zap className="h-8 w-8 mb-2 opacity-50" />
              <p>{t('fingerprint:no_running_instances')}</p>
            </div>
          ) : (
            instances.map((instance) => {
              const metric = metrics.get(instance.instance_id);
              return (
                <Card key={instance.instance_id}>
                  <CardContent className="pt-4">
                    <div className="space-y-3">
                      {/* 头部信息 */}
                      <div className="flex items-start justify-between">
                        <div>
                          <p className="font-mono text-sm text-muted-foreground">
                            {instance.instance_id}
                          </p>
                          <p className="text-xs text-muted-foreground mt-1">
                            {t('fingerprint:env')}: {instance.env_uuid}
                          </p>
                        </div>
                        <Badge className={getStatusColor(instance.status)}>
                          {getStatusLabel(instance.status)}
                        </Badge>
                      </div>

                      {/* 资源使用情况 */}
                      {metric && instance.status === 'Running' && (
                        <div className="space-y-2">
                          <div className="flex items-center justify-between text-sm">
                            <span className="flex items-center gap-1 text-muted-foreground">
                              <Cpu className="h-3 w-3" />
                              CPU
                            </span>
                            <span>{metric.cpu.toFixed(1)}%</span>
                          </div>
                          <Progress value={metric.cpu} className="h-1" />

                          <div className="flex items-center justify-between text-sm">
                            <span className="flex items-center gap-1 text-muted-foreground">
                              <HardDrive className="h-3 w-3" />
                              {t('fingerprint:memory')}
                            </span>
                            <span>{(metric.memory / 1024).toFixed(1)} MB</span>
                          </div>
                          <Progress
                            value={(metric.memory / 2048) * 100}
                            className="h-1"
                          />

                          <div className="text-xs text-muted-foreground mt-2">
                            <p>
                              ⬆️{' '}
                              {metric.network.upload.toFixed(2)}
                              Mbps / ⬇️{' '}
                              {metric.network.download.toFixed(2)}
                              Mbps
                            </p>
                          </div>
                        </div>
                      )}

                      {/* 时间信息 */}
                      <div className="text-xs text-muted-foreground pt-2 border-t">
                        <p>
                          {t('fingerprint:started')}:{' '}
                          {instance.started_at
                            ? new Date(
                                instance.started_at
                              ).toLocaleTimeString()
                            : '-'}
                        </p>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              );
            })
          )}
        </div>
      </ScrollArea>
    </div>
  );
}
