import React from 'react';
import {
  LayoutDashboard,
  Cpu,
  Network,
  Layers,
  TerminalSquare,
  ShieldCheck,
  ShieldAlert,
  Sliders,
  RotateCcw,
} from 'lucide-react';

export type PageId =
  | 'overview'
  | 'system'
  | 'network'
  | 'applications'
  | 'processes'
  | 'security'
  | 'protection'
  | 'settings';

interface SidebarProps {
  currentPage: PageId;
  onSelectPage: (page: PageId) => void;
  incidentCount: number;
  activePolicyCount: number;
  onRestoreAllClick: () => void;
}

export const Sidebar: React.FC<SidebarProps> = ({
  currentPage,
  onSelectPage,
  incidentCount,
  activePolicyCount,
  onRestoreAllClick,
}) => {
  const navItems: { id: PageId; label: string; icon: React.ReactNode; badge?: number }[] = [
    { id: 'overview', label: 'Overview', icon: <LayoutDashboard size={17} /> },
    { id: 'system', label: 'System', icon: <Cpu size={17} /> },
    { id: 'network', label: 'Network', icon: <Network size={17} /> },
    { id: 'applications', label: 'Applications', icon: <Layers size={17} /> },
    { id: 'processes', label: 'Processes', icon: <TerminalSquare size={17} /> },
    {
      id: 'security',
      label: 'Security',
      icon: <ShieldAlert size={17} />,
      badge: incidentCount > 0 ? incidentCount : undefined,
    },
    {
      id: 'protection',
      label: 'Protection',
      icon: <ShieldCheck size={17} />,
      badge: activePolicyCount > 0 ? activePolicyCount : undefined,
    },
    { id: 'settings', label: 'Settings', icon: <Sliders size={17} /> },
  ];

  return (
    <aside className="sidebar">
      {/* Brand Header */}
      <div
        style={{
          padding: '16px 18px',
          borderBottom: '1px solid var(--border-subtle)',
          display: 'flex',
          alignItems: 'center',
          gap: 10,
        }}
      >
        <div
          style={{
            width: 30,
            height: 30,
            borderRadius: 'var(--radius-md)',
            background: 'linear-gradient(135deg, #2563eb, #06b6d4)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            color: '#fff',
            fontWeight: 800,
            fontSize: '14px',
            boxShadow: '0 2px 8px rgba(37, 99, 235, 0.4)',
          }}
        >
          T
        </div>
        <div>
          <div className="font-bold text-base" style={{ letterSpacing: '0.04em', color: '#fff' }}>
            TENREC
          </div>
          <div className="text-xs text-muted" style={{ fontSize: '10px', textTransform: 'uppercase' }}>
            Intelligence & Control
          </div>
        </div>
      </div>

      {/* Navigation List */}
      <nav style={{ flex: 1, padding: '12px 10px', overflowY: 'auto' }}>
        <div
          className="text-xs font-semibold text-muted"
          style={{ padding: '0 8px 8px', fontSize: '10px', textTransform: 'uppercase', letterSpacing: '0.05em' }}
        >
          Platform
        </div>
        {navItems.map((item) => {
          const isActive = currentPage === item.id;
          return (
            <button
              key={item.id}
              onClick={() => onSelectPage(item.id)}
              style={{
                width: '100%',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                padding: '8px 10px',
                borderRadius: 'var(--radius-md)',
                backgroundColor: isActive ? 'var(--bg-elevated)' : 'transparent',
                color: isActive ? 'var(--text-primary)' : 'var(--text-secondary)',
                border: `1px solid ${isActive ? 'var(--border-medium)' : 'transparent'}`,
                cursor: 'pointer',
                marginBottom: 3,
                fontSize: '12.5px',
                fontWeight: isActive ? 600 : 500,
                transition: 'all 0.12s ease',
                textAlign: 'left',
              }}
            >
              <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
                <span style={{ color: isActive ? 'var(--accent-cyan)' : 'var(--text-muted)' }}>
                  {item.icon}
                </span>
                <span>{item.label}</span>
              </div>
              {item.badge !== undefined && (
                <span
                  className="badge badge-amber"
                  style={{ fontSize: '10px', padding: '1px 6px', borderRadius: '10px' }}
                >
                  {item.badge}
                </span>
              )}
            </button>
          );
        })}
      </nav>

      {/* Emergency Restoration Anchor */}
      <div
        style={{
          padding: '12px 14px',
          borderTop: '1px solid var(--border-subtle)',
          backgroundColor: 'var(--bg-primary)',
        }}
      >
        <button
          className="btn btn-restore"
          style={{ width: '100%', justifyContent: 'center' }}
          onClick={onRestoreAllClick}
        >
          <RotateCcw size={14} />
          Restore Everything
        </button>
        <div className="text-xs text-muted" style={{ textAlign: 'center', marginTop: 6, fontSize: '10px' }}>
          Rollback all active policies
        </div>
      </div>
    </aside>
  );
};
