import * as React from 'react';
import { Activity, AppWindow, Cpu, Crosshair, Gauge, Network, Settings, ShieldAlert, Undo2 } from 'lucide-react';
import { ActionReviewProvider } from '@/components/action-review';
import { Banner } from '@/components/common';
import { ToastProvider } from '@/components/toast';
import { TooltipProvider } from '@/components/ui/tooltip';
import { useAppData } from '@/lib/store';
import { NavigateContext, type PageId } from '@/lib/navigation';
import { cn } from '@/lib/utils';
import { OverviewPage } from '@/pages/overview';
import { FocusPage } from '@/pages/focus';
import { ProcessesPage } from '@/pages/processes';
import { ApplicationsPage } from '@/pages/applications';
import { NetworkPage } from '@/pages/network';
import { SecurityPage } from '@/pages/security';
import { ProtectionPage } from '@/pages/protection';
import { SystemPage } from '@/pages/system';
import { SettingsPage } from '@/pages/settings';


const NAV: { id: PageId; label: string; icon: React.ComponentType<{ className?: string }> }[] = [
  { id: 'overview', label: 'Overview', icon: Gauge },
  { id: 'focus', label: 'Focus', icon: Crosshair },
  { id: 'processes', label: 'Processes', icon: Activity },
  { id: 'applications', label: 'Applications', icon: AppWindow },
  { id: 'network', label: 'Network', icon: Network },
  { id: 'security', label: 'Security', icon: ShieldAlert },
  { id: 'protection', label: 'Protection', icon: Undo2 },
  { id: 'system', label: 'System', icon: Cpu },
  { id: 'settings', label: 'Settings', icon: Settings },
];

function Sidebar({ page, onSelect }: { page: PageId; onSelect: (p: PageId) => void }) {
  const incidents = useAppData((s) => s.incidents);
  const policies = useAppData((s) => s.policies);
  const status = useAppData((s) => s.status);
  const activeIncidents = incidents.filter((i) => i.status === 'ACTIVE').length;
  const focusActive = useAppData((s) => s.focus?.active ?? false);

  return (
    <nav aria-label="Main" className="flex w-52 shrink-0 flex-col border-r bg-sidebar">
      <div className="px-4 pt-5 pb-4">
        <div className="text-[15px] font-semibold tracking-tight">TENREC</div>
        <div className="text-xs text-muted-foreground">System monitor{status ? ` · v${status.version}` : ''}</div>
      </div>
      <ul className="flex flex-1 flex-col gap-0.5 px-2">
        {NAV.map(({ id, label, icon: Icon }) => {
          const count = id === 'security' ? activeIncidents : id === 'protection' ? policies.length : 0;
          const active = page === id;
          return (
            <li key={id}>
              <button
                type="button"
                onClick={() => onSelect(id)}
                aria-current={active ? 'page' : undefined}
                className={cn(
                  'flex h-8 w-full items-center gap-2.5 rounded-md px-2.5 text-left transition-colors',
                  active ? 'bg-accent font-medium text-accent-foreground' : 'text-muted-foreground hover:bg-muted hover:text-foreground',
                )}
              >
                <Icon className="size-4" />
                <span className="flex-1">{label}</span>
                {id === 'focus' && focusActive && <span className="size-2 rounded-full bg-primary" aria-label="Focus is on" />}
                {count > 0 && (
                  <span
                    className={cn(
                      'num min-w-5 rounded px-1 text-center text-[11px] font-semibold',
                      id === 'security' ? 'bg-danger/15 text-danger' : 'bg-primary/15 text-primary',
                    )}
                    aria-label={id === 'security' ? `${count} active incidents` : `${count} active changes`}
                  >
                    {count}
                  </span>
                )}
              </button>
            </li>
          );
        })}
      </ul>
      <div className="border-t px-4 py-3 text-xs text-muted-foreground">
        {status ? (
          <div className="flex items-center gap-2">
            <span className={cn('size-2 rounded-full', status.elevated ? 'bg-ok' : 'bg-warn')} aria-hidden />
            {status.elevated ? 'Running as administrator' : 'Standard user (firewall actions off)'}
          </div>
        ) : (
          'Connecting…'
        )}
      </div>
    </nav>
  );
}

function Content({ page }: { page: PageId }) {
  const error = useAppData((s) => s.error);
  const loading = useAppData((s) => s.loading);
  const status = useAppData((s) => s.status);

  if (error) {
    return (
      <div className="p-8">
        <Banner tone="danger" title="No system data">
          {error}
        </Banner>
      </div>
    );
  }
  if (loading) {
    return (
      <div className="flex h-full items-center justify-center text-muted-foreground" role="status">
        Taking the first sample…
      </div>
    );
  }

  return (
    <div className="mx-auto max-w-[1400px] px-6 py-6">
      {(status?.storageError || status?.lastError) && (
        <div className="mb-4 flex flex-col gap-2">
          {status.storageError && <Banner tone="warn" title="History is not being saved">{status.storageError}</Banner>}
          {status.lastError && <Banner tone="warn" title="Sampling problem">{status.lastError}</Banner>}
        </div>
      )}
      {page === 'overview' && <OverviewPage />}
      {page === 'focus' && <FocusPage />}
      {page === 'processes' && <ProcessesPage />}
      {page === 'applications' && <ApplicationsPage />}
      {page === 'network' && <NetworkPage />}
      {page === 'security' && <SecurityPage />}
      {page === 'protection' && <ProtectionPage />}
      {page === 'system' && <SystemPage />}
      {page === 'settings' && <SettingsPage />}
    </div>
  );
}

export default function App() {
  const [page, setPage] = React.useState<PageId>('overview');
  const mainRef = React.useRef<HTMLElement>(null);
  const navigate = React.useCallback((p: PageId) => {
    setPage(p);
    mainRef.current?.scrollTo({ top: 0 });
  }, []);

  return (
    <TooltipProvider delayDuration={300}>
      <ToastProvider>
        <ActionReviewProvider>
          <NavigateContext.Provider value={navigate}>
            <div className="flex h-full">
              <Sidebar page={page} onSelect={navigate} />
              <main ref={mainRef} className="min-w-0 flex-1 overflow-y-auto">
                <Content page={page} />
              </main>
            </div>
          </NavigateContext.Provider>
        </ActionReviewProvider>
      </ToastProvider>
    </TooltipProvider>
  );
}
