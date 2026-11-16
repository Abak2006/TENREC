import * as React from 'react';
import { Undo2 } from 'lucide-react';
import { Button } from '@/components/ui/button';
import {
  AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogContent, AlertDialogDescription, AlertDialogFooter,
  AlertDialogHeader, AlertDialogTitle, AlertDialogTrigger,
} from '@/components/ui/alert-dialog';
import { DataTable, EmptyState, PageHeader, Section, Tag, Td, Th } from '@/components/common';
import { useToast } from '@/components/toast';
import { api } from '@/lib/api';
import { formatRelative, formatTime } from '@/lib/format';
import { store, useAppData } from '@/lib/store';
import type { ChangeKind, RestoreResult } from '@/lib/types';

const KIND_LABEL: Record<ChangeKind, string> = {
  firewall_rule: 'Firewall',
  priority: 'Priority',
  suspended: 'Suspended',
  tuning: 'Focus',
  setting: 'Setting',
  scheduled_task: 'Task',
  qos_policy: 'Upload limit',
  power_mode: 'Power',
};

export function ProtectionPage() {
  const policies = useAppData((s) => s.policies);
  const snapshots = useAppData((s) => s.snapshots);
  const audit = useAppData((s) => s.audit);
  const [busy, setBusy] = React.useState<string | null>(null);
  const toast = useToast();

  const report = (r: RestoreResult) => {
    const detail = r.messages.join('\n') || undefined;
    if (r.failed > 0) toast({ tone: 'error', title: `${r.restored} undone, ${r.failed} failed`, detail });
    else toast({ tone: 'ok', title: r.restored > 0 ? `${r.restored} change${r.restored === 1 ? '' : 's'} undone` : 'Nothing to undo', detail });
  };

  const run = async (key: string, fn: () => Promise<RestoreResult>) => {
    setBusy(key);
    try {
      report(await fn());
    } catch (e) {
      toast({ tone: 'error', title: 'Undo failed', detail: (e as Error).message });
    } finally {
      setBusy(null);
      void store.refreshAfterAction();
    }
  };

  return (
    <>
      <PageHeader
        title="Protection"
        description="Every change TENREC makes is recorded here and can be undone, including after a restart. TENREC only ever touches firewall rules named TENREC_RULE_*."
        actions={
          <AlertDialog>
            <AlertDialogTrigger asChild>
              <Button variant="outline" disabled={busy !== null}>
                <Undo2 /> Undo everything
              </Button>
            </AlertDialogTrigger>
            <AlertDialogContent>
              <AlertDialogHeader>
                <AlertDialogTitle>Undo every TENREC change?</AlertDialogTitle>
                <AlertDialogDescription>
                  Removes all TENREC firewall rules, restores process priorities and resumes suspended processes.
                  When running as administrator it also removes leftover TENREC_RULE_* rules from older versions.
                </AlertDialogDescription>
              </AlertDialogHeader>
              <AlertDialogFooter>
                <AlertDialogCancel>Cancel</AlertDialogCancel>
                <AlertDialogAction onClick={() => run('all', api.restoreAll)}>Undo everything</AlertDialogAction>
              </AlertDialogFooter>
            </AlertDialogContent>
          </AlertDialog>
        }
      />

      <div className="flex flex-col gap-4">
        <Section title="In effect now" bodyClassName="p-0">
          {policies.length === 0 ? (
            <EmptyState title="No TENREC changes are in effect">Approve an action from Security, Processes or Applications to see it here.</EmptyState>
          ) : (
            <ul className="divide-y">
              {policies.map((p) => (
                <li key={p.id} className="flex items-center gap-3 px-4 py-2.5">
                  <Tag tone="accent">{KIND_LABEL[p.kind]}</Tag>
                  <span className="min-w-0 flex-1 truncate">{p.description}</span>
                  <span className="text-xs text-muted-foreground">{formatRelative(p.createdAt)}</span>
                </li>
              ))}
            </ul>
          )}
        </Section>

        <Section title="History" description="Grouped by the approval that created them" bodyClassName="p-0">
          {snapshots.length === 0 ? (
            <EmptyState title="No history yet" />
          ) : (
            <ul className="divide-y">
              {snapshots.map((s) => (
                <li key={s.id} className="px-4 py-3">
                  <div className="flex items-center gap-2">
                    <span className="font-medium">{formatTime(s.timestamp)}</span>
                    {s.status === 'ACTIVE' && <Tag tone="accent">active</Tag>}
                    {s.status === 'RESTORED' && <Tag>undone</Tag>}
                    {s.status === 'PARTIALLY_RESTORED' && <Tag tone="warn">partly undone</Tag>}
                    {s.status !== 'RESTORED' && (
                      <Button size="sm" variant="outline" className="ml-auto" disabled={busy !== null} onClick={() => run(s.id, () => api.restoreSnapshot(s.id))}>
                        {busy === s.id ? 'Undoing…' : 'Undo'}
                      </Button>
                    )}
                  </div>
                  <ul className="mt-1.5 flex flex-col gap-0.5">
                    {s.kind === 'focus' && s.changes.length > 6 && (
                      <li className="text-sm text-muted-foreground">{s.changes.length} process and setting changes; first 6 shown</li>
                    )}
                    {(s.kind === 'focus' ? s.changes.slice(0, 6) : s.changes).map((c, i) => (
                      <li key={i} className="flex items-baseline gap-2 text-sm">
                        <span className={c.status === 'RESTORED' ? 'text-muted-foreground line-through' : ''}>{c.description}</span>
                        {c.status === 'RESTORE_FAILED' && <Tag tone="danger">undo failed</Tag>}
                        {c.note && <span className="text-xs text-muted-foreground">{c.note}</span>}
                      </li>
                    ))}
                  </ul>
                </li>
              ))}
            </ul>
          )}
        </Section>

        <Section title="Audit log" description="Last 100 actions, stored locally" bodyClassName="p-0">
          {audit.length === 0 ? (
            <EmptyState title="No actions recorded" />
          ) : (
            <DataTable label="Audit log" className="max-h-96 rounded-none border-0">
              <thead>
                <tr>
                  <Th>Time</Th>
                  <Th>Action</Th>
                  <Th>Target</Th>
                  <Th>Result</Th>
                </tr>
              </thead>
              <tbody>
                {audit.map((e) => (
                  <tr key={e.id}>
                    <Td className="whitespace-nowrap text-muted-foreground">{formatTime(e.timestamp)}</Td>
                    <Td className="whitespace-nowrap">{e.actionType.replaceAll('_', ' ')}</Td>
                    <Td className="max-w-md truncate">{e.target}</Td>
                    <Td className="max-w-md">
                      {e.success ? <Tag tone="ok">ok</Tag> : <Tag tone="danger">failed</Tag>}{' '}
                      {!e.success && <span className="text-xs text-muted-foreground">{e.details}</span>}
                    </Td>
                  </tr>
                ))}
              </tbody>
            </DataTable>
          )}
        </Section>
      </div>
    </>
  );
}
