import * as React from 'react';
import { AlertTriangle, ChevronDown, ChevronUp, Info, Search } from 'lucide-react';
import { cn } from '@/lib/utils';
import type { SortState } from '@/lib/sort';
import type { Severity } from '@/lib/types';

export function PageHeader({ title, description, actions }: { title: string; description?: string; actions?: React.ReactNode }) {
  return (
    <header className="flex flex-wrap items-end justify-between gap-3 pb-5">
      <div className="min-w-0">
        <h1 className="text-xl font-semibold tracking-tight">{title}</h1>
        {description && <p className="mt-0.5 max-w-2xl text-muted-foreground">{description}</p>}
      </div>
      {actions && <div className="flex items-center gap-2">{actions}</div>}
    </header>
  );
}

export function Section({
  title,
  description,
  actions,
  children,
  className,
  bodyClassName,
}: {
  title?: React.ReactNode;
  description?: React.ReactNode;
  actions?: React.ReactNode;
  children: React.ReactNode;
  className?: string;
  bodyClassName?: string;
}) {
  return (
    <section className={cn('rounded-lg border bg-card text-card-foreground', className)}>
      {(title || actions) && (
        <div className="flex items-start justify-between gap-3 border-b px-4 py-3">
          <div className="min-w-0">
            {title && <h2 className="font-semibold">{title}</h2>}
            {description && <p className="text-xs text-muted-foreground">{description}</p>}
          </div>
          {actions}
        </div>
      )}
      <div className={cn('p-4', bodyClassName)}>{children}</div>
    </section>
  );
}

export function Sparkline({ values, max, className, colorVar = '--chart-1', label }: {
  values: number[];
  max?: number;
  className?: string;
  colorVar?: string;
  label: string;
}) {
  const w = 120;
  const h = 32;
  const top = Math.max(max ?? 0, ...values, 1);
  const pts = values.length > 1
    ? values.map((v, i) => `${(i / (values.length - 1)) * w},${h - (v / top) * (h - 2) - 1}`).join(' ')
    : '';
  return (
    <svg viewBox={`0 0 ${w} ${h}`} preserveAspectRatio="none" className={cn('h-8 w-full', className)} role="img" aria-label={label}>
      {pts && (
        <>
          <polygon points={`0,${h} ${pts} ${w},${h}`} fill={`var(${colorVar})`} opacity={0.12} />
          <polyline points={pts} fill="none" stroke={`var(${colorVar})`} strokeWidth={1.5} vectorEffect="non-scaling-stroke" />
        </>
      )}
    </svg>
  );
}

export function Meter({ value, label, className }: { value: number; label: string; className?: string }) {
  const v = Math.max(0, Math.min(100, value));
  const tone = v >= 90 ? 'bg-danger' : v >= 75 ? 'bg-warn' : 'bg-primary';
  return (
    <div
      role="meter"
      aria-label={label}
      aria-valuenow={Math.round(v)}
      aria-valuemin={0}
      aria-valuemax={100}
      className={cn('h-1.5 w-full overflow-hidden rounded-full bg-muted', className)}
    >
      <div className={cn('h-full rounded-full transition-[width] duration-500', tone)} style={{ width: `${v}%` }} />
    </div>
  );
}

export function Stat({ label, value, sub, children }: { label: string; value: React.ReactNode; sub?: React.ReactNode; children?: React.ReactNode }) {
  return (
    <div className="flex flex-col gap-2 rounded-lg border bg-card p-4">
      <div className="text-xs font-medium text-muted-foreground">{label}</div>
      <div className="num text-2xl font-semibold leading-none tracking-tight">{value}</div>
      {sub && <div className="num text-xs text-muted-foreground">{sub}</div>}
      {children}
    </div>
  );
}

export function EmptyState({ title, children, icon }: { title: string; children?: React.ReactNode; icon?: React.ReactNode }) {
  return (
    <div className="flex flex-col items-center justify-center gap-1 px-6 py-10 text-center">
      {icon && <div className="mb-1 text-muted-foreground">{icon}</div>}
      <div className="font-medium">{title}</div>
      {children && <div className="max-w-md text-muted-foreground">{children}</div>}
    </div>
  );
}

export function Banner({ tone = 'info', title, children, action }: {
  tone?: 'info' | 'warn' | 'danger';
  title: string;
  children?: React.ReactNode;
  action?: React.ReactNode;
}) {
  const Icon = tone === 'info' ? Info : AlertTriangle;
  return (
    <div
      role={tone === 'danger' ? 'alert' : 'status'}
      className={cn(
        'flex items-start gap-3 rounded-lg border px-4 py-3',
        tone === 'info' && 'border-primary/30 bg-primary/5',
        tone === 'warn' && 'border-warn/40 bg-warn/10',
        tone === 'danger' && 'border-danger/40 bg-danger/10',
      )}
    >
      <Icon className={cn('mt-0.5 size-4 shrink-0', tone === 'info' ? 'text-primary' : tone === 'warn' ? 'text-warn' : 'text-danger')} aria-hidden />
      <div className="min-w-0 flex-1">
        <div className="font-medium">{title}</div>
        {children && <div className="text-muted-foreground">{children}</div>}
      </div>
      {action}
    </div>
  );
}

export function SeverityBadge({ severity }: { severity: Severity }) {
  return (
    <span
      className={cn(
        'inline-flex items-center rounded px-1.5 py-0.5 text-[11px] font-semibold uppercase tracking-wide',
        severity === 'HIGH' && 'bg-danger/15 text-danger',
        severity === 'MEDIUM' && 'bg-warn/15 text-warn',
        severity === 'LOW' && 'bg-muted text-muted-foreground',
      )}
    >
      {severity.toLowerCase()}
    </span>
  );
}

export function Tag({ children, tone = 'neutral', title }: { children: React.ReactNode; tone?: 'neutral' | 'ok' | 'warn' | 'danger' | 'accent'; title?: string }) {
  return (
    <span
      title={title}
      className={cn(
        'inline-flex items-center whitespace-nowrap rounded px-1.5 py-0.5 text-[11px] font-medium',
        tone === 'neutral' && 'bg-muted text-muted-foreground',
        tone === 'ok' && 'bg-ok/15 text-ok',
        tone === 'warn' && 'bg-warn/15 text-warn',
        tone === 'danger' && 'bg-danger/15 text-danger',
        tone === 'accent' && 'bg-primary/12 text-primary',
      )}
    >
      {children}
    </span>
  );
}

export function SearchInput({ value, onChange, placeholder, label }: { value: string; onChange: (v: string) => void; placeholder: string; label: string }) {
  return (
    <div className="relative w-64 max-w-full">
      <Search className="pointer-events-none absolute top-1/2 left-2.5 size-3.5 -translate-y-1/2 text-muted-foreground" aria-hidden />
      <input
        type="search"
        aria-label={label}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder}
        className="h-8 w-full rounded-md border border-input bg-background pr-2 pl-8 text-sm placeholder:text-muted-foreground focus-visible:outline-2 focus-visible:outline-ring"
      />
    </div>
  );
}

export function SortHeader<K extends string>({ label, sortKey, sort, onSort, align = 'left', className }: {
  label: string;
  sortKey: K;
  sort: SortState<K>;
  onSort: (k: K) => void;
  align?: 'left' | 'right';
  className?: string;
}) {
  const active = sort.key === sortKey;
  return (
    <th
      scope="col"
      aria-sort={active ? (sort.dir === 'asc' ? 'ascending' : 'descending') : 'none'}
      className={cn('h-9 px-3 text-xs font-medium text-muted-foreground', align === 'right' ? 'text-right' : 'text-left', className)}
    >
      <button
        type="button"
        onClick={() => onSort(sortKey)}
        className={cn('inline-flex items-center gap-1 rounded hover:text-foreground', align === 'right' && 'flex-row-reverse', active && 'text-foreground')}
      >
        {label}
        {active ? (sort.dir === 'asc' ? <ChevronUp className="size-3" /> : <ChevronDown className="size-3" />) : <span className="size-3" />}
      </button>
    </th>
  );
}

export function Th({ children, align = 'left', className }: { children?: React.ReactNode; align?: 'left' | 'right'; className?: string }) {
  return (
    <th scope="col" className={cn('h-9 px-3 text-xs font-medium text-muted-foreground', align === 'right' ? 'text-right' : 'text-left', className)}>
      {children}
    </th>
  );
}

export function Td({ children, align = 'left', className, mono }: { children?: React.ReactNode; align?: 'left' | 'right'; className?: string; mono?: boolean }) {
  return (
    <td className={cn('h-9 px-3 align-middle', align === 'right' && 'num whitespace-nowrap text-right', mono && 'font-mono text-xs', className)}>{children}</td>
  );
}

/** Scrollable table container with a sticky header. */
export function DataTable({ children, className, label }: { children: React.ReactNode; className?: string; label: string }) {
  return (
    <div className={cn('overflow-auto rounded-lg border bg-card', className)}>
      <table aria-label={label} className="w-full border-collapse text-sm [&_thead]:sticky [&_thead]:top-0 [&_thead]:z-10 [&_thead]:bg-card [&_thead_tr]:border-b [&_tbody_tr]:border-b [&_tbody_tr:last-child]:border-0 [&_tbody_tr:hover]:bg-muted/50">
        {children}
      </table>
    </div>
  );
}
