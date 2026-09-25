import React from 'react';
import { RefreshCw, Database, Cpu, HardDrive } from 'lucide-react';
import { StatusBadge } from '../common/StatusBadge';
import type { SystemHealthStatus, SystemSnapshot, NetworkThroughput } from '../../types';

interface TopBarProps {
  status: SystemHealthStatus;
  snapshot: SystemSnapshot | null;
  network: NetworkThroughput | null;
  onRefresh: () => void;
  isRefreshing: boolean;
}

export const TopBar: React.FC<TopBarProps> = ({
  status,
  snapshot,
  network,
  onRefresh,
  isRefreshing,
}) => {
  const formatSpeed = (bytesSec: number) => {
    const bits = bytesSec * 8;
    if (bits >= 1_000_000_000) return `${(bits / 1_000_000_000).toFixed(1)} Gbps`;
    if (bits >= 1_000_000) return `${(bits / 1_000_000).toFixed(1)} Mbps`;
    if (bits >= 1_000) return `${(bits / 1_000).toFixed(1)} Kbps`;
    return `${bits.toFixed(0)} bps`;
  };

  return (
    <header className="topbar">
      {/* Left: System Status & Philosophy */}
      <div style={{ display: 'flex', alignItems: 'center', gap: 14 }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
          <StatusBadge
            status={status === 'HEALTHY' ? 'healthy' : status === 'ATTENTION' ? 'attention' : 'critical'}
            label={status === 'HEALTHY' ? 'SYSTEM HEALTHY' : status === 'ATTENTION' ? 'ATTENTION' : 'ACTIVE INCIDENT'}
          />
        </div>

        <div style={{ height: 16, width: 1, backgroundColor: 'var(--border-subtle)' }} />

        <div className="text-xs text-muted" style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
          <Database size={13} className="text-cyan" />
          <span>Local Engine &bull; SQLite Active</span>
        </div>
      </div>

      {/* Right: Real-time quick telemetry telemetry badge + refresh */}
      <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
        {snapshot && (
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: 12,
              backgroundColor: 'var(--bg-card)',
              padding: '4px 10px',
              borderRadius: 'var(--radius-md)',
              border: '1px solid var(--border-subtle)',
              fontSize: '11px',
            }}
          >
            <div style={{ display: 'flex', alignItems: 'center', gap: 4 }}>
              <Cpu size={12} className="text-muted" />
              <span className="mono font-semibold text-secondary">
                CPU: {snapshot.cpu.usagePercent.toFixed(0)}%
              </span>
            </div>

            <div style={{ width: 1, height: 12, backgroundColor: 'var(--border-subtle)' }} />

            <div style={{ display: 'flex', alignItems: 'center', gap: 4 }}>
              <HardDrive size={12} className="text-muted" />
              <span className="mono font-semibold text-secondary">
                RAM: {snapshot.memory.usagePercent.toFixed(0)}%
              </span>
            </div>

            {network && (
              <>
                <div style={{ width: 1, height: 12, backgroundColor: 'var(--border-subtle)' }} />
                <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
                  <span className="mono text-cyan" style={{ fontSize: '10.5px' }}>
                    &darr; {formatSpeed(network.bytesInSec)}
                  </span>
                  <span className="mono text-blue" style={{ fontSize: '10.5px' }}>
                    &uarr; {formatSpeed(network.bytesOutSec)}
                  </span>
                </div>
              </>
            )}
          </div>
        )}

        <button
          className="btn btn-secondary"
          style={{ padding: '6px 9px' }}
          onClick={onRefresh}
          title="Refresh Telemetry"
        >
          <RefreshCw size={13} className={isRefreshing ? 'spin-icon' : ''} />
        </button>
      </div>
    </header>
  );
};
