import * as React from 'react';
import { RefreshCw } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Switch } from '@/components/ui/switch';
import { Banner, DataTable, EmptyState, PageHeader, Section, Tag, Td, Th } from '@/components/common';
import { useToast } from '@/components/toast';
import { api } from '@/lib/api';
import { formatRelative } from '@/lib/format';
import { store, useAppData } from '@/lib/store';
import type { PrivacyEvidence, PrivacySetting } from '@/lib/types';

function Evidence() {
  const [evidence, setEvidence] = React.useState<PrivacyEvidence | null>(null);
  const [error, setError] = React.useState<string | null>(null);

  React.useEffect(() => {
    const load = () => api.privacyEvidence().then(setEvidence, (e: Error) => setError(e.message));
    load();
    const t = setInterval(load, 10000);
    return () => clearInterval(t);
  }, []);

  return (
    <Section
      title="What this PC actually sent"
      description={evidence ? `Connections to Microsoft diagnostic endpoints since TENREC started (${formatRelative(evidence.since)})` : undefined}
      bodyClassName="p-0"
    >
      {error && <div className="p-4"><Banner tone="danger" title="Could not read evidence">{error}</Banner></div>}
      {evidence && evidence.hostnamesKnown === 0 && (
        <div className="p-4">
          <Banner tone="warn" title="No hostnames available on this PC">
            TENREC matches connections to hostnames using the Windows DNS cache, and it is empty here. A VPN or secure-DNS client
            (for example Cloudflare WARP or Tailscale) can bypass it. The settings below still work; only this evidence view is unavailable.
          </Banner>
        </div>
      )}
      {evidence && evidence.hostnamesKnown > 0 && evidence.observations.length === 0 && (
        <EmptyState title="No diagnostic connections seen yet">
          Windows sends diagnostic data in batches, so this can take a while. Required diagnostic data is still sent even with every setting below turned off.
        </EmptyState>
      )}
      {evidence && evidence.observations.length > 0 && (
        <DataTable label="Diagnostic connections" className="rounded-none border-0">
          <thead>
            <tr>
              <Th>Endpoint</Th>
              <Th>Process</Th>
              <Th align="right">Seen</Th>
              <Th align="right">Last</Th>
            </tr>
          </thead>
          <tbody>
            {evidence.observations.map((o) => (
              <tr key={o.hostname}>
                <Td mono>{o.hostname}</Td>
                <Td>{o.processes.join(', ')}</Td>
                <Td align="right">{o.samplesSeen} × 2 s</Td>
                <Td align="right" className="text-muted-foreground">{formatRelative(o.lastSeen)}</Td>
              </tr>
            ))}
          </tbody>
        </DataTable>
      )}
    </Section>
  );
}

export function PrivacyPage() {
  const [settings, setSettings] = React.useState<PrivacySetting[] | null>(null);
  const [error, setError] = React.useState<string | null>(null);
  const [busy, setBusy] = React.useState<string | null>(null);
  const elevated = useAppData((s) => s.status?.elevated ?? false);
  const toast = useToast();

  const load = React.useCallback(() => {
    api.privacySettings().then(
      (s) => {
        setSettings(s);
        setError(null);
      },
      (e: Error) => setError(e.message),
    );
  }, []);
  React.useEffect(load, [load]);

  const toggle = async (s: PrivacySetting, makePrivate: boolean) => {
    setBusy(s.id);
    try {
      const r = await api.setPrivacySetting(s.id, makePrivate);
      toast({ tone: 'ok', title: r.description, detail: 'Undo it from the Protection page.' });
    } catch (e) {
      toast({ tone: 'error', title: `Could not change ${s.title}`, detail: (e as Error).message });
    } finally {
      setBusy(null);
      load();
      void store.refreshSlow();
    }
  };

  const privateCount = settings?.filter((s) => s.isPrivate).length ?? 0;

  return (
    <>
      <PageHeader
        title="Privacy"
        description="Windows privacy settings in one place. Each switch writes the same value the Settings app or a documented Group Policy writes, and every change can be undone."
        actions={<Button size="sm" variant="ghost" onClick={load} aria-label="Reload settings"><RefreshCw /></Button>}
      />
      <div className="flex flex-col gap-4">
        <Section
          title="Settings"
          description={settings ? `${privateCount} of ${settings.length} set to the private option` : undefined}
          bodyClassName="p-0"
        >
          {error && <div className="p-4"><Banner tone="danger" title="Could not read settings">{error}</Banner></div>}
          {!settings && !error && <p className="p-4 text-muted-foreground">Reading settings…</p>}
          {settings && (
            <ul className="divide-y">
              {settings.map((s) => {
                const locked = s.requiresAdmin && !elevated;
                return (
                  <li key={s.id} className="flex items-start gap-4 px-4 py-3">
                    <div className="min-w-0 flex-1">
                      <div className="flex flex-wrap items-center gap-2">
                        <span className="font-medium">{s.title}</span>
                        {s.source === 'group_policy' && <Tag title="Written as a Group Policy value">policy</Tag>}
                        {s.requiresAdmin && <Tag>needs admin</Tag>}
                      </div>
                      <p className="text-muted-foreground">{s.description}</p>
                      {s.note && <p className="text-xs text-muted-foreground">{s.note}</p>}
                      <p className="mt-0.5 truncate font-mono text-[11px] text-muted-foreground" title={s.location}>{s.location}</p>
                    </div>
                    <div className="flex shrink-0 items-center gap-2 pt-0.5">
                      <span className="text-xs text-muted-foreground">{s.isPrivate ? 'Off' : 'On'}</span>
                      <Switch
                        checked={!s.isPrivate}
                        disabled={busy !== null || locked}
                        title={locked ? 'Needs administrator rights' : undefined}
                        aria-label={`${s.title}: ${s.isPrivate ? 'off' : 'on'}`}
                        onCheckedChange={(on) => toggle(s, !on)}
                      />
                    </div>
                  </li>
                );
              })}
            </ul>
          )}
        </Section>
        <Evidence />
      </div>
    </>
  );
}
