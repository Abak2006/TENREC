import React from 'react';
import { ShieldCheck, RotateCcw, Clock } from 'lucide-react';
import { Card } from '../components/common/Card';
import { EmptyState } from '../components/common/EmptyState';
import type { ActivePolicy, RestorationSnapshot } from '../types';

interface ProtectionPageProps {
  policies: ActivePolicy[];
  snapshots: RestorationSnapshot[];
  onRestoreSnapshot: (snapshotId: string) => void;
  onRestoreAll: () => void;
}

export const ProtectionPage: React.FC<ProtectionPageProps> = ({
  policies,
  snapshots,
  onRestoreSnapshot,
  onRestoreAll,
}) => {
  const activePolicies = policies.filter((p) => p.status === 'ACTIVE');

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 14 }}>
      {/* Header and Master Rollback Button */}
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <div>
          <div className="text-lg font-bold text-primary">Protection Engine &amp; Policy Control</div>
          <div className="text-xs text-muted">
            All firewall rules and process throttles were explicitly approved by you. Every action can be reversed instantly.
          </div>
        </div>

        <button
          className="btn btn-restore"
          style={{ padding: '6px 14px' }}
          disabled={activePolicies.length === 0 && snapshots.length === 0}
          onClick={onRestoreAll}
        >
          <RotateCcw size={15} />
          Restore Everything ({activePolicies.length})
        </button>
      </div>

      {/* Active Policies Table */}
      <Card
        title="Active Enforcement Policies"
        subtitle="Current active Windows Firewall rules and process throttles"
        icon={<ShieldCheck size={16} />}
      >
        {activePolicies.length === 0 ? (
          <EmptyState
            title="No Active Protection Policies"
            message="No active restrictions applied to any application or endpoint. System operates under standard default policies."
          />
        ) : (
          <div className="table-container">
            <table className="data-table">
              <thead>
                <tr>
                  <th>Policy Name</th>
                  <th>Type</th>
                  <th>Target Application</th>
                  <th>Endpoint</th>
                  <th>Applied At</th>
                  <th>Expires</th>
                  <th>Reason</th>
                  <th>Status</th>
                </tr>
              </thead>
              <tbody>
                {activePolicies.map((p) => (
                  <tr key={p.id}>
                    <td className="font-semibold text-primary">{p.name}</td>
                    <td>
                      <span className="badge badge-blue" style={{ fontSize: '9px' }}>
                        {p.policyType}
                      </span>
                    </td>
                    <td className="text-secondary">{p.applicationName}</td>
                    <td className="mono text-muted">{p.destinationEndpoint || 'All Outbound'}</td>
                    <td className="mono text-xs text-muted">{p.createdAt}</td>
                    <td className="mono text-xs text-amber">{p.expiresAt || 'Until Restored'}</td>
                    <td className="text-muted text-xs" style={{ maxWidth: 220, overflow: 'hidden', textOverflow: 'ellipsis' }}>
                      {p.reason}
                    </td>
                    <td>
                      <span className="badge badge-green" style={{ fontSize: '9px' }}>ACTIVE</span>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </Card>

      {/* Restoration Snapshots Table */}
      <Card
        title="Restoration Snapshots"
        subtitle="Automatic snapshots captured immediately prior to applying any system modification"
        icon={<Clock size={16} />}
      >
        {snapshots.length === 0 ? (
          <EmptyState
            title="No Restoration Snapshots"
            message="No system modifications have been approved yet. Snapshots will be recorded automatically when actions are applied."
          />
        ) : (
          <div className="table-container">
            <table className="data-table">
              <thead>
                <tr>
                  <th>Snapshot ID</th>
                  <th>Timestamp</th>
                  <th>Session Title</th>
                  <th>Changes Captured</th>
                  <th>Status</th>
                  <th>Action</th>
                </tr>
              </thead>
              <tbody>
                {snapshots.map((s) => (
                  <tr key={s.id}>
                    <td className="mono text-cyan">{s.id.slice(0, 8)}...</td>
                    <td className="mono text-xs text-muted">{s.timestamp}</td>
                    <td className="font-semibold text-primary">{s.title}</td>
                    <td>
                      <span className="text-xs text-secondary">
                        {s.actionSummary.join(', ')}
                      </span>
                    </td>
                    <td>
                      {s.status === 'ACTIVE' ? (
                        <span className="badge badge-amber" style={{ fontSize: '9px' }}>ACTIVE</span>
                      ) : (
                        <span className="badge badge-muted" style={{ fontSize: '9px' }}>RESTORED</span>
                      )}
                    </td>
                    <td>
                      {s.status === 'ACTIVE' && (
                        <button
                          className="btn btn-secondary"
                          style={{ fontSize: '10.5px', padding: '2px 8px' }}
                          onClick={() => onRestoreSnapshot(s.id)}
                        >
                          <RotateCcw size={12} />
                          Restore
                        </button>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </Card>
    </div>
  );
};
