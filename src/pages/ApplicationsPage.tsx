import React, { useState } from 'react';
import { Layers, Search, X } from 'lucide-react';
import { Card } from '../components/common/Card';
import { EmptyState } from '../components/common/EmptyState';
import type { ApplicationItem } from '../types';

interface ApplicationsPageProps {
  applications: ApplicationItem[];
  onRequestPolicy: (app: ApplicationItem) => void;
}

export const ApplicationsPage: React.FC<ApplicationsPageProps> = ({
  applications,
  onRequestPolicy,
}) => {
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedApp, setSelectedApp] = useState<ApplicationItem | null>(null);

  const formatBytes = (bytes: number): string => {
    if (bytes >= 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
    if (bytes >= 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    if (bytes >= 1024) return `${(bytes / 1024).toFixed(0)} KB`;
    return `${bytes} B`;
  };

  const filteredApps = applications.filter((app) => {
    const q = searchQuery.toLowerCase();
    return (
      app.name.toLowerCase().includes(q) ||
      app.publisher.toLowerCase().includes(q) ||
      app.executablePath.toLowerCase().includes(q)
    );
  });

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 14 }}>
      {/* Header & Search */}
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <div>
          <div className="text-lg font-bold text-primary">Applications ({applications.length})</div>
          <div className="text-xs text-muted">
            Aggregated application identities correlating multiple worker processes, resource loads, and network activity.
          </div>
        </div>

        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: 6,
            background: 'var(--bg-card)',
            border: '1px solid var(--border-subtle)',
            borderRadius: 'var(--radius-md)',
            padding: '4px 10px',
            width: 280,
          }}
        >
          <Search size={14} className="text-muted" />
          <input
            type="text"
            placeholder="Search application or publisher..."
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

      {/* Applications Table */}
      <Card>
        {filteredApps.length === 0 ? (
          <EmptyState
            title="No Matching Applications"
            message="No active applications found matching the current search query."
          />
        ) : (
          <div className="table-container">
            <table className="data-table">
              <thead>
                <tr>
                  <th>Application</th>
                  <th>Publisher</th>
                  <th>Processes</th>
                  <th>CPU Load</th>
                  <th>RAM Usage</th>
                  <th>Active Sockets</th>
                  <th>Baseline Status</th>
                  <th>Security Context</th>
                  <th>Action</th>
                </tr>
              </thead>
              <tbody>
                {filteredApps.map((app) => (
                  <tr
                    key={app.id}
                    onClick={() => setSelectedApp(app)}
                    style={{ cursor: 'pointer' }}
                  >
                    <td>
                      <div className="font-bold text-primary">{app.name}</div>
                      <div
                        className="text-xs text-muted mono"
                        style={{ maxWidth: 220, overflow: 'hidden', textOverflow: 'ellipsis' }}
                      >
                        {app.executablePath}
                      </div>
                    </td>
                    <td className="text-secondary" style={{ maxWidth: 160, overflow: 'hidden', textOverflow: 'ellipsis' }}>
                      {app.publisher || 'Unknown / Unsigned'}
                    </td>
                    <td className="mono text-muted">
                      {app.processCount} ({app.pids.slice(0, 3).join(', ')}{app.pids.length > 3 ? '...' : ''})
                    </td>
                    <td className="mono font-semibold text-primary">
                      {app.cpuPercent.toFixed(1)}%
                    </td>
                    <td className="mono text-secondary">
                      {formatBytes(app.memoryBytes)}
                    </td>
                    <td className="mono text-cyan">
                      {app.connectionCount}
                    </td>
                    <td>
                      {app.baselineStatus === 'normal' ? (
                        <span className="badge badge-green" style={{ fontSize: '9px' }}>NORMAL</span>
                      ) : app.baselineStatus === 'learning' ? (
                        <span className="badge badge-amber" style={{ fontSize: '9px' }}>LEARNING</span>
                      ) : (
                        <span className="badge badge-red" style={{ fontSize: '9px' }}>DIVERGENT</span>
                      )}
                    </td>
                    <td>
                      {app.securityStatus === 'incident' ? (
                        <span className="badge badge-red" style={{ fontSize: '9px' }}>INCIDENT</span>
                      ) : app.securityStatus === 'attention' ? (
                        <span className="badge badge-amber" style={{ fontSize: '9px' }}>ATTENTION</span>
                      ) : (
                        <span className="badge badge-green" style={{ fontSize: '9px' }}>HEALTHY</span>
                      )}
                    </td>
                    <td>
                      <button
                        className="btn btn-secondary"
                        style={{ fontSize: '10.5px', padding: '3px 8px' }}
                        onClick={(e) => {
                          e.stopPropagation();
                          onRequestPolicy(app);
                        }}
                      >
                        Create Policy
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </Card>

      {/* Application Detail Drawer / Modal */}
      {selectedApp && (
        <div className="modal-backdrop">
          <div className="modal-content" style={{ width: '640px' }}>
            <div className="modal-header">
              <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
                <Layers size={20} className="text-cyan" />
                <div>
                  <div className="text-base font-bold">{selectedApp.name}</div>
                  <div className="text-xs text-muted">{selectedApp.publisher || 'Unsigned binary'}</div>
                </div>
              </div>
              <button
                className="btn btn-secondary"
                style={{ padding: 4 }}
                onClick={() => setSelectedApp(null)}
              >
                <X size={16} />
              </button>
            </div>

            <div className="modal-body">
              <div className="grid-2" style={{ marginBottom: 14 }}>
                <div style={{ background: 'var(--bg-card)', padding: '10px 12px', borderRadius: 'var(--radius-md)' }}>
                  <span className="text-xs text-muted">Executable Location</span>
                  <div className="text-xs text-primary mono" style={{ wordBreak: 'break-all', marginTop: 4 }}>
                    {selectedApp.executablePath}
                  </div>
                </div>

                <div style={{ background: 'var(--bg-card)', padding: '10px 12px', borderRadius: 'var(--radius-md)' }}>
                  <span className="text-xs text-muted">Correlated Process IDs</span>
                  <div className="text-sm font-bold text-cyan mono" style={{ marginTop: 4 }}>
                    {selectedApp.pids.join(', ')}
                  </div>
                </div>
              </div>

              <div className="grid-3" style={{ marginBottom: 14 }}>
                <div style={{ background: 'var(--bg-card)', padding: '10px 12px', borderRadius: 'var(--radius-md)' }}>
                  <span className="text-xs text-muted">Processor Load</span>
                  <div className="text-lg font-bold text-primary mono" style={{ marginTop: 2 }}>
                    {selectedApp.cpuPercent.toFixed(1)}%
                  </div>
                </div>

                <div style={{ background: 'var(--bg-card)', padding: '10px 12px', borderRadius: 'var(--radius-md)' }}>
                  <span className="text-xs text-muted">RAM Working Set</span>
                  <div className="text-lg font-bold text-primary mono" style={{ marginTop: 2 }}>
                    {formatBytes(selectedApp.memoryBytes)}
                  </div>
                </div>

                <div style={{ background: 'var(--bg-card)', padding: '10px 12px', borderRadius: 'var(--radius-md)' }}>
                  <span className="text-xs text-muted">Active Sockets</span>
                  <div className="text-lg font-bold text-cyan mono" style={{ marginTop: 2 }}>
                    {selectedApp.connectionCount}
                  </div>
                </div>
              </div>

              <div style={{ background: 'var(--bg-card)', padding: '12px', borderRadius: 'var(--radius-md)' }}>
                <div className="text-xs font-semibold text-secondary" style={{ marginBottom: 6 }}>
                  Attribution &amp; Behavioral Baseline Context
                </div>
                <div className="text-xs text-muted" style={{ lineHeight: 1.5 }}>
                  This application was first observed on <strong className="text-secondary mono">{selectedApp.firstSeen}</strong>.
                  Current outbound connection volume and memory allocation are evaluated against the local machine baseline.
                  No autonomous changes are ever made without explicit review.
                </div>
              </div>
            </div>

            <div className="modal-footer">
              <button className="btn btn-secondary" onClick={() => setSelectedApp(null)}>
                Close
              </button>
              <button
                className="btn btn-primary"
                onClick={() => {
                  onRequestPolicy(selectedApp);
                  setSelectedApp(null);
                }}
              >
                Propose Temporary Policy
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
