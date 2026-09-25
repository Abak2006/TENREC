import React, { useState } from 'react';
import { Network, Activity, ArrowDownRight, ArrowUpRight, Search } from 'lucide-react';
import { Card } from '../components/common/Card';
import { MetricCard } from '../components/common/MetricCard';
import { TelemetryGraph } from '../components/common/TelemetryGraph';
import { EmptyState } from '../components/common/EmptyState';
import type { NetworkThroughput, ConnectionInfo, ListeningPortInfo, TopDestination } from '../types';

interface NetworkPageProps {
  network: NetworkThroughput | null;
  connections: ConnectionInfo[];
  listeningPorts: ListeningPortInfo[];
  destinations: TopDestination[];
  throughputHistory: { inBytes: number[]; outBytes: number[] };
  onInspectConnection?: (conn: ConnectionInfo) => void;
}

export const NetworkPage: React.FC<NetworkPageProps> = ({
  network,
  connections,
  listeningPorts,
  destinations,
  throughputHistory,
  onInspectConnection,
}) => {
  const [activeTab, setActiveTab] = useState<'connections' | 'listening' | 'destinations'>('connections');
  const [searchQuery, setSearchQuery] = useState('');

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

  // Filter connections
  const filteredConnections = connections.filter((c) => {
    const q = searchQuery.toLowerCase();
    return (
      c.processName.toLowerCase().includes(q) ||
      c.remoteAddress.toLowerCase().includes(q) ||
      c.hostname.toLowerCase().includes(q) ||
      c.pid.toString().includes(q)
    );
  });

  // Filter listening ports
  const filteredPorts = listeningPorts.filter((p) => {
    const q = searchQuery.toLowerCase();
    return (
      p.processName.toLowerCase().includes(q) ||
      p.port.toString().includes(q) ||
      p.bindAddress.includes(q)
    );
  });

  // Filter destinations
  const filteredDestinations = destinations.filter((d) => {
    const q = searchQuery.toLowerCase();
    return (
      d.destination.toLowerCase().includes(q) ||
      d.hostname.toLowerCase().includes(q) ||
      d.application.toLowerCase().includes(q)
    );
  });

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 14 }}>
      {/* Network Metrics Row */}
      <div className="grid-4">
        <MetricCard
          label="Download Throughput"
          value={network ? formatRate(network.bytesInSec).split(' ')[0] : '0.0'}
          unit={network ? formatRate(network.bytesInSec).split(' ')[1] : 'Kbps'}
          secondary={network ? `Total: ${formatBytes(network.totalBytesIn)}` : undefined}
          icon={<ArrowDownRight size={16} className="text-cyan" />}
        />

        <MetricCard
          label="Upload Throughput"
          value={network ? formatRate(network.bytesOutSec).split(' ')[0] : '0.0'}
          unit={network ? formatRate(network.bytesOutSec).split(' ')[1] : 'Kbps'}
          secondary={network ? `Total: ${formatBytes(network.totalBytesOut)}` : undefined}
          icon={<ArrowUpRight size={16} className="text-blue" />}
        />

        <MetricCard
          label="Active Connections"
          value={network ? network.activeConnectionCount : connections.length}
          secondary={network ? `Interface: ${network.primaryInterface}` : 'Querying...'}
          icon={<Network size={16} />}
        />

        <MetricCard
          label="Listening Sockets"
          value={listeningPorts.length}
          secondary="Local open ports"
          icon={<Activity size={16} />}
        />
      </div>

      {/* Real-time Network Throughput Chart */}
      <Card
        title="Interface Throughput Timeline (Live)"
        subtitle="Bandwidth transfer rates sampled across active network adapters"
        icon={<Activity size={15} />}
      >
        <TelemetryGraph
          height={140}
          unit=" KB/s"
          formatValue={(v) => (v / 1024).toFixed(0)}
          series={[
            { name: 'Inbound Rate (KB/s)', color: '#06b6d4', data: throughputHistory.inBytes },
            { name: 'Outbound Rate (KB/s)', color: '#3b82f6', data: throughputHistory.outBytes },
          ]}
        />
      </Card>

      {/* Navigation Tabs & Search Controls */}
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <div style={{ display: 'flex', gap: 8 }}>
          <button
            className="btn"
            style={{
              backgroundColor: activeTab === 'connections' ? 'var(--bg-elevated)' : 'transparent',
              color: activeTab === 'connections' ? 'var(--text-primary)' : 'var(--text-muted)',
              border: `1px solid ${activeTab === 'connections' ? 'var(--border-medium)' : 'var(--border-subtle)'}`,
            }}
            onClick={() => setActiveTab('connections')}
          >
            Active Connections ({connections.length})
          </button>

          <button
            className="btn"
            style={{
              backgroundColor: activeTab === 'listening' ? 'var(--bg-elevated)' : 'transparent',
              color: activeTab === 'listening' ? 'var(--text-primary)' : 'var(--text-muted)',
              border: `1px solid ${activeTab === 'listening' ? 'var(--border-medium)' : 'var(--border-subtle)'}`,
            }}
            onClick={() => setActiveTab('listening')}
          >
            Listening Ports ({listeningPorts.length})
          </button>

          <button
            className="btn"
            style={{
              backgroundColor: activeTab === 'destinations' ? 'var(--bg-elevated)' : 'transparent',
              color: activeTab === 'destinations' ? 'var(--text-primary)' : 'var(--text-muted)',
              border: `1px solid ${activeTab === 'destinations' ? 'var(--border-medium)' : 'var(--border-subtle)'}`,
            }}
            onClick={() => setActiveTab('destinations')}
          >
            Top Destinations ({destinations.length})
          </button>
        </div>

        {/* Search input */}
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: 6,
            background: 'var(--bg-card)',
            border: '1px solid var(--border-subtle)',
            borderRadius: 'var(--radius-md)',
            padding: '4px 10px',
            width: 260,
          }}
        >
          <Search size={14} className="text-muted" />
          <input
            type="text"
            placeholder="Filter by process, IP, port..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            style={{
              background: 'transparent',
              border: 'none',
              outline: 'none',
              color: 'var(--text-primary)',
              fontSize: '12px',
              width: '100%',
              fontFamily: 'inherit',
            }}
          />
        </div>
      </div>

      {/* Tab 1: Active Connections */}
      {activeTab === 'connections' && (
        <Card>
          {filteredConnections.length === 0 ? (
            <EmptyState
              title="No Matching Connections"
              message="No active connections match the current filter criteria."
            />
          ) : (
            <div className="table-container">
              <table className="data-table">
                <thead>
                  <tr>
                    <th>Process / Application</th>
                    <th>PID</th>
                    <th>Proto</th>
                    <th>Local Socket</th>
                    <th>Remote Destination</th>
                    <th>Hostname</th>
                    <th>State</th>
                    <th>Traffic</th>
                    <th>Status</th>
                  </tr>
                </thead>
                <tbody>
                  {filteredConnections.map((c) => (
                    <tr
                      key={c.id}
                      onClick={() => onInspectConnection && onInspectConnection(c)}
                      style={{ cursor: onInspectConnection ? 'pointer' : 'default' }}
                    >
                      <td>
                        <div className="font-semibold text-primary">{c.processName}</div>
                        <div className="text-xs text-muted">{c.applicationName}</div>
                      </td>
                      <td className="mono text-muted">{c.pid}</td>
                      <td>
                        <span className="badge badge-blue" style={{ fontSize: '9px', padding: '1px 5px' }}>
                          {c.protocol}
                        </span>
                      </td>
                      <td className="mono text-secondary">
                        {c.localAddress}:{c.localPort}
                      </td>
                      <td className="mono text-primary font-semibold">
                        {c.remoteAddress}:{c.remotePort}
                      </td>
                      <td className="text-muted" style={{ maxWidth: 180, overflow: 'hidden', textOverflow: 'ellipsis' }}>
                        {c.hostname || 'Hostname unavailable'}
                      </td>
                      <td>
                        <span className="mono text-xs text-muted">{c.state}</span>
                      </td>
                      <td className="mono text-xs text-cyan">
                        &darr;{formatBytes(c.bytesReceived)} &bull; &uarr;{formatBytes(c.bytesSent)}
                      </td>
                      <td>
                        {c.isUnusual ? (
                          <span className="badge badge-amber" style={{ fontSize: '9px' }}>UNUSUAL</span>
                        ) : c.isNew ? (
                          <span className="badge badge-blue" style={{ fontSize: '9px' }}>NEW</span>
                        ) : (
                          <span className="badge badge-green" style={{ fontSize: '9px' }}>NORMAL</span>
                        )}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </Card>
      )}

      {/* Tab 2: Listening Ports */}
      {activeTab === 'listening' && (
        <Card>
          {filteredPorts.length === 0 ? (
            <EmptyState
              title="No Listening Ports"
              message="No listening ports match the search criteria."
            />
          ) : (
            <div className="table-container">
              <table className="data-table">
                <thead>
                  <tr>
                    <th>Port</th>
                    <th>Protocol</th>
                    <th>Owning Process</th>
                    <th>PID</th>
                    <th>Application</th>
                    <th>Bind Interface</th>
                    <th>State</th>
                  </tr>
                </thead>
                <tbody>
                  {filteredPorts.map((p, idx) => (
                    <tr key={idx}>
                      <td className="mono text-cyan font-bold" style={{ fontSize: '13px' }}>
                        :{p.port}
                      </td>
                      <td>
                        <span className="badge badge-blue" style={{ fontSize: '9px', padding: '1px 5px' }}>
                          {p.protocol}
                        </span>
                      </td>
                      <td className="font-semibold text-primary">{p.processName}</td>
                      <td className="mono text-muted">{p.pid}</td>
                      <td className="text-secondary">{p.applicationName}</td>
                      <td className="mono text-muted">{p.bindAddress}</td>
                      <td>
                        <span className="badge badge-green" style={{ fontSize: '9px' }}>{p.state}</span>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </Card>
      )}

      {/* Tab 3: Top Destinations */}
      {activeTab === 'destinations' && (
        <Card>
          {filteredDestinations.length === 0 ? (
            <EmptyState
              title="No Destinations Observed"
              message="Awaiting socket observations to map destination patterns."
            />
          ) : (
            <div className="table-container">
              <table className="data-table">
                <thead>
                  <tr>
                    <th>Remote Destination</th>
                    <th>Resolved Hostname</th>
                    <th>Primary Application</th>
                    <th>Total Transfer</th>
                    <th>Sockets</th>
                    <th>Last Seen</th>
                    <th>Security Context</th>
                  </tr>
                </thead>
                <tbody>
                  {filteredDestinations.map((d, idx) => (
                    <tr key={idx}>
                      <td className="mono font-semibold text-primary">{d.destination}</td>
                      <td className="text-secondary">{d.hostname || 'Hostname unavailable'}</td>
                      <td>
                        <span className="badge badge-muted" style={{ fontSize: '9px', padding: '1px 5px' }}>
                          {d.application}
                        </span>
                      </td>
                      <td className="mono text-cyan font-semibold">{formatBytes(d.totalBytes)}</td>
                      <td className="mono text-muted">{d.connectionCount}</td>
                      <td className="mono text-xs text-muted">{d.lastSeen}</td>
                      <td>
                        {d.securityStatus === 'suspicious' ? (
                          <span className="badge badge-red" style={{ fontSize: '9px' }}>SUSPICIOUS</span>
                        ) : d.securityStatus === 'unusual' ? (
                          <span className="badge badge-amber" style={{ fontSize: '9px' }}>UNUSUAL</span>
                        ) : (
                          <span className="badge badge-green" style={{ fontSize: '9px' }}>NORMAL</span>
                        )}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </Card>
      )}
    </div>
  );
};
