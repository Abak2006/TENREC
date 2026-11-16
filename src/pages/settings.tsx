import * as React from 'react';
import { Button } from '@/components/ui/button';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { PageHeader, Section } from '@/components/common';
import { useToast } from '@/components/toast';
import { api } from '@/lib/api';
import { formatBytes } from '@/lib/format';
import { useAppData } from '@/lib/store';
import { useTheme, type ThemePref } from '@/lib/theme';
import type { DatabaseStats } from '@/lib/types';

export function SettingsPage() {
  const [theme, setTheme] = useTheme();
  const status = useAppData((s) => s.status);
  const [stats, setStats] = React.useState<DatabaseStats | null>(null);
  const [statsError, setStatsError] = React.useState<string | null>(null);
  const [retention, setRetention] = React.useState('168');
  const [purging, setPurging] = React.useState(false);
  const toast = useToast();

  const loadStats = React.useCallback(() => {
    api.databaseStats().then(
      (s) => {
        setStats(s);
        setStatsError(null);
      },
      (e: Error) => setStatsError(e.message),
    );
  }, []);
  React.useEffect(loadStats, [loadStats]);

  const purge = async () => {
    setPurging(true);
    try {
      const n = await api.purgeTelemetry(Number(retention));
      toast({ tone: 'ok', title: `Deleted ${n} stored sample${n === 1 ? '' : 's'}` });
      loadStats();
    } catch (e) {
      toast({ tone: 'error', title: 'Could not delete history', detail: (e as Error).message });
    } finally {
      setPurging(false);
    }
  };

  return (
    <>
      <PageHeader title="Settings" />
      <div className="flex max-w-3xl flex-col gap-4">
        <Section title="Appearance">
          <div className="flex items-center justify-between gap-4">
            <div>
              <div className="font-medium">Theme</div>
              <div className="text-muted-foreground">System follows your Windows light/dark setting.</div>
            </div>
            <Select value={theme} onValueChange={(v) => setTheme(v as ThemePref)}>
              <SelectTrigger className="w-36" aria-label="Theme">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="system">System</SelectItem>
                <SelectItem value="light">Light</SelectItem>
                <SelectItem value="dark">Dark</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </Section>

        <Section title="Stored data" description="Everything stays on this PC. TENREC makes no network requests of its own.">
          {statsError ? (
            <p className="text-danger">{statsError}</p>
          ) : stats ? (
            <dl className="grid grid-cols-[max-content_minmax(0,1fr)] gap-x-6 gap-y-1.5">
              <dt className="text-muted-foreground">Location</dt>
              <dd className="break-all font-mono text-xs leading-5">{stats.dbPath}</dd>
              <dt className="text-muted-foreground">Size</dt>
              <dd className="num">{formatBytes(stats.fileSizeBytes)}</dd>
              <dt className="text-muted-foreground">Contents</dt>
              <dd className="num">
                {stats.sampleCount} system samples (one every 30 s) · {stats.snapshotCount} undo records · {stats.auditCount} audit entries
              </dd>
            </dl>
          ) : (
            <p className="text-muted-foreground">Loading…</p>
          )}
          <div className="mt-4 flex flex-wrap items-center gap-2 border-t pt-4">
            <span>Delete system samples older than</span>
            <Select value={retention} onValueChange={setRetention}>
              <SelectTrigger className="w-28" aria-label="Retention period">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="1">1 hour</SelectItem>
                <SelectItem value="24">1 day</SelectItem>
                <SelectItem value="168">7 days</SelectItem>
              </SelectContent>
            </Select>
            <Button variant="outline" onClick={purge} disabled={purging}>
              {purging ? 'Deleting…' : 'Delete'}
            </Button>
          </div>
          <p className="mt-2 text-xs text-muted-foreground">Samples older than 7 days are deleted automatically at startup. Undo records and the audit log are kept.</p>
        </Section>

        <Section title="About">
          <dl className="grid grid-cols-[max-content_minmax(0,1fr)] gap-x-6 gap-y-1.5">
            <dt className="text-muted-foreground">Version</dt>
            <dd>{status?.version ?? '–'}</dd>
            <dt className="text-muted-foreground">Privileges</dt>
            <dd>{status?.elevated ? 'Administrator' : 'Standard user. Restart with “Run as administrator” to enable firewall actions.'}</dd>
          </dl>
        </Section>
      </div>
    </>
  );
}
