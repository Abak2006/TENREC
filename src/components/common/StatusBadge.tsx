import React from 'react';

interface StatusBadgeProps {
  status: 'healthy' | 'attention' | 'critical' | 'info' | 'normal' | 'learning' | 'divergent';
  label?: string;
}

export const StatusBadge: React.FC<StatusBadgeProps> = ({ status, label }) => {
  let badgeClass = 'badge badge-green';
  let defaultLabel = 'HEALTHY';

  switch (status) {
    case 'attention':
    case 'learning':
      badgeClass = 'badge badge-amber';
      defaultLabel = 'ATTENTION';
      break;
    case 'critical':
    case 'divergent':
      badgeClass = 'badge badge-red';
      defaultLabel = 'CRITICAL';
      break;
    case 'info':
      badgeClass = 'badge badge-blue';
      defaultLabel = 'INFO';
      break;
    case 'normal':
    case 'healthy':
    default:
      badgeClass = 'badge badge-green';
      defaultLabel = 'NORMAL';
      break;
  }

  return (
    <span className={badgeClass}>
      <span
        style={{
          width: 6,
          height: 6,
          borderRadius: '50%',
          backgroundColor: 'currentColor',
          display: 'inline-block',
        }}
      />
      {label || defaultLabel}
    </span>
  );
};
