import React from 'react';
import { AlertCircle } from 'lucide-react';

interface EmptyStateProps {
  title: string;
  message: string;
  icon?: React.ReactNode;
  action?: React.ReactNode;
}

export const EmptyState: React.FC<EmptyStateProps> = ({
  title,
  message,
  icon = <AlertCircle size={28} className="text-muted" />,
  action,
}) => {
  return (
    <div
      style={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        padding: '36px 20px',
        textAlign: 'center',
        background: 'rgba(15, 20, 32, 0.4)',
        borderRadius: 'var(--radius-lg)',
        border: '1px dashed var(--border-subtle)',
      }}
    >
      <div style={{ marginBottom: 10, opacity: 0.7 }}>{icon}</div>
      <div className="text-sm font-semibold text-secondary" style={{ marginBottom: 4 }}>
        {title}
      </div>
      <div className="text-xs text-muted" style={{ maxWidth: 360, marginBottom: action ? 14 : 0 }}>
        {message}
      </div>
      {action && <div>{action}</div>}
    </div>
  );
};
