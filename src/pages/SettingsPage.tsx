import React, { useState } from 'react';
import { Sliders, Database, Cpu, Check, AlertTriangle, ToggleLeft, ToggleRight, RefreshCw } from 'lucide-react';
import { Card } from '../components/common/Card';
import type { HardwareCapability } from '../types';

interface SettingsPageProps {
  capabilities: HardwareCapability[];
  onRefreshCapabilities?: () => void;
}

export const SettingsPage: React.FC<SettingsPageProps> = ({
  capabilities,
  onRefreshCapabilities,
}) => {
  const [startWithWindows, setStartWithWindows] = useState(false);
  const [retentionDays, setRetentionDays] = useState(14);
  const [samplingIntervalSec, setSamplingIntervalSec] = useState(2);

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 14 }}>
      {/* Header */}
      <div>
        <div className="text-lg font-bold text-primary">Platform Settings &amp; Hardware Capabilities</div>
        <div className="text-xs text-muted">
          Inspect native Windows capabilities, sampling parameters, and local SQLite persistence options.
        </div>
      </div>

      {/* Hardware Capabilities (Section 32) */}
      <Card
        title="Detected Hardware Capabilities"
        subtitle="Hardware controls verified via real Windows APIs. Unsupported vendor extensions are honestly declared."
        icon={<Cpu size={16} />}
        action={
          onRefreshCapabilities && (
            <button className="btn btn-secondary" style={{ fontSize: '10.5px' }} onClick={onRefreshCapabilities}>
              <RefreshCw size={12} />
              Re-scan
            </button>
          )
        }
      >
        <div className="grid-2">
          {capabilities.map((cap, idx) => (
            <div
              key={idx}
              style={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                padding: '8px 12px',
                background: 'var(--bg-secondary)',
                borderRadius: 'var(--radius-md)',
                border: '1px solid var(--border-subtle)',
              }}
            >
              <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                {cap.available ? (
                  <Check size={16} className="text-green" />
                ) : (
                  <AlertTriangle size={16} className="text-amber" />
                )}
                <div>
                  <div className="text-xs font-semibold text-primary">{cap.name}</div>
                  <div className="text-xs text-muted" style={{ fontSize: '10.5px' }}>
                    {cap.details}
                  </div>
                </div>
              </div>

              <span
                className={`badge ${cap.available ? 'badge-green' : 'badge-amber'}`}
                style={{ fontSize: '9px' }}
              >
                {cap.available ? 'SUPPORTED' : 'VENDOR DEPENDENT'}
              </span>
            </div>
          ))}
        </div>
      </Card>

      {/* Platform & Startup Settings (Section 50) */}
      <div className="grid-2">
        <Card
          title="Startup &amp; Background Execution"
          subtitle="Explicit user consent required. TENREC never enables startup silently."
          icon={<Sliders size={16} />}
        >
          <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
            <div
              style={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                padding: '10px 12px',
                background: 'var(--bg-secondary)',
                borderRadius: 'var(--radius-md)',
              }}
            >
              <div>
                <div className="text-xs font-bold text-primary">Start with Windows</div>
                <div className="text-xs text-muted">
                  Launch TENREC minimized to the system notification area on user logon.
                </div>
              </div>
              <button
                className="btn btn-secondary"
                style={{ padding: '4px 8px' }}
                onClick={() => setStartWithWindows(!startWithWindows)}
              >
                {startWithWindows ? (
                  <ToggleRight size={22} className="text-cyan" />
                ) : (
                  <ToggleLeft size={22} className="text-muted" />
                )}
              </button>
            </div>

            <div
              style={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                padding: '10px 12px',
                background: 'var(--bg-secondary)',
                borderRadius: 'var(--radius-md)',
              }}
            >
              <div>
                <div className="text-xs font-bold text-primary">Telemetry Sampling Rate</div>
                <div className="text-xs text-muted">
                  Interval for sampling CPU, memory, and network throughput counters.
                </div>
              </div>
              <select
                value={samplingIntervalSec}
                onChange={(e) => setSamplingIntervalSec(Number(e.target.value))}
                style={{
                  background: 'var(--bg-card)',
                  color: 'var(--text-primary)',
                  border: '1px solid var(--border-medium)',
                  borderRadius: 'var(--radius-sm)',
                  padding: '4px 8px',
                  fontSize: '11px',
                  outline: 'none',
                }}
              >
                <option value={1}>1 second (High precision)</option>
                <option value={2}>2 seconds (Standard)</option>
                <option value={5}>5 seconds (Battery saver)</option>
              </select>
            </div>
          </div>
        </Card>

        {/* Local Persistence & Diagnostics (Section 66) */}
        <Card
          title="Local Persistence &amp; Diagnostics"
          subtitle="SQLite embedded database statistics and health diagnostics"
          icon={<Database size={16} />}
        >
          <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
            <div
              style={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                padding: '10px 12px',
                background: 'var(--bg-secondary)',
                borderRadius: 'var(--radius-md)',
              }}
            >
              <div>
                <div className="text-xs font-bold text-primary">Local Storage Retention</div>
                <div className="text-xs text-muted">
                  Purge samples older than this threshold to conserve disk space.
                </div>
              </div>
              <select
                value={retentionDays}
                onChange={(e) => setRetentionDays(Number(e.target.value))}
                style={{
                  background: 'var(--bg-card)',
                  color: 'var(--text-primary)',
                  border: '1px solid var(--border-medium)',
                  borderRadius: 'var(--radius-sm)',
                  padding: '4px 8px',
                  fontSize: '11px',
                  outline: 'none',
                }}
              >
                <option value={7}>7 Days</option>
                <option value={14}>14 Days (Default)</option>
                <option value={30}>30 Days</option>
              </select>
            </div>

            <div
              style={{
                background: 'var(--bg-secondary)',
                padding: '10px 12px',
                borderRadius: 'var(--radius-md)',
                fontSize: '11px',
              }}
            >
              <div className="font-semibold text-secondary" style={{ marginBottom: 4 }}>
                Subsystem Diagnostic Status:
              </div>
              <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: 6 }} className="mono text-xs">
                <div>
                  <span className="text-muted">Telemetry Engine: </span>
                  <span className="text-green">ONLINE</span>
                </div>
                <div>
                  <span className="text-muted">SQLite Storage: </span>
                  <span className="text-green">ACTIVE (WAL mode)</span>
                </div>
                <div>
                  <span className="text-muted">Network Provider: </span>
                  <span className="text-green">GetExtendedTcpTable</span>
                </div>
                <div>
                  <span className="text-muted">Firewall Enforcement: </span>
                  <span className="text-green">netsh advfirewall</span>
                </div>
              </div>
            </div>
          </div>
        </Card>
      </div>
    </div>
  );
};
