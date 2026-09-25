import React from 'react';
import { Cpu, HardDrive, Battery, BatteryCharging, Zap, Activity, Monitor, Server } from 'lucide-react';
import { Card } from '../components/common/Card';
import type { SystemSnapshot } from '../types';

interface SystemPageProps {
  snapshot: SystemSnapshot | null;
}

export const SystemPage: React.FC<SystemPageProps> = ({ snapshot }) => {
  const formatBytes = (bytes: number): string => {
    if (bytes >= 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
    if (bytes >= 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    if (bytes >= 1024) return `${(bytes / 1024).toFixed(0)} KB`;
    return `${bytes} B`;
  };

  const formatUptime = (seconds: number): string => {
    const days = Math.floor(seconds / 86400);
    const hours = Math.floor((seconds % 86400) / 3600);
    const mins = Math.floor((seconds % 3600) / 60);
    const secs = seconds % 60;
    return `${days > 0 ? `${days}d ` : ''}${hours}h ${mins}m ${secs}s`;
  };

  if (!snapshot) {
    return (
      <div style={{ padding: 20, textAlign: 'center', color: 'var(--text-muted)' }}>
        Querying native Windows system telemetry...
      </div>
    );
  }

  const { cpu, memory, disk, power, gpu, hostname, osName, uptimeSeconds } = snapshot;

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 14 }}>
      {/* Header System Overview Strip */}
      <Card
        title="Host Information & Environment"
        subtitle="Hardware and operating system identity"
        icon={<Server size={16} />}
        action={
          <div className="mono text-xs text-muted" style={{ display: 'flex', gap: 14 }}>
            <span>Host: <strong className="text-primary">{hostname}</strong></span>
            <span>OS: <strong className="text-primary">{osName}</strong></span>
            <span>Uptime: <strong className="text-primary">{formatUptime(uptimeSeconds)}</strong></span>
          </div>
        }
      >
        <div className="grid-4" style={{ marginTop: 6 }}>
          <div style={{ background: 'var(--bg-secondary)', padding: '10px 12px', borderRadius: 'var(--radius-md)' }}>
            <span className="text-xs text-muted">CPU Architecture</span>
            <div className="text-sm font-bold text-primary mono" style={{ marginTop: 2 }}>
              x86_64 ({cpu.coreCount} Cores / {cpu.logicalCoreCount} Threads)
            </div>
          </div>

          <div style={{ background: 'var(--bg-secondary)', padding: '10px 12px', borderRadius: 'var(--radius-md)' }}>
            <span className="text-xs text-muted">Total Physical Memory</span>
            <div className="text-sm font-bold text-primary mono" style={{ marginTop: 2 }}>
              {formatBytes(memory.totalBytes)} RAM
            </div>
          </div>

          <div style={{ background: 'var(--bg-secondary)', padding: '10px 12px', borderRadius: 'var(--radius-md)' }}>
            <span className="text-xs text-muted">Power Source</span>
            <div className="text-sm font-bold text-primary mono" style={{ marginTop: 2, display: 'flex', alignItems: 'center', gap: 6 }}>
              {power.acConnected ? <Zap size={13} className="text-green" /> : <Battery size={13} className="text-amber" />}
              {power.acConnected ? 'AC Power Line' : 'Battery Discharging'}
            </div>
          </div>

          <div style={{ background: 'var(--bg-secondary)', padding: '10px 12px', borderRadius: 'var(--radius-md)' }}>
            <span className="text-xs text-muted">Graphics Acceleration</span>
            <div className="text-sm font-bold text-primary mono" style={{ marginTop: 2 }}>
              {gpu.name || 'Integrated / Generic'}
            </div>
          </div>
        </div>
      </Card>

      {/* CPU & Memory In-Depth Cards */}
      <div className="grid-2">
        {/* CPU Section */}
        <Card
          title="Processor Utilization & Core Distribution"
          subtitle="Real-time multi-core breakdown"
          icon={<Cpu size={16} />}
          action={<span className="mono text-xs text-cyan font-bold">{cpu.usagePercent.toFixed(1)}% Overall</span>}
        >
          <div style={{ marginBottom: 12 }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 4 }}>
              <span className="text-xs text-muted">Combined Processor Load</span>
              <span className="mono text-xs font-bold text-primary">{cpu.usagePercent.toFixed(1)}%</span>
            </div>
            <div style={{ height: 6, background: 'var(--bg-secondary)', borderRadius: 3, overflow: 'hidden' }}>
              <div
                style={{
                  height: '100%',
                  width: `${cpu.usagePercent}%`,
                  background: cpu.usagePercent > 85 ? 'var(--status-red)' : 'var(--status-green)',
                }}
              />
            </div>
          </div>

          <div className="text-xs font-semibold text-secondary" style={{ marginBottom: 8 }}>
            Logical Core Allocations ({cpu.perCoreUsage.length} cores):
          </div>

          <div
            style={{
              display: 'grid',
              gridTemplateColumns: 'repeat(4, 1fr)',
              gap: 8,
              maxHeight: '180px',
              overflowY: 'auto',
            }}
          >
            {cpu.perCoreUsage.map((usage, idx) => (
              <div
                key={idx}
                style={{
                  background: 'var(--bg-secondary)',
                  padding: '6px 8px',
                  borderRadius: 'var(--radius-sm)',
                  border: '1px solid var(--border-subtle)',
                }}
              >
                <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '10.5px' }}>
                  <span className="text-muted mono">Core {idx}</span>
                  <span className="mono font-semibold text-primary">{usage.toFixed(0)}%</span>
                </div>
                <div style={{ height: 3, background: 'var(--border-subtle)', borderRadius: 1.5, marginTop: 4 }}>
                  <div
                    style={{
                      height: '100%',
                      width: `${usage}%`,
                      background: 'var(--accent-blue)',
                    }}
                  />
                </div>
              </div>
            ))}
          </div>
        </Card>

        {/* Memory Section */}
        <Card
          title="Physical RAM (Memory) State"
          subtitle="Windows GlobalMemoryStatusEx telemetry"
          icon={<HardDrive size={16} />}
          action={<span className="mono text-xs text-purple font-bold">{memory.usagePercent.toFixed(1)}% Load</span>}
        >
          <div style={{ marginBottom: 14 }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 4 }}>
              <span className="text-xs text-muted">Allocated Working Memory</span>
              <span className="mono text-xs font-bold text-primary">
                {formatBytes(memory.usedBytes)} / {formatBytes(memory.totalBytes)}
              </span>
            </div>
            <div style={{ height: 6, background: 'var(--bg-secondary)', borderRadius: 3, overflow: 'hidden' }}>
              <div
                style={{
                  height: '100%',
                  width: `${memory.usagePercent}%`,
                  background: memory.usagePercent > 90 ? 'var(--status-red)' : '#a855f7',
                }}
              />
            </div>
          </div>

          <div className="grid-2" style={{ marginTop: 12 }}>
            <div style={{ background: 'var(--bg-secondary)', padding: '10px 12px', borderRadius: 'var(--radius-md)' }}>
              <span className="text-xs text-muted">Available / Free RAM</span>
              <div className="text-lg font-bold text-green mono" style={{ marginTop: 2 }}>
                {formatBytes(memory.availableBytes)}
              </div>
              <span className="text-xs text-muted">Instant OS headroom</span>
            </div>

            <div style={{ background: 'var(--bg-secondary)', padding: '10px 12px', borderRadius: 'var(--radius-md)' }}>
              <span className="text-xs text-muted">Memory Load Factor</span>
              <div className="text-lg font-bold text-primary mono" style={{ marginTop: 2 }}>
                {memory.usagePercent.toFixed(0)}%
              </div>
              <span className="text-xs text-muted">Windows commit pressure</span>
            </div>
          </div>
        </Card>
      </div>

      {/* Storage Volumes & Power State */}
      <div className="grid-2">
        {/* Storage Drives */}
        <Card
          title="Storage Drives & Logical Volumes"
          subtitle="Real capacity query via GetDiskFreeSpaceExW"
          icon={<Activity size={16} />}
        >
          <div style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
            {disk.drives.map((d) => (
              <div
                key={d.mountPoint}
                style={{
                  background: 'var(--bg-secondary)',
                  padding: '10px 12px',
                  borderRadius: 'var(--radius-md)',
                  border: '1px solid var(--border-subtle)',
                }}
              >
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 4 }}>
                  <div>
                    <span className="mono font-bold text-primary">{d.mountPoint} </span>
                    <span className="text-xs text-muted">
                      ({d.volumeName || 'Local Disk'}, {d.fileSystem})
                    </span>
                  </div>
                  <span className="mono text-xs font-semibold text-primary">
                    {formatBytes(d.usedBytes)} used of {formatBytes(d.totalBytes)} ({d.usagePercent.toFixed(0)}%)
                  </span>
                </div>

                <div style={{ height: 6, background: 'var(--border-subtle)', borderRadius: 3, overflow: 'hidden' }}>
                  <div
                    style={{
                      height: '100%',
                      width: `${d.usagePercent}%`,
                      background: d.usagePercent > 90 ? 'var(--status-red)' : 'var(--accent-cyan)',
                    }}
                  />
                </div>

                <div className="text-xs text-muted mono" style={{ marginTop: 4, fontSize: '10.5px' }}>
                  {formatBytes(d.freeBytes)} unallocated space remaining
                </div>
              </div>
            ))}
          </div>
        </Card>

        {/* Power & Graphics State */}
        <Card
          title="Power, Battery & GPU Telemetry"
          subtitle="Windows power management and hardware capabilities"
          icon={<Zap size={16} />}
        >
          <div style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
            {/* Battery details */}
            <div
              style={{
                background: 'var(--bg-secondary)',
                padding: '10px 12px',
                borderRadius: 'var(--radius-md)',
                border: '1px solid var(--border-subtle)',
              }}
            >
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 4 }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
                  {power.acConnected ? <BatteryCharging size={16} className="text-green" /> : <Battery size={16} className="text-amber" />}
                  <span className="text-xs font-bold text-primary">Power Subsystem</span>
                </div>
                <span className="mono text-xs text-secondary">
                  {power.hasBattery && power.batteryPercent !== null
                    ? `${power.batteryPercent}% (${power.batteryStatus})`
                    : 'Desktop (No Battery)'}
                </span>
              </div>
              <div className="text-xs text-muted">
                Active Windows Power Profile: <strong className="text-secondary mono">{power.powerMode}</strong>
              </div>
            </div>

            {/* GPU details */}
            <div
              style={{
                background: 'var(--bg-secondary)',
                padding: '10px 12px',
                borderRadius: 'var(--radius-md)',
                border: '1px solid var(--border-subtle)',
              }}
            >
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 4 }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
                  <Monitor size={16} className="text-cyan" />
                  <span className="text-xs font-bold text-primary">GPU / Display Adapter</span>
                </div>
                <span className="mono text-xs text-muted">
                  {gpu.isAvailable ? 'Active' : 'Report'}
                </span>
              </div>
              <div className="text-xs text-secondary font-semibold">{gpu.name || 'Standard Display Adapter'}</div>
              <div className="text-xs text-muted" style={{ marginTop: 2 }}>
                {gpu.statusMessage}
              </div>
            </div>
          </div>
        </Card>
      </div>
    </div>
  );
};
