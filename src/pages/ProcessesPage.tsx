import React, { useState } from 'react';
import { TerminalSquare, Search, X } from 'lucide-react';
import { Card } from '../components/common/Card';
import { EmptyState } from '../components/common/EmptyState';
import type { ProcessItem } from '../types';

interface ProcessesPageProps {
  processes: ProcessItem[];
  onRequestProcessControl: (proc: ProcessItem, actionType: 'suspend' | 'priority' | 'terminate') => void;
}

export const ProcessesPage: React.FC<ProcessesPageProps> = ({
  processes,
  onRequestProcessControl,
}) => {
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedProcess, setSelectedProcess] = useState<ProcessItem | null>(null);
  const [sortBy, setSortBy] = useState<'cpu' | 'memory' | 'name' | 'pid'>('cpu');
  const [sortAsc, setSortAsc] = useState(false);

  const formatBytes = (bytes: number): string => {
    if (bytes >= 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
    if (bytes >= 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    if (bytes >= 1024) return `${(bytes / 1024).toFixed(0)} KB`;
    return `${bytes} B`;
  };

  const handleSort = (field: 'cpu' | 'memory' | 'name' | 'pid') => {
    if (sortBy === field) {
      setSortAsc(!sortAsc);
    } else {
      setSortBy(field);
      setSortAsc(false);
    }
  };

  const filteredProcesses = processes
    .filter((p) => {
      const q = searchQuery.toLowerCase();
      return (
        p.name.toLowerCase().includes(q) ||
        p.pid.toString().includes(q) ||
        p.publisher.toLowerCase().includes(q) ||
        p.executablePath.toLowerCase().includes(q)
      );
    })
    .sort((a, b) => {
      let diff = 0;
      if (sortBy === 'cpu') diff = b.cpuPercent - a.cpuPercent;
      else if (sortBy === 'memory') diff = b.memoryBytes - a.memoryBytes;
      else if (sortBy === 'pid') diff = b.pid - a.pid;
      else diff = a.name.localeCompare(b.name);
      return sortAsc ? -diff : diff;
    });

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 14 }}>
      {/* Header and Filter Controls */}
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <div>
          <div className="text-lg font-bold text-primary">Process Explorer ({processes.length})</div>
          <div className="text-xs text-muted">
            Individual OS process runtime metrics, parent linkage, and publisher signatures.
          </div>
        </div>

        <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
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
              placeholder="Search process name, PID, path..."
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
      </div>

      {/* Processes Table */}
      <Card>
        {filteredProcesses.length === 0 ? (
          <EmptyState
            title="No Matching Processes"
            message="No active processes match the search criteria."
          />
        ) : (
          <div className="table-container" style={{ maxHeight: 'calc(100vh - 220px)', overflowY: 'auto' }}>
            <table className="data-table">
              <thead>
                <tr>
                  <th onClick={() => handleSort('name')} style={{ cursor: 'pointer' }}>
                    Process Name {sortBy === 'name' && (sortAsc ? '▲' : '▼')}
                  </th>
                  <th onClick={() => handleSort('pid')} style={{ cursor: 'pointer' }}>
                    PID {sortBy === 'pid' && (sortAsc ? '▲' : '▼')}
                  </th>
                  <th>Parent PID</th>
                  <th>Publisher</th>
                  <th onClick={() => handleSort('cpu')} style={{ cursor: 'pointer' }}>
                    CPU % {sortBy === 'cpu' && (sortAsc ? '▲' : '▼')}
                  </th>
                  <th onClick={() => handleSort('memory')} style={{ cursor: 'pointer' }}>
                    Memory {sortBy === 'memory' && (sortAsc ? '▲' : '▼')}
                  </th>
                  <th>Sockets</th>
                  <th>Privilege</th>
                  <th>Action</th>
                </tr>
              </thead>
              <tbody>
                {filteredProcesses.map((p) => (
                  <tr
                    key={p.pid}
                    onClick={() => setSelectedProcess(p)}
                    style={{ cursor: 'pointer' }}
                  >
                    <td>
                      <div className="font-semibold text-primary">{p.name}</div>
                      <div
                        className="text-xs text-muted mono"
                        style={{ maxWidth: 220, overflow: 'hidden', textOverflow: 'ellipsis' }}
                      >
                        {p.executablePath}
                      </div>
                    </td>
                    <td className="mono text-cyan font-bold">{p.pid}</td>
                    <td className="mono text-muted">{p.ppid !== null ? p.ppid : '--'}</td>
                    <td className="text-secondary" style={{ maxWidth: 150, overflow: 'hidden', textOverflow: 'ellipsis' }}>
                      {p.publisher || 'Unknown / Unsigned'}
                    </td>
                    <td className="mono font-semibold text-primary">
                      {p.cpuPercent.toFixed(1)}%
                    </td>
                    <td className="mono text-secondary">
                      {formatBytes(p.memoryBytes)}
                    </td>
                    <td className="mono text-cyan">
                      {p.connectionCount}
                    </td>
                    <td>
                      {p.isElevated ? (
                        <span className="badge badge-amber" style={{ fontSize: '9px' }}>ELEVATED</span>
                      ) : (
                        <span className="badge badge-muted" style={{ fontSize: '9px' }}>USER</span>
                      )}
                    </td>
                    <td>
                      <button
                        className="btn btn-secondary"
                        style={{ fontSize: '10px', padding: '2px 7px' }}
                        onClick={(e) => {
                          e.stopPropagation();
                          setSelectedProcess(p);
                        }}
                      >
                        Inspect
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </Card>

      {/* Process Detail & Control Modal */}
      {selectedProcess && (
        <div className="modal-backdrop">
          <div className="modal-content" style={{ width: '660px' }}>
            <div className="modal-header">
              <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
                <TerminalSquare size={20} className="text-cyan" />
                <div>
                  <div className="text-base font-bold">
                    {selectedProcess.name} &bull; PID {selectedProcess.pid}
                  </div>
                  <div className="text-xs text-muted">{selectedProcess.publisher || 'Unsigned'}</div>
                </div>
              </div>
              <button
                className="btn btn-secondary"
                style={{ padding: 4 }}
                onClick={() => setSelectedProcess(null)}
              >
                <X size={16} />
              </button>
            </div>

            <div className="modal-body">
              <div className="grid-2" style={{ marginBottom: 14 }}>
                <div style={{ background: 'var(--bg-card)', padding: '10px 12px', borderRadius: 'var(--radius-md)' }}>
                  <span className="text-xs text-muted">Full Path</span>
                  <div className="text-xs text-primary mono" style={{ wordBreak: 'break-all', marginTop: 4 }}>
                    {selectedProcess.executablePath}
                  </div>
                </div>

                <div style={{ background: 'var(--bg-card)', padding: '10px 12px', borderRadius: 'var(--radius-md)' }}>
                  <span className="text-xs text-muted">Process Ancestry</span>
                  <div className="text-xs text-secondary mono" style={{ marginTop: 4 }}>
                    Parent PID: <strong>{selectedProcess.ppid !== null ? selectedProcess.ppid : 'None (Root/System)'}</strong>
                  </div>
                  <div className="text-xs text-secondary mono" style={{ marginTop: 2 }}>
                    Started: {selectedProcess.startTime}
                  </div>
                </div>
              </div>

              <div className="grid-3" style={{ marginBottom: 14 }}>
                <div style={{ background: 'var(--bg-card)', padding: '10px 12px', borderRadius: 'var(--radius-md)' }}>
                  <span className="text-xs text-muted">Processor Load</span>
                  <div className="text-lg font-bold text-primary mono" style={{ marginTop: 2 }}>
                    {selectedProcess.cpuPercent.toFixed(1)}%
                  </div>
                </div>

                <div style={{ background: 'var(--bg-card)', padding: '10px 12px', borderRadius: 'var(--radius-md)' }}>
                  <span className="text-xs text-muted">Working Set Memory</span>
                  <div className="text-lg font-bold text-primary mono" style={{ marginTop: 2 }}>
                    {formatBytes(selectedProcess.memoryBytes)}
                  </div>
                </div>

                <div style={{ background: 'var(--bg-card)', padding: '10px 12px', borderRadius: 'var(--radius-md)' }}>
                  <span className="text-xs text-muted">Active Sockets</span>
                  <div className="text-lg font-bold text-cyan mono" style={{ marginTop: 2 }}>
                    {selectedProcess.connectionCount}
                  </div>
                </div>
              </div>

              {/* Risky Controls Section */}
              <div
                style={{
                  background: 'rgba(239, 68, 68, 0.06)',
                  border: '1px solid var(--status-red-border)',
                  borderRadius: 'var(--radius-md)',
                  padding: '12px',
                }}
              >
                <div className="text-xs font-bold text-red" style={{ marginBottom: 4 }}>
                  Risky Process Controls (Requires Approval)
                </div>
                <div className="text-xs text-muted" style={{ marginBottom: 10 }}>
                  Suspending or terminating processes may cause unsaved data loss in target applications.
                  All operations create a restoration snapshot.
                </div>

                <div style={{ display: 'flex', gap: 8 }}>
                  <button
                    className="btn btn-secondary"
                    style={{ fontSize: '11px' }}
                    onClick={() => {
                      onRequestProcessControl(selectedProcess, 'priority');
                      setSelectedProcess(null);
                    }}
                  >
                    Change Priority
                  </button>

                  <button
                    className="btn btn-secondary"
                    style={{ fontSize: '11px' }}
                    onClick={() => {
                      onRequestProcessControl(selectedProcess, 'suspend');
                      setSelectedProcess(null);
                    }}
                  >
                    Suspend Process
                  </button>

                  <button
                    className="btn btn-danger"
                    style={{ fontSize: '11px' }}
                    onClick={() => {
                      onRequestProcessControl(selectedProcess, 'terminate');
                      setSelectedProcess(null);
                    }}
                  >
                    Terminate Process
                  </button>
                </div>
              </div>
            </div>

            <div className="modal-footer">
              <button className="btn btn-secondary" onClick={() => setSelectedProcess(null)}>
                Close
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
