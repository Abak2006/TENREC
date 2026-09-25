import React from 'react';

interface MetricCardProps {
  label: string;
  value: string | number;
  unit?: string;
  secondary?: string;
  icon?: React.ReactNode;
  trend?: 'up' | 'down' | 'stable';
  status?: 'normal' | 'attention' | 'critical';
  progress?: number; // 0 to 100
  onClick?: () => void;
}

export const MetricCard: React.FC<MetricCardProps> = ({
  label,
  value,
  unit,
  secondary,
  icon,
  status = 'normal',
  progress,
  onClick,
}) => {
  const getStatusColor = () => {
    switch (status) {
      case 'attention':
        return 'var(--status-amber)';
      case 'critical':
        return 'var(--status-red)';
      default:
        return 'var(--accent-cyan)';
    }
  };

  return (
    <div
      className="card"
      style={{
        padding: '12px 14px',
        cursor: onClick ? 'pointer' : 'default',
        position: 'relative',
        overflow: 'hidden',
      }}
      onClick={onClick}
    >
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', marginBottom: 6 }}>
        <span className="text-xs font-semibold text-muted" style={{ letterSpacing: '0.04em', textTransform: 'uppercase' }}>
          {label}
        </span>
        {icon && <span style={{ color: getStatusColor(), opacity: 0.85 }}>{icon}</span>}
      </div>

      <div style={{ display: 'flex', alignItems: 'baseline', gap: 4, marginBottom: 4 }}>
        <span className="mono text-2xl font-bold" style={{ color: 'var(--text-primary)' }}>
          {value}
        </span>
        {unit && <span className="text-xs text-muted font-medium">{unit}</span>}
      </div>

      {progress !== undefined && (
        <div
          style={{
            height: 4,
            background: 'var(--border-subtle)',
            borderRadius: 2,
            overflow: 'hidden',
            margin: '6px 0',
          }}
        >
          <div
            style={{
              height: '100%',
              width: `${Math.min(100, Math.max(0, progress))}%`,
              background: getStatusColor(),
              transition: 'width 0.3s ease',
            }}
          />
        </div>
      )}

      {secondary && <div className="text-xs text-secondary mono" style={{ fontSize: '11px' }}>{secondary}</div>}
    </div>
  );
};
