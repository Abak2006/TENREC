import React from 'react';
import {
  Cpu,
  HardDrive,
  Activity,
  ShieldAlert,
  ArrowDownRight,
  ArrowUpRight,
  Clock,
  ExternalLink,
  Layers,
} from 'lucide-react';
import { Card } from '../components/common/Card';
import { MetricCard } from '../components/common/MetricCard';
import { TelemetryGraph } from '../components/common/TelemetryGraph';
import { EmptyState } from '../components/common/EmptyState';
import type {
  SystemSnapshot,
  NetworkThroughput,
  SecurityIncident,
  WhatChangedItem,
  ConnectionInfo,
  TopDestination,
  ApplicationItem,
} from '../types';

interface OverviewPageProps {
  snapshot: SystemSnapshot | null;
  network: NetworkThroughput | null;
  incidents: SecurityIncident[];
  whatChanged: WhatChangedItem[];
  connections: ConnectionInfo[];
  destinations: TopDestination[];
  applications: ApplicationItem[];
  netThroughputHistory: { inBytes: number[]; outBytes: number[] };
  cpuHistory: number[];
  memoryHistory: number[];
  onOpenActionReview: (incidentId?: string) => void;
  onNavigate: (page: any) => void;
}

export const OverviewPage: React.FC<OverviewPageProps> = ({
  snapshot,
  network,
  incidents,
  whatChanged,
  connections,
  destinations,
  applications,
  netThroughputHistory,
  cpuHistory,
  memoryHistory,
  onOpenActionReview,
  onNavigate,
}) => {
  const formatBytes = (bytes: number): string => {
    if (bytes >= 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
    if (bytes >= 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    if (bytes >= 1024) return `${(bytes / 1024).toFixed(0)} KB`;
    return `${bytes} B`;
  };

  const formatRate = (bytesSec: number): string => {
    const bits = bytesSec * 8;
    if (bits >= 1_000_000_000) return `${(bits / 1_000_000_000).toFixed(1)} Gbps`;
    if (bits >= 1_000_000) return `${(bits / 1_000_000).toFixed(1)} Mbps`;
    if (bits >= 1_000) return `${(bits / 1_000).toFixed(1)} Kbps`;
    return `${bits.toFixed(0)} bps`;
  };

  const activeIncidents = incidents.filter((i) => i.status === 'ACTIVE');
  const primaryDrive = snapshot?.disk.drives[0];

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 14 }}>
      {/* Primary Telemetry Metric Grid */}
      <div className="grid-6">
        <MetricCard
          label="CPU Load"
          value={snapshot ? `${snapshot.cpu.usagePercent.toFixed(1)}` : '--'}
          unit="%"
          secondary={snapshot ? `${snapshot.cpu.logicalCoreCount} threads` : undefined}
          progress={snapshot?.cpu.usagePercent}
          icon={<Cpu size={16} />}
          status={snapshot && snapshot.cpu.usagePercent > 85 ? 'attention' : 'normal'}
          onClick={() => onNavigate('system')}
        />

        <MetricCard
          label="Memory Used"
          value={snapshot ? `${snapshot.memory.usagePercent.toFixed(0)}` : '--'}
          unit="%"
          secondary={
            snapshot
              ? `${formatBytes(snapshot.memory.usedBytes)} / ${formatBytes(snapshot.memory.totalBytes)}`
              : undefined
          }
          progress={snapshot?.memory.usagePercent}
          icon={<HardDrive size={16} />}
          status={snapshot && snapshot.memory.usagePercent > 90 ? 'critical' : 'normal'}
          onClick={() => onNavigate('system')}
        />

        <MetricCard
          label="Download"
          value={network ? formatRate(network.bytesInSec).split(' ')[0] : '0.0'}
          unit={network ? formatRate(network.bytesInSec).split(' ')[1] : 'Kbps'}
          secondary={network ? `Total: ${formatBytes(network.totalBytesIn)}` : undefined}
          icon={<ArrowDownRight size={16} className="text-cyan" />}
          onClick={() => onNavigate('network')}
        />

        <MetricCard
          label="Upload"
          value={network ? formatRate(network.bytesOutSec).split(' ')[0] : '0.0'}
          unit={network ? formatRate(network.bytesOutSec).split(' ')[1] : 'Kbps'}
          secondary={network ? `Total: ${formatBytes(network.totalBytesOut)}` : undefined}
          icon={<ArrowUpRight size={16} className="text-blue" />}
          onClick={() => onNavigate('network')}
        />

        <MetricCard
          label="Primary Disk"
          value={primaryDrive ? `${primaryDrive.usagePercent.toFixed(0)}` : '--'}
          unit="%"
          secondary={primaryDrive ? `${formatBytes(primaryDrive.freeBytes)} free` : 'Checking...'}
          progress={primaryDrive?.usagePercent}
          icon={<Activity size={16} />}
          onClick={() => onNavigate('system')}
        />

        <MetricCard
          label="Security Incidents"
          value={activeIncidents.length}
          secondary={activeIncidents.length > 0 ? 'Requires attention' : 'Baseline normal'}
          icon={<ShieldAlert size={16} />}
          status={activeIncidents.length > 0 ? 'attention' : 'normal'}
          onClick={() => onNavigate('security')}
        />
      </div>

      {/* Dual Telemetry Charts */}
      <div className="grid-2">
        <Card
          title="Network Throughput (Real-Time)"
          subtitle="Observed interface traffic rates"
          icon={<Activity size={15} />}
          action={
            <span className="mono text-xs text-muted">
              Active sockets: {network ? network.activeConnectionCount : 0}
            </span>
          }
        >
          <TelemetryGraph
            height={130}
            unit=" KB/s"
            formatValue={(v) => (v / 1024).toFixed(0)}
            series={[
              { name: 'Download', color: '#06b6d4', data: netThroughputHistory.inBytes },
              { name: 'Upload', color: '#3b82f6', data: netThroughputHistory.outBytes },
            ]}
          />
        </Card>

        <Card
          title="System Resource Utilization"
          subtitle="Real-time CPU and Memory loads"
          icon={<Cpu size={15} />}
          action={
            <span className="mono text-xs text-muted">
              Uptime: {snapshot ? `${Math.floor(snapshot.uptimeSeconds / 3600)}h ${Math.floor((snapshot.uptimeSeconds % 3600) / 60)}m` : '--'}
            </span>
          }
        >
          <TelemetryGraph
            height={130}
            unit="%"
            formatValue={(v) => v.toFixed(0)}
            series={[
              { name: 'CPU Load', color: '#10b981', data: cpuHistory },
              { name: 'Memory Load', color: '#a855f7', data: memoryHistory },
            ]}
          />
        </Card>
      </div>

      {/* Security Incidents & What Changed Row */}
      <div className="grid-2">
        {/* Security & Attention Incidents */}
        <Card
          title="Security Observations & Incidents"
          subtitle="Explainable behavioral detections supported by concrete evidence"
          icon={<ShieldAlert size={15} />}
          action={
            activeIncidents.length > 0 ? (
              <button
                className="btn btn-primary"
                style={{ fontSize: '11px', padding: '3px 8px' }}
                onClick={() => onOpenActionReview()}
              >
                Review Recommendations ({activeIncidents.length})
              </button>
            ) : undefined
          }
        >
          {activeIncidents.length === 0 ? (
            <EmptyState
              title="No Security Incidents Detected"
              message="All active processes and network destinations are consistent with the observed system baseline."
            />
          ) : (
            <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
              {activeIncidents.slice(0, 3).map((incident) => (
                <div
                  key={incident.id}
                  style={{
                    backgroundColor: 'var(--bg-secondary)',
                    border: '1px solid var(--border-subtle)',
                    borderRadius: 'var(--radius-md)',
                    padding: '10px 12px',
                  }}
                >
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 4 }}>
                    <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
                      <span
                        className={`badge ${
                          incident.severity === 'CRITICAL' || incident.severity === 'HIGH'
                            ? 'badge-red'
                            : 'badge-amber'
                        }`}
                        style={{ fontSize: '9px', padding: '1px 5px' }}
                      >
                        {incident.severity}
                      </span>
                      <span className="text-sm font-semibold">{incident.title}</span>
                    </div>
                    <span className="mono text-xs text-muted">{incident.confidencePercent}% confidence</span>
                  </div>

                  <div className="text-xs text-secondary" style={{ marginBottom: 6 }}>
                    {incident.summary}
                  </div>

                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                    <span className="mono text-xs text-muted" style={{ fontSize: '10.5px' }}>
                      App: <span className="text-cyan">{incident.affectedApplication}</span>
                      {incident.affectedPid && ` (PID ${incident.affectedPid})`}
                    </span>
                    <button
                      className="btn btn-secondary"
                      style={{ fontSize: '10.5px', padding: '2px 8px' }}
                      onClick={() => onOpenActionReview(incident.id)}
                    >
                      Inspect Evidence &amp; Actions
                    </button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </Card>

        {/* What Changed Feed */}
        <Card
          title="What Changed?"
          subtitle="First-class audit of new entities, destinations, and volume deltas"
          icon={<Clock size={15} />}
          action={
            <span className="mono text-xs text-muted">
              {whatChanged.length} events
            </span>
          }
        >
          {whatChanged.length === 0 ? (
            <EmptyState
              title="Learning Machine Baseline..."
              message="Observing process lifecycles and network activity to identify future deviations and new entities."
            />
          ) : (
            <div style={{ display: 'flex', flexDirection: 'column', gap: 6, maxHeight: '220px', overflowY: 'auto' }}>
              {whatChanged.slice(0, 5).map((item) => (
                <div
                  key={item.id}
                  style={{
                    display: 'flex',
                    alignItems: 'flex-start',
                    gap: 10,
                    padding: '8px 10px',
                    backgroundColor: 'var(--bg-secondary)',
                    borderRadius: 'var(--radius-md)',
                    border: '1px solid var(--border-subtle)',
                  }}
                >
                  <div
                    style={{
                      width: 6,
                      height: 6,
                      borderRadius: '50%',
                      marginTop: 5,
                      backgroundColor:
                        item.severity === 'warning'
                          ? 'var(--status-red)'
                          : item.severity === 'attention'
                          ? 'var(--status-amber)'
                          : 'var(--accent-cyan)',
                      flexShrink: 0,
                    }}
                  />
                  <div style={{ flex: 1 }}>
                    <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'baseline' }}>
                      <span className="text-xs font-semibold text-primary">{item.title}</span>
                      <span className="mono text-xs text-muted" style={{ fontSize: '10px' }}>
                        {item.timestamp}
                      </span>
                    </div>
                    <div className="text-xs text-secondary" style={{ marginTop: 2 }}>
                      {item.description}
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </Card>
      </div>

      {/* Active Network Connections & Top Destinations */}
      <div className="grid-2">
        {/* Active Connections */}
        <Card
          title="Active Network Sockets"
          subtitle="Live correlated process ↔ socket bindings"
          icon={<ExternalLink size={15} />}
          action={
            <button
              className="btn btn-secondary"
              style={{ fontSize: '10.5px', padding: '2px 8px' }}
              onClick={() => onNavigate('network')}
            >
              View All ({connections.length})
            </button>
          }
        >
          {connections.length === 0 ? (
            <EmptyState
              title="No Active Connections"
              message="No active TCP or UDP endpoints currently communicating on external interfaces."
            />
          ) : (
            <div className="table-container">
              <table className="data-table">
                <thead>
                  <tr>
                    <th>Process</th>
                    <th>PID</th>
                    <th>Proto</th>
                    <th>Remote Endpoint</th>
                    <th>State</th>
                  </tr>
                </thead>
                <tbody>
                  {connections.slice(0, 6).map((c) => (
                    <tr key={c.id}>
                      <td className="font-semibold text-primary">{c.processName}</td>
                      <td className="mono text-muted">{c.pid}</td>
                      <td>
                        <span className="badge badge-blue" style={{ fontSize: '9px', padding: '1px 4px' }}>
                          {c.protocol}
                        </span>
                      </td>
                      <td className="mono text-secondary">
                        {c.remoteAddress}:{c.remotePort}
                      </td>
                      <td>
                        <span className="mono text-xs text-muted">{c.state}</span>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </Card>

        {/* Top Observed Destinations & Applications */}
        <Card
          title="Top Destinations"
          subtitle={`Tracking ${applications.length} applications across active destinations`}
          icon={<Layers size={15} />}
          action={
            <button
              className="btn btn-secondary"
              style={{ fontSize: '10.5px', padding: '2px 8px' }}
              onClick={() => onNavigate('network')}
            >
              View All
            </button>
          }
        >
          {destinations.length === 0 ? (
            <EmptyState
              title="No Destination History"
              message="Awaiting socket telemetry to map outgoing traffic to remote IP addresses and hostnames."
            />
          ) : (
            <div className="table-container">
              <table className="data-table">
                <thead>
                  <tr>
                    <th>Destination IP</th>
                    <th>Hostname</th>
                    <th>Primary App</th>
                    <th>Volume</th>
                  </tr>
                </thead>
                <tbody>
                  {destinations.slice(0, 6).map((d, idx) => (
                    <tr key={idx}>
                      <td className="mono text-primary font-semibold">{d.destination}</td>
                      <td className="text-secondary">{d.hostname || 'Hostname unavailable'}</td>
                      <td>
                        <span className="badge badge-muted" style={{ fontSize: '9px', padding: '1px 5px' }}>
                          {d.application}
                        </span>
                      </td>
                      <td className="mono text-cyan font-semibold">{formatBytes(d.totalBytes)}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </Card>
      </div>
    </div>
  );
};
