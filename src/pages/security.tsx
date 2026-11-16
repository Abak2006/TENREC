import * as React from 'react';
import { ShieldCheck } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { EmptyState, PageHeader, Section, SeverityBadge, Tag } from '@/components/common';
import { useActionReview } from '@/components/action-review';
import { useToast } from '@/components/toast';
import { api } from '@/lib/api';
import { formatRelative, formatTime } from '@/lib/format';
import { store, useAppData } from '@/lib/store';
import type { SecurityIncident } from '@/lib/types';
import { cn } from '@/lib/utils';

export function SecurityPage() {
  const incidents = useAppData((s) => s.incidents);
  const [showDismissed, setShowDismissed] = React.useState(false);
  const [selectedId, setSelectedId] = React.useState<string | null>(null);
  const review = useActionReview();
  const toast = useToast();

  const list = incidents.filter((i) => showDismissed || i.status === 'ACTIVE');
  const selected = list.find((i) => i.id === selectedId) ?? list[0] ?? null;
  const dismissedCount = incidents.filter((i) => i.status === 'DISMISSED').length;

  const setStatus = async (i: SecurityIncident, status: 'ACTIVE' | 'DISMISSED') => {
    try {
      await api.setIncidentStatus(i.id, status);
      await store.refreshLive();
    } catch (e) {
      toast({ tone: 'error', title: 'Could not update incident', detail: (e as Error).message });
    }
  };

  return (
    <>
      <PageHeader
        title="Security"
        description="Simple, explainable rules over live process and socket data. They flag things worth a look; they are not malware detection, and normal software can trigger them."
        actions={
          dismissedCount > 0 && (
            <label className="flex items-center gap-2 text-sm">
              <input type="checkbox" className="size-4 accent-[var(--primary)]" checked={showDismissed} onChange={(e) => setShowDismissed(e.target.checked)} />
              Show dismissed ({dismissedCount})
            </label>
          )
        }
      />

      {list.length === 0 ? (
        <Section>
          <EmptyState title="Nothing to review" icon={<ShieldCheck className="size-6 text-ok" />}>
            TENREC checks every 2 seconds for script hosts talking to the internet, programs running from Downloads or Temp going online or accepting
            connections, and listeners on common backdoor ports.
          </EmptyState>
        </Section>
      ) : (
        <div className="grid gap-4 lg:grid-cols-[minmax(0,2fr)_minmax(0,3fr)]">
          <ul className="flex flex-col gap-2" aria-label="Incidents">
            {list.map((i) => (
              <li key={i.id}>
                <button
                  type="button"
                  onClick={() => setSelectedId(i.id)}
                  aria-current={selected?.id === i.id}
                  className={cn(
                    'w-full rounded-lg border bg-card p-3 text-left transition-colors',
                    selected?.id === i.id ? 'border-primary/60 ring-1 ring-primary/30' : 'hover:bg-muted/50',
                    i.status === 'DISMISSED' && 'opacity-60',
                  )}
                >
                  <div className="flex items-center gap-2">
                    <SeverityBadge severity={i.severity} />
                    {i.currentlyObserved ? <Tag tone="danger">ongoing</Tag> : <Tag>ended</Tag>}
                    <span className="ml-auto text-xs text-muted-foreground">{formatRelative(i.lastObserved)}</span>
                  </div>
                  <div className="mt-1.5 font-medium">{i.title}</div>
                  <div className="truncate text-xs text-muted-foreground">{i.summary}</div>
                </button>
              </li>
            ))}
          </ul>

          {selected && (
            <Section
              title={selected.title}
              description={`Rule: ${selected.detector.replaceAll('_', ' ')}`}
              actions={<SeverityBadge severity={selected.severity} />}
            >
              <p>{selected.summary}</p>
              <h3 className="mt-4 text-xs font-semibold uppercase tracking-wide text-muted-foreground">Why this was flagged</h3>
              <p className="mt-1 text-muted-foreground">{selected.whyDetected}</p>

              <h3 className="mt-4 text-xs font-semibold uppercase tracking-wide text-muted-foreground">Evidence</h3>
              <dl className="mt-1 grid grid-cols-[max-content_minmax(0,1fr)] gap-x-4 gap-y-1">
                {selected.evidence.map((e) => (
                  <React.Fragment key={e.label}>
                    <dt className="text-muted-foreground">{e.label}</dt>
                    <dd className="break-all font-mono text-xs leading-5">{e.value}</dd>
                  </React.Fragment>
                ))}
                <dt className="text-muted-foreground">First seen</dt>
                <dd>{formatTime(selected.firstObserved)}</dd>
                <dt className="text-muted-foreground">Last seen</dt>
                <dd>{formatTime(selected.lastObserved)} <span className="text-muted-foreground">({selected.observationCount} samples)</span></dd>
              </dl>

              <div className="mt-5 flex flex-wrap gap-2 border-t pt-4">
                <Button onClick={() => review(() => api.proposedActions(selected.id))} disabled={selected.proposedActionIds.length === 0}>
                  Review actions
                </Button>
                {selected.status === 'ACTIVE' ? (
                  <Button variant="outline" onClick={() => setStatus(selected, 'DISMISSED')}>Dismiss</Button>
                ) : (
                  <Button variant="outline" onClick={() => setStatus(selected, 'ACTIVE')}>Restore to active</Button>
                )}
              </div>
            </Section>
          )}
        </div>
      )}
    </>
  );
}
