import React from 'react';

interface CardProps {
  title?: string;
  subtitle?: string;
  icon?: React.ReactNode;
  action?: React.ReactNode;
  children: React.ReactNode;
  className?: string;
}

export const Card: React.FC<CardProps> = ({
  title,
  subtitle,
  icon,
  action,
  children,
  className = '',
}) => {
  return (
    <div className={`card ${className}`}>
      {(title || action) && (
        <div className="card-header">
          <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
            {icon && <span style={{ color: 'var(--accent-cyan)' }}>{icon}</span>}
            <div>
              <div className="card-title">{title}</div>
              {subtitle && <div className="text-xs text-muted">{subtitle}</div>}
            </div>
          </div>
          {action && <div>{action}</div>}
        </div>
      )}
      {children}
    </div>
  );
};
