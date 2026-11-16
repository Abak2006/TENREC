import * as React from 'react';
import { RefreshCw } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Switch } from '@/components/ui/switch';
import { Banner, DataTable, EmptyState, PageHeader, SearchInput, Section, Tag, Td, Th } from '@/components/common';
import { useToast } from '@/components/toast';
import { api } from '@/lib/api';
import { store, useAppData } from '@/lib/store';
import type { ScheduledTaskItem, SettingChangeResult, StartupItem } from '@/lib/types';

type Load<T> = { state: 'loading' } | { state: 'error'; message: string } | { state: 'ready'; data: T };

function useLoad<T>(fn: () => Promise<T>) {
  const [value, setValue] = React.useState<Load<T>>({ state: 'loading' });
  const reload = React.useCallback(() => {
    fn().then(
      (data) => setValue({ state: 'ready', data }),
      (e: Error) => setValue({ state: 'error', message: e.message }),
    );
  }, [fn]);
  React.useEffect(reload, [reload]);
  return [value, reload] as const;
}

function useToggle(reload: () => void) {
  const toast = useToast();
  const [busy, setBusy] = React.useState<string | null>(null);
  const run = async (key: string, fn: () => Promise<SettingChangeResult>) => {
    setBusy(key);
    try {
      const r = await fn();
      toast({ tone: 'ok', title: r.description, detail: 'Undo it from the Protection page.' });
    } catch (e) {
      toast({ tone: 'error', title: 'Could not change it', detail: (e as Error).message });
    } finally {
      setBusy(null);
      reload();
      void store.refreshSlow();
    }
  };
  return { busy, run };
}

const loadStartup = () => api.startupItems();
const loadTasks = () => api.scheduledTasks();

function StartupApps({ query }: { query: string }) {
  const [items, reload] = useLoad(loadStartup);
  const elevated = useAppData((s) => s.status?.elevated ?? false);
  const { busy, run } = useToggle(reload);
  const q = query.toLowerCase();

  return (
    <Section
      title="Startup apps"
      description="Programs Windows starts when you sign in. Turning one off uses the same switch as Task Manager; the program itself is not changed."
      actions={<Button size="sm" variant="ghost" onClick={reload} aria-label="Reload startup apps"><RefreshCw /></Button>}
      bodyClassName="p-0"
    >
      {items.state === 'loading' && <p className="p-4 text-muted-foreground">Reading startup entries…</p>}
      {items.state === 'error' && <div className="p-4"><Banner tone="danger" title="Could not read startup entries">{items.message}</Banner></div>}
      {items.state === 'ready' && (
        <DataTable label="Startup apps" className="rounded-none border-0">
          <thead>
            <tr>
              <Th>App</Th>
              <Th>Where</Th>
              <Th>Status</Th>
              <Th align="right">Starts with Windows</Th>
            </tr>
          </thead>
          <tbody>
            {items.data
              .filter((i: StartupItem) => !q || i.name.toLowerCase().includes(q) || i.command.toLowerCase().includes(q))
              .map((i: StartupItem) => {
                const locked = i.requiresAdmin && !elevated;
                return (
                  <tr key={i.id}>
                    <Td className="max-w-96">
                      <div className="truncate font-medium">{i.name}</div>
                      <div className="truncate text-xs text-muted-foreground" title={i.command}>{[i.company, i.command].filter(Boolean).join(' · ')}</div>
                    </Td>
                    <Td className="whitespace-nowrap text-muted-foreground">{i.sourceLabel}</Td>
                    <Td>{i.runningNow ? <Tag tone="accent">running now</Tag> : <Tag>not running</Tag>}</Td>
                    <Td align="right">
                      <Switch
                        checked={i.enabled}
                        disabled={busy !== null || locked}
                        title={locked ? 'All-users entries need administrator rights' : undefined}
                        aria-label={`${i.enabled ? 'Disable' : 'Enable'} ${i.name} at startup`}
                        onCheckedChange={(v) => run(i.id, () => api.setStartupEnabled(i.id, v))}
                      />
                    </Td>
                  </tr>
                );
              })}
          </tbody>
        </DataTable>
      )}
      {items.state === 'ready' && items.data.length === 0 && <EmptyState title="No startup entries found" />}
    </Section>
  );
}

function Tasks({ query }: { query: string }) {
  const [tasks, reload] = useLoad(loadTasks);
  const { busy, run } = useToggle(reload);
  const [logonOnly, setLogonOnly] = React.useState(false);
  const q = query.toLowerCase();

  return (
    <Section
      title="Scheduled tasks"
      description="Tasks installed by other apps (Windows' own tasks under \Microsoft\ are not shown or changed). Updaters and launchers often live here."
      actions={
        <div className="flex items-center gap-3">
          <label className="flex items-center gap-2 text-sm">
            <input type="checkbox" className="size-4 accent-[var(--primary)]" checked={logonOnly} onChange={(e) => setLogonOnly(e.target.checked)} />
            At sign-in or startup only
          </label>
          <Button size="sm" variant="ghost" onClick={reload} aria-label="Reload scheduled tasks"><RefreshCw /></Button>
        </div>
      }
      bodyClassName="p-0"
    >
      {tasks.state === 'loading' && <p className="p-4 text-muted-foreground">Reading scheduled tasks…</p>}
      {tasks.state === 'error' && <div className="p-4"><Banner tone="danger" title="Could not read scheduled tasks">{tasks.message}</Banner></div>}
      {tasks.state === 'ready' && (
        <DataTable label="Scheduled tasks" className="rounded-none border-0">
          <thead>
            <tr>
              <Th>Task</Th>
              <Th>Runs</Th>
              <Th align="right">Enabled</Th>
            </tr>
          </thead>
          <tbody>
            {tasks.data
              .filter((t: ScheduledTaskItem) => (!logonOnly || t.runsAtLogonOrBoot) && (!q || t.path.toLowerCase().includes(q) || t.actions.join(' ').toLowerCase().includes(q)))
              .map((t: ScheduledTaskItem) => (
                <tr key={t.path}>
                  <Td className="max-w-[28rem]">
                    <div className="truncate font-medium" title={t.path}>{t.name}</div>
                    <div className="truncate text-xs text-muted-foreground" title={t.actions.join('\n')}>
                      {[t.author, t.actions[0]].filter(Boolean).join(' · ') || t.path}
                    </div>
                  </Td>
                  <Td className="text-muted-foreground">{t.triggers.length ? [...new Set(t.triggers)].join(', ') : 'On demand'}</Td>
                  <Td align="right">
                    <Switch
                      checked={t.enabled}
                      disabled={busy !== null}
                      aria-label={`${t.enabled ? 'Disable' : 'Enable'} task ${t.name}`}
                      onCheckedChange={(v) => run(t.path, () => api.setTaskEnabled(t.path, v))}
                    />
                  </Td>
                </tr>
              ))}
          </tbody>
        </DataTable>
      )}
      {tasks.state === 'ready' && tasks.data.length === 0 && <EmptyState title="No third-party scheduled tasks" />}
    </Section>
  );
}

export function StartupPage() {
  const [query, setQuery] = React.useState('');
  return (
    <>
      <PageHeader
        title="Startup"
        description="What runs when Windows starts or you sign in. Every change is recorded and can be undone from the Protection page."
        actions={<SearchInput value={query} onChange={setQuery} placeholder="Filter by name or command" label="Filter startup items" />}
      />
      <div className="flex flex-col gap-4">
        <StartupApps query={query} />
        <Tasks query={query} />
      </div>
    </>
  );
}
