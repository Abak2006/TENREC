import * as React from 'react';
import { CheckCircle2, ShieldAlert, XCircle } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Banner, Tag } from '@/components/common';
import { useToast } from '@/components/toast';
import { api } from '@/lib/api';
import { store, useAppData } from '@/lib/store';
import type { ActionOutcome, ProposedAction } from '@/lib/types';
import { cn } from '@/lib/utils';

type Open = (load: () => Promise<ProposedAction[]>) => void;
const ReviewContext = React.createContext<Open>(() => {});

/** Opens the review dialog. `load` fetches backend-built actions to show. */
export function useActionReview() {
  return React.useContext(ReviewContext);
}

type Phase = { kind: 'closed' } | { kind: 'loading' } | { kind: 'error'; message: string } | { kind: 'review' } | { kind: 'applying' } | { kind: 'done'; outcomes: ActionOutcome[] };

export function ActionReviewProvider({ children }: { children: React.ReactNode }) {
  const [phase, setPhase] = React.useState<Phase>({ kind: 'closed' });
  const [actions, setActions] = React.useState<ProposedAction[]>([]);
  const [selected, setSelected] = React.useState<Set<string>>(new Set());
  const elevated = useAppData((s) => s.status?.elevated ?? false);
  const toast = useToast();

  const open: Open = React.useCallback((load) => {
    setPhase({ kind: 'loading' });
    setActions([]);
    load()
      .then((list) => {
        if (list.length === 0) {
          setPhase({ kind: 'error', message: 'There are no actions to review for this item.' });
          return;
        }
        setActions(list);
        setSelected(new Set(list.filter((a) => a.defaultSelected).map((a) => a.id)));
        setPhase({ kind: 'review' });
      })
      .catch((e: Error) => setPhase({ kind: 'error', message: e.message }));
  }, []);

  const toggle = (id: string) =>
    setSelected((s) => {
      const n = new Set(s);
      if (n.has(id)) n.delete(id);
      else n.add(id);
      return n;
    });

  const apply = async () => {
    setPhase({ kind: 'applying' });
    try {
      const res = await api.applyActions([...selected]);
      const ok = res.outcomes.filter((o) => o.success).length;
      if (ok > 0) toast({ tone: 'ok', title: `${ok} change${ok === 1 ? '' : 's'} applied`, detail: 'You can undo it from the Protection page.' });
      setPhase({ kind: 'done', outcomes: res.outcomes });
      void store.refreshAfterAction();
    } catch (e) {
      setPhase({ kind: 'error', message: (e as Error).message });
    }
  };

  const close = () => setPhase({ kind: 'closed' });
  const needsAdmin = actions.some((a) => selected.has(a.id) && a.requiresAdmin) && !elevated;

  return (
    <ReviewContext.Provider value={open}>
      {children}
      <Dialog open={phase.kind !== 'closed'} onOpenChange={(o) => !o && phase.kind !== 'applying' && close()}>
        <DialogContent className="sm:max-w-xl">
          <DialogHeader>
            <DialogTitle>{phase.kind === 'done' ? 'Result' : 'Review changes'}</DialogTitle>
            <DialogDescription>
              {phase.kind !== 'done'
                ? 'Nothing changes until you apply. Every change is recorded and can be undone.'
                : phase.outcomes.some((o) => o.success)
                  ? 'Applied changes are recorded and can be undone from the Protection page.'
                  : 'Nothing was changed.'}
            </DialogDescription>
          </DialogHeader>

          {phase.kind === 'loading' && <p className="py-6 text-center text-muted-foreground">Preparing actions…</p>}

          {phase.kind === 'error' && (
            <Banner tone="danger" title="Can't do that">
              {phase.message}
            </Banner>
          )}

          {(phase.kind === 'review' || phase.kind === 'applying') && (
            <div className="flex max-h-[55vh] flex-col gap-2 overflow-y-auto pr-1">
              {needsAdmin && (
                <Banner tone="warn" title="Administrator rights needed">
                  Firewall changes will fail unless TENREC is started with “Run as administrator”. Process changes still work.
                </Banner>
              )}
              {actions.map((a) => {
                const checked = selected.has(a.id);
                return (
                  <label
                    key={a.id}
                    className={cn(
                      'flex cursor-pointer gap-3 rounded-lg border p-3 transition-colors',
                      checked ? 'border-primary/50 bg-primary/5' : 'hover:bg-muted/50',
                    )}
                  >
                    <input
                      type="checkbox"
                      className="mt-1 size-4 shrink-0 accent-[var(--primary)]"
                      checked={checked}
                      disabled={phase.kind === 'applying'}
                      onChange={() => toggle(a.id)}
                    />
                    <div className="min-w-0 flex-1">
                      <div className="flex flex-wrap items-center gap-2">
                        <span className="font-medium">{a.title}</span>
                        <Tag tone={a.risk === 'SAFE' ? 'ok' : 'warn'}>{a.risk === 'SAFE' ? 'Low risk' : 'Disruptive'}</Tag>
                        {a.requiresAdmin && <Tag>Needs admin</Tag>}
                      </div>
                      <p className="mt-1 text-muted-foreground">{a.description}</p>
                      <dl className="mt-2 grid gap-1 text-xs sm:grid-cols-2">
                        <div>
                          <dt className="font-medium">Effect</dt>
                          <dd className="text-muted-foreground">{a.expectedBenefit}</dd>
                        </div>
                        <div>
                          <dt className="font-medium">Side effects</dt>
                          <dd className="text-muted-foreground">{a.potentialConsequence}</dd>
                        </div>
                      </dl>
                    </div>
                  </label>
                );
              })}
            </div>
          )}

          {phase.kind === 'done' && (
            <ul className="flex flex-col gap-2">
              {phase.outcomes.map((o) => (
                <li key={o.actionId} className="flex items-start gap-2 rounded-lg border p-3">
                  {o.success ? <CheckCircle2 className="mt-0.5 size-4 shrink-0 text-ok" aria-label="Applied" /> : <XCircle className="mt-0.5 size-4 shrink-0 text-danger" aria-label="Failed" />}
                  <div className="min-w-0">
                    <div className="font-medium">{o.title}</div>
                    {o.error && <div className="text-muted-foreground">{o.error}</div>}
                  </div>
                </li>
              ))}
            </ul>
          )}

          <DialogFooter>
            {phase.kind === 'review' || phase.kind === 'applying' ? (
              <>
                <Button variant="outline" onClick={close} disabled={phase.kind === 'applying'}>
                  Cancel
                </Button>
                <Button onClick={apply} disabled={selected.size === 0 || phase.kind === 'applying'}>
                  <ShieldAlert />
                  {phase.kind === 'applying' ? 'Applying…' : `Apply ${selected.size || ''} change${selected.size === 1 ? '' : 's'}`}
                </Button>
              </>
            ) : (
              <Button variant="outline" onClick={close}>
                Close
              </Button>
            )}
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </ReviewContext.Provider>
  );
}
