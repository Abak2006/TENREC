import * as React from 'react';
import { Gauge, Pause } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { DataTable, EmptyState, PageHeader, SearchInput, SortHeader, Tag, Td, Th } from '@/components/common';
import { sortBy, useSort } from '@/lib/sort';
import { useActionReview } from '@/components/action-review';
import { api } from '@/lib/api';
import { formatBytes, formatPercent, formatRate } from '@/lib/format';
import { useAppData } from '@/lib/store';
import type { ProcessItem } from '@/lib/types';

type Key = 'name' | 'pid' | 'cpu' | 'memory' | 'io' | 'connections' | 'company';

const value = (p: ProcessItem, k: Key) =>
  k === 'name' ? p.name : k === 'pid' ? p.pid : k === 'cpu' ? p.cpuPercent : k === 'memory' ? p.memoryBytes
    : k === 'io' ? p.ioReadBps + p.ioWriteBps : k === 'connections' ? p.connectionCount : p.company ?? '';

function Restricted() {
  return (
    <span className="text-muted-foreground" title="Windows only reports this for service and system processes when TENREC runs as administrator">
      n/a
    </span>
  );
}

export function ProcessesPage() {
  const processes = useAppData((s) => s.processes);
  const [query, setQuery] = React.useState('');
  const { sort, toggle } = useSort<Key>({ key: 'cpu', dir: 'desc' });
  const review = useActionReview();

  const q = query.trim().toLowerCase();
  const rows = sortBy(
    processes.filter((p) => p.pid !== 0 && (!q || p.name.toLowerCase().includes(q) || String(p.pid) === q || p.executablePath.toLowerCase().includes(q) || p.applicationName.toLowerCase().includes(q))),
    sort,
    value,
  );

  return (
    <>
      <PageHeader
        title="Processes"
        description="Every running process. CPU is a share of total capacity across all cores. I/O counts file, device and network reads and writes. Without administrator rights Windows hides CPU and I/O of service and system processes (shown as n/a)."
        actions={<SearchInput value={query} onChange={setQuery} placeholder="Filter by name, PID or path" label="Filter processes" />}
      />
      <div className="mb-2 text-xs text-muted-foreground num">{rows.length} of {processes.length - 1} processes</div>
      <DataTable label="Processes" className="max-h-[calc(100vh-11rem)]">
        <thead>
          <tr>
            <SortHeader label="Name" sortKey="name" sort={sort} onSort={toggle} />
            <SortHeader label="PID" sortKey="pid" sort={sort} onSort={toggle} align="right" />
            <SortHeader label="CPU" sortKey="cpu" sort={sort} onSort={toggle} align="right" />
            <SortHeader label="Memory" sortKey="memory" sort={sort} onSort={toggle} align="right" />
            <SortHeader label="I/O" sortKey="io" sort={sort} onSort={toggle} align="right" />
            <SortHeader label="Conn." sortKey="connections" sort={sort} onSort={toggle} align="right" />
            <SortHeader label="Company (unverified)" sortKey="company" sort={sort} onSort={toggle} className="hidden xl:table-cell" />
            <Th align="right"><span className="sr-only">Actions</span></Th>
          </tr>
        </thead>
        <tbody>
          {rows.map((p) => (
            <tr key={p.pid} className="group">
              <Td className="max-w-80">
                <div className="flex items-center gap-2">
                  <span className="truncate font-medium" title={p.executablePath || undefined}>{p.name}</span>
                  {p.runsFromUserWritableLocation && <Tag tone="warn" title="Runs from a Downloads or Temp folder">temp/downloads</Tag>}
                </div>
                <div className="truncate text-xs text-muted-foreground">{p.executablePath || 'Path unavailable (protected or other user)'}</div>
              </Td>
              <Td align="right">{p.pid}</Td>
              <Td align="right">{p.restricted ? <Restricted /> : formatPercent(p.cpuPercent, 1)}</Td>
              <Td align="right">{formatBytes(p.memoryBytes)}</Td>
              <Td align="right">{p.restricted ? <Restricted /> : p.ioReadBps + p.ioWriteBps > 0 ? formatRate(p.ioReadBps + p.ioWriteBps) : '–'}</Td>
              <Td align="right">{p.connectionCount || '–'}</Td>
              <Td className="hidden max-w-48 truncate text-muted-foreground xl:table-cell">{p.company ?? '–'}</Td>
              <Td align="right">
                <div className="flex justify-end gap-1 opacity-60 group-hover:opacity-100 group-focus-within:opacity-100">
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button size="icon-sm" variant="ghost" aria-label={`Lower priority of ${p.name}`} onClick={() => review(async () => [await api.prepareProcessAction(p.pid, 'priority')])}>
                        <Gauge />
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent>Lower priority</TooltipContent>
                  </Tooltip>
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button size="icon-sm" variant="ghost" aria-label={`Suspend ${p.name}`} onClick={() => review(async () => [await api.prepareProcessAction(p.pid, 'suspend')])}>
                        <Pause />
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent>Suspend</TooltipContent>
                  </Tooltip>
                </div>
              </Td>
            </tr>
          ))}
        </tbody>
      </DataTable>
      {rows.length === 0 && <EmptyState title="No matching processes">Try a different name, PID or path.</EmptyState>}
    </>
  );
}
