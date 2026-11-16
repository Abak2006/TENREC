import * as React from 'react';
import { Ban, Crosshair } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { DataTable, EmptyState, PageHeader, SearchInput, SortHeader, Tag, Td, Th } from '@/components/common';
import { sortBy, useSort } from '@/lib/sort';
import { useActionReview } from '@/components/action-review';
import { api } from '@/lib/api';
import { formatBytes, formatPercent, formatRate } from '@/lib/format';
import { useAppData } from '@/lib/store';
import { setPendingFocusApp, useNavigate } from '@/lib/navigation';
import type { ApplicationItem } from '@/lib/types';

type Key = 'name' | 'processes' | 'cpu' | 'memory' | 'io' | 'connections';

const value = (a: ApplicationItem, k: Key) =>
  k === 'name' ? a.name : k === 'processes' ? a.processCount : k === 'cpu' ? a.cpuPercent : k === 'memory' ? a.memoryBytes
    : k === 'io' ? a.ioReadBps + a.ioWriteBps : a.connectionCount;

const baseline = {
  learning: { tone: 'neutral', label: 'learning', title: 'Fewer than 10 samples so far' },
  normal: { tone: 'ok', label: 'normal', title: 'Within its usual CPU and I/O range this session' },
  divergent: { tone: 'warn', label: 'unusual', title: 'CPU or I/O well above its average this session' },
} as const;

export function ApplicationsPage() {
  const apps = useAppData((s) => s.applications);
  const [query, setQuery] = React.useState('');
  const [onlineOnly, setOnlineOnly] = React.useState(false);
  const { sort, toggle } = useSort<Key>({ key: 'cpu', dir: 'desc' });
  const review = useActionReview();
  const navigate = useNavigate();

  const q = query.trim().toLowerCase();
  const rows = sortBy(
    apps.filter((a) => (!onlineOnly || a.connectionCount > 0) && (!q || a.name.toLowerCase().includes(q) || a.executablePath.toLowerCase().includes(q) || (a.company ?? '').toLowerCase().includes(q))),
    sort,
    value,
  );

  return (
    <>
      <PageHeader
        title="Applications"
        description="Processes grouped by executable. Names and companies come from each file's version information, which any program can set; they are not a signature check."
        actions={
          <>
            <label className="flex items-center gap-2 text-sm">
              <input type="checkbox" className="size-4 accent-[var(--primary)]" checked={onlineOnly} onChange={(e) => setOnlineOnly(e.target.checked)} />
              With connections only
            </label>
            <SearchInput value={query} onChange={setQuery} placeholder="Filter applications" label="Filter applications" />
          </>
        }
      />
      <DataTable label="Applications" className="max-h-[calc(100vh-10rem)]">
        <thead>
          <tr>
            <SortHeader label="Application" sortKey="name" sort={sort} onSort={toggle} />
            <SortHeader label="Procs" sortKey="processes" sort={sort} onSort={toggle} align="right" />
            <SortHeader label="CPU" sortKey="cpu" sort={sort} onSort={toggle} align="right" />
            <SortHeader label="Memory" sortKey="memory" sort={sort} onSort={toggle} align="right" />
            <SortHeader label="I/O" sortKey="io" sort={sort} onSort={toggle} align="right" />
            <SortHeader label="Conn." sortKey="connections" sort={sort} onSort={toggle} align="right" />
            <Th>Behaviour</Th>
            <Th align="right"><span className="sr-only">Actions</span></Th>
          </tr>
        </thead>
        <tbody>
          {rows.map((a) => {
            const b = baseline[a.baselineStatus];
            return (
              <tr key={a.id} className="group">
                <Td className="max-w-96">
                  <div className="truncate font-medium">{a.name}</div>
                  <div className="truncate text-xs text-muted-foreground" title={a.executablePath || undefined}>
                    {[a.company, a.executablePath || 'Path unavailable'].filter(Boolean).join(' · ')}
                  </div>
                </Td>
                <Td align="right">{a.processCount}</Td>
                <Td align="right">
                  {a.restrictedProcessCount === a.processCount ? (
                    <span className="text-muted-foreground" title="Needs administrator rights to measure">n/a</span>
                  ) : (
                    <span title={a.restrictedProcessCount > 0 ? `${a.restrictedProcessCount} of ${a.processCount} processes could not be measured` : undefined}>
                      {formatPercent(a.cpuPercent, 1)}{a.restrictedProcessCount > 0 ? '*' : ''}
                    </span>
                  )}
                </Td>
                <Td align="right">{formatBytes(a.memoryBytes)}</Td>
                <Td align="right">{a.ioReadBps + a.ioWriteBps > 0 ? formatRate(a.ioReadBps + a.ioWriteBps) : '–'}</Td>
                <Td align="right">{a.connectionCount || '–'}</Td>
                <Td><Tag tone={b.tone} title={b.title}>{b.label}</Tag></Td>
                <Td align="right">
                  <div className="flex justify-end gap-1 opacity-60 group-hover:opacity-100 group-focus-within:opacity-100">
                  <Button
                    size="sm"
                    variant="ghost"
                    disabled={!a.executablePath}
                    onClick={() => {
                      setPendingFocusApp(a.id);
                      navigate('focus');
                    }}
                  >
                    <Crosshair /> Focus
                  </Button>
                  <Button
                    size="sm"
                    variant="ghost"
                    disabled={!a.executablePath}
                    title={a.executablePath ? undefined : 'Windows did not report a path for this application'}
                    onClick={() => review(async () => [await api.prepareAppBlock(a.id)])}
                  >
                    <Ban /> Block internet
                  </Button>
                  </div>
                </Td>
              </tr>
            );
          })}
        </tbody>
      </DataTable>
      {rows.length === 0 && <EmptyState title="No matching applications" />}
    </>
  );
}
