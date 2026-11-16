import { ArrowRight, ShieldCheck } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Banner, DataTable, EmptyState, Meter, PageHeader, Section, SeverityBadge, Sparkline, Stat, Tag, Td, Th } from '@/components/common';
import { useActionReview } from '@/components/action-review';
import { api } from '@/lib/api';
import { formatBytes, formatDuration, formatPercent, formatRate, formatRelative } from '@/lib/format';
import { useNavigate } from '@/lib/navigation';
import { useAppData } from '@/lib/store';

export function OverviewPage() {
  const snapshot = useAppData((s) => s.snapshot);
  const throughput = useAppData((s) => s.throughput);
  const history = useAppData((s) => s.history);
  const processes = useAppData((s) => s.processes);
  const incidents = useAppData((s) => s.incidents);
  const changes = useAppData((s) => s.changes);
  const policies = useAppData((s) => s.policies);
  const status = useAppData((s) => s.status);
  const navigate = useNavigate();
  const review = useActionReview();

  if (!snapshot) return null;
  const active = incidents.filter((i) => i.status === 'ACTIVE');
  const top = processes.filter((p) => p.pid > 4).slice(0, 8);
  const sysDrive = snapshot.disk.drives.find((d) => d.mountPoint.toUpperCase().startsWith('C')) ?? snapshot.disk.drives[0];

  return (
    <>
      <PageHeader
        title="Overview"
        description={`${snapshot.hostname} · ${snapshot.osName} · up ${formatDuration(snapshot.uptimeSeconds)}`}
      />

      <div className="mb-4">
        {active.length > 0 ? (
          <Banner
            tone={active.some((i) => i.severity === 'HIGH') ? 'danger' : 'warn'}
            title={`${active.length} item${active.length === 1 ? '' : 's'} to review`}
            action={
              <Button size="sm" variant="outline" onClick={() => navigate('security')}>
                Review <ArrowRight />
              </Button>
            }
          >
            {active[0].title}
            {active.length > 1 ? ` and ${active.length - 1} more` : ''}
          </Banner>
        ) : (
          <div className="flex items-center gap-2 rounded-lg border bg-card px-4 py-3">
            <ShieldCheck className="size-4 text-ok" aria-hidden />
            <span className="font-medium">Nothing needs your attention.</span>
            <span className="text-muted-foreground">
              {policies.length > 0 ? `${policies.length} change${policies.length === 1 ? '' : 's'} made by TENREC still in effect.` : 'No changes made by TENREC are in effect.'}
            </span>
          </div>
        )}
      </div>

      <div className="grid grid-cols-2 gap-3 xl:grid-cols-4">
        <Stat label="CPU" value={formatPercent(snapshot.cpu.usagePercent)} sub={`${snapshot.cpu.logicalCoreCount} logical cores`}>
          <Sparkline values={history.cpu} max={100} label="CPU usage, last two minutes" />
        </Stat>
        <Stat label="Memory" value={formatPercent(snapshot.memory.usagePercent)} sub={`${formatBytes(snapshot.memory.usedBytes)} of ${formatBytes(snapshot.memory.totalBytes)}`}>
          <Sparkline values={history.memory} max={100} label="Memory usage, last two minutes" />
        </Stat>
        <Stat label="Network" value={formatRate((throughput?.bytesInSec ?? 0) + (throughput?.bytesOutSec ?? 0))} sub={`↓ ${formatRate(throughput?.bytesInSec)} · ↑ ${formatRate(throughput?.bytesOutSec)}`}>
          <Sparkline values={history.netIn.map((v, i) => v + (history.netOut[i] ?? 0))} label="Network throughput, last two minutes" colorVar="--chart-2" />
        </Stat>
        <Stat label="Process I/O" value={formatRate(snapshot.io.readBytesSec + snapshot.io.writeBytesSec)} sub={`read ${formatRate(snapshot.io.readBytesSec)} · write ${formatRate(snapshot.io.writeBytesSec)}`}>
          {sysDrive && (
            <div className="flex flex-col gap-1">
              <Meter value={sysDrive.usagePercent} label={`${sysDrive.mountPoint} used`} />
              <span className="num text-xs text-muted-foreground">
                {sysDrive.mountPoint} {formatBytes(sysDrive.freeBytes)} free
              </span>
            </div>
          )}
        </Stat>
      </div>

      <div className="mt-4 grid gap-4 xl:grid-cols-[minmax(0,3fr)_minmax(0,2fr)]">
        <Section
          title="Busiest processes"
          description={status?.elevated ? 'By share of total CPU' : 'By share of total CPU · service and system processes need administrator rights to measure'}
          actions={<Button size="sm" variant="ghost" onClick={() => navigate('processes')}>All processes <ArrowRight /></Button>}
          bodyClassName="p-0"
        >
          <DataTable label="Busiest processes" className="rounded-none border-0">
            <thead>
              <tr>
                <Th>Process</Th>
                <Th align="right">CPU</Th>
                <Th align="right">Memory</Th>
                <Th align="right">Connections</Th>
              </tr>
            </thead>
            <tbody>
              {top.map((p) => (
                <tr key={p.pid}>
                  <Td>
                    <div className="font-medium">{p.applicationName}</div>
                    <div className="text-xs text-muted-foreground">{p.name} · PID {p.pid}</div>
                  </Td>
                  <Td align="right">{formatPercent(p.cpuPercent, 1)}</Td>
                  <Td align="right">{formatBytes(p.memoryBytes)}</Td>
                  <Td align="right">{p.connectionCount || '–'}</Td>
                </tr>
              ))}
            </tbody>
          </DataTable>
        </Section>

        <div className="flex flex-col gap-4">
          {active.length > 0 && (
            <Section title="Needs review" bodyClassName="p-0">
              <ul className="divide-y">
                {active.slice(0, 4).map((i) => (
                  <li key={i.id} className="flex items-start justify-between gap-3 px-4 py-3">
                    <div className="min-w-0">
                      <div className="flex items-center gap-2">
                        <SeverityBadge severity={i.severity} />
                        <span className="truncate font-medium">{i.title}</span>
                      </div>
                      <div className="text-xs text-muted-foreground">{formatRelative(i.lastObserved)}</div>
                    </div>
                    <Button size="sm" variant="outline" onClick={() => review(() => api.proposedActions(i.id))}>
                      Actions
                    </Button>
                  </li>
                ))}
              </ul>
            </Section>
          )}
          <Section title="What changed" description="Since TENREC started" bodyClassName="p-0">
            {changes.length === 0 ? (
              <EmptyState title="No changes yet">New applications, destinations and listening ports appear here as they happen.</EmptyState>
            ) : (
              <ul className="max-h-80 divide-y overflow-y-auto">
                {changes.slice(0, 30).map((c) => (
                  <li key={c.id} className="px-4 py-2.5">
                    <div className="flex items-center gap-2">
                      {c.severity === 'attention' && <Tag tone="warn">exposed</Tag>}
                      <span className="truncate font-medium">{c.title}</span>
                      <span className="ml-auto shrink-0 text-xs text-muted-foreground">{formatRelative(c.timestamp)}</span>
                    </div>
                    <div className="truncate text-xs text-muted-foreground">{c.description}</div>
                  </li>
                ))}
              </ul>
            )}
          </Section>
          {!status?.elevated && (
            <p className="px-1 text-xs text-muted-foreground">
              TENREC is running as a standard user. Monitoring works fully; firewall actions need “Run as administrator”.
            </p>
          )}
        </div>
      </div>
    </>
  );
}
