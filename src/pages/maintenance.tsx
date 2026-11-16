import * as React from 'react';
import { RefreshCw, Trash2 } from 'lucide-react';
import { Button } from '@/components/ui/button';
import {
  AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogContent, AlertDialogDescription, AlertDialogFooter,
  AlertDialogHeader, AlertDialogTitle, AlertDialogTrigger,
} from '@/components/ui/alert-dialog';
import { Banner, PageHeader, Section, Tag } from '@/components/common';
import { useToast } from '@/components/toast';
import { api } from '@/lib/api';
import { formatBytes } from '@/lib/format';
import { store, useAppData } from '@/lib/store';
import type { CategoryScan, PowerState } from '@/lib/types';
import { cn } from '@/lib/utils';

const MODES = [
  { id: 'best_efficiency', label: 'Best power efficiency' },
  { id: 'balanced', label: 'Balanced' },
  { id: 'best_performance', label: 'Best performance' },
] as const;

function Power() {
  const [state, setState] = React.useState<PowerState | null>(null);
  const [error, setError] = React.useState<string | null>(null);
  const [busy, setBusy] = React.useState(false);
  const toast = useToast();

  const load = React.useCallback(() => {
    api.powerState().then(
      (s) => {
        setState(s);
        setError(null);
      },
      (e: Error) => setError(e.message),
    );
  }, []);
  React.useEffect(load, [load]);

  const change = async (fn: () => Promise<{ description: string }>) => {
    setBusy(true);
    try {
      const r = await fn();
      toast({ tone: 'ok', title: r.description, detail: 'Undo it from the Protection page.' });
    } catch (e) {
      toast({ tone: 'error', title: 'Could not change power setting', detail: (e as Error).message });
    } finally {
      setBusy(false);
      load();
      void store.refreshSlow();
    }
  };

  return (
    <Section title="Power" description="Windows power mode and power plans. Focus mode can also switch the power mode only while it runs.">
      {error && <Banner tone="danger" title="Could not read power settings">{error}</Banner>}
      {!state && !error && <p className="text-muted-foreground">Reading power settings…</p>}
      {state && (
        <div className="flex flex-col gap-5">
          <div>
            <div className="mb-2 font-medium">Power mode</div>
            {state.mode === null ? (
              <p className="text-muted-foreground">Not available with the current power plan. Plans like Ultimate Performance do not support power modes; switch to Balanced to use them.</p>
            ) : (
              <div role="radiogroup" aria-label="Power mode" className="inline-flex rounded-lg border p-0.5">
                {MODES.map((m) => (
                  <button
                    key={m.id}
                    type="button"
                    role="radio"
                    aria-checked={state.mode === m.id}
                    disabled={busy || state.mode === m.id}
                    onClick={() => change(() => api.setPowerMode(m.id))}
                    className={cn('rounded-md px-3 py-1.5 text-sm', state.mode === m.id ? 'bg-primary text-primary-foreground' : 'hover:bg-muted')}
                  >
                    {m.label}
                  </button>
                ))}
              </div>
            )}
          </div>
          <div>
            <div className="mb-2 font-medium">Power plan</div>
            <ul className="flex flex-col gap-1.5">
              {state.plans.map((p) => (
                <li key={p.guid} className="flex items-center gap-3">
                  <span className={cn('size-2 rounded-full', p.active ? 'bg-primary' : 'bg-muted')} aria-hidden />
                  <span className="flex-1">{p.name}</span>
                  {p.active ? (
                    <Tag tone="accent">active</Tag>
                  ) : (
                    <Button size="sm" variant="outline" disabled={busy} onClick={() => change(() => api.setPowerPlan(p.guid))}>Use</Button>
                  )}
                </li>
              ))}
            </ul>
          </div>
        </div>
      )}
    </Section>
  );
}

function Cleanup() {
  const [scan, setScan] = React.useState<CategoryScan[] | null>(null);
  const [scanning, setScanning] = React.useState(true);
  const [busy, setBusy] = React.useState<string | null>(null);
  const elevated = useAppData((s) => s.status?.elevated ?? false);
  const toast = useToast();

  const fetchScan = React.useCallback(() => {
    api.scanCleanup().then(setScan, (e: Error) => toast({ tone: 'error', title: 'Scan failed', detail: e.message })).finally(() => setScanning(false));
  }, [toast]);
  React.useEffect(fetchScan, [fetchScan]);
  const runScan = () => {
    setScanning(true);
    fetchScan();
  };

  const clean = async (c: CategoryScan) => {
    setBusy(c.category);
    try {
      const r = await api.runCleanup(c.category);
      if (r.error) toast({ tone: 'error', title: `${c.title}: not cleaned`, detail: r.error });
      else
        toast({
          tone: 'ok',
          title: `${c.title}: ${formatBytes(r.freedBytes)} freed`,
          detail: `${r.deletedFiles} file(s) deleted${r.skippedFiles ? `, ${r.skippedFiles} in use and skipped` : ''}.`,
        });
    } catch (e) {
      toast({ tone: 'error', title: 'Cleanup failed', detail: (e as Error).message });
    } finally {
      setBusy(null);
      runScan();
      void store.refreshSlow();
    }
  };

  const total = scan?.reduce((n, c) => n + c.bytes, 0) ?? 0;

  return (
    <Section
      title="Storage cleanup"
      description="Deleting files cannot be undone, so each category asks first. Files in use are skipped."
      actions={<Button size="sm" variant="ghost" onClick={runScan} disabled={scanning} aria-label="Scan again"><RefreshCw /></Button>}
      bodyClassName="p-0"
    >
      {!scan && <p className="p-4 text-muted-foreground">Scanning…</p>}
      {scan && (
        <>
          <ul className="divide-y">
            {scan.map((c) => {
              const locked = c.requiresAdmin && !elevated;
              return (
                <li key={c.category} className="flex items-center gap-4 px-4 py-3">
                  <div className="min-w-0 flex-1">
                    <div className="flex items-center gap-2">
                      <span className="font-medium">{c.title}</span>
                      {c.requiresAdmin && <Tag>needs admin</Tag>}
                    </div>
                    <p className="text-muted-foreground">{c.description}</p>
                    {c.error && !locked && <p className="text-xs text-danger">{c.error}</p>}
                  </div>
                  <div className="num w-28 text-right">
                    <div className="font-medium">{locked && c.bytes === 0 ? '–' : formatBytes(c.bytes)}</div>
                    <div className="text-xs text-muted-foreground">{locked && c.files === 0 ? 'admin to scan' : `${c.files} item(s)`}</div>
                  </div>
                  <AlertDialog>
                    <AlertDialogTrigger asChild>
                      <Button size="sm" variant="outline" disabled={busy !== null || c.bytes === 0 || locked}>
                        <Trash2 /> {busy === c.category ? 'Cleaning…' : 'Clean'}
                      </Button>
                    </AlertDialogTrigger>
                    <AlertDialogContent>
                      <AlertDialogHeader>
                        <AlertDialogTitle>Delete {formatBytes(c.bytes)} of {c.title.toLowerCase()}?</AlertDialogTitle>
                        <AlertDialogDescription>{c.description} This cannot be undone.</AlertDialogDescription>
                      </AlertDialogHeader>
                      <AlertDialogFooter>
                        <AlertDialogCancel>Cancel</AlertDialogCancel>
                        <AlertDialogAction onClick={() => clean(c)}>Delete</AlertDialogAction>
                      </AlertDialogFooter>
                    </AlertDialogContent>
                  </AlertDialog>
                </li>
              );
            })}
          </ul>
          <div className="num border-t px-4 py-2 text-right text-xs text-muted-foreground">{formatBytes(total)} can be freed</div>
        </>
      )}
    </Section>
  );
}

export function MaintenancePage() {
  return (
    <>
      <PageHeader title="Maintenance" description="Power settings and disk cleanup. Power changes can be undone from the Protection page; deleted files cannot." />
      <div className="flex flex-col gap-4">
        <Power />
        <Cleanup />
      </div>
    </>
  );
}
