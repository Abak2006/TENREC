import * as React from 'react';
import { Crosshair, Square } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Banner, EmptyState, PageHeader, SearchInput, Section, Stat, Tag } from '@/components/common';
import { useToast } from '@/components/toast';
import { api } from '@/lib/api';
import { formatPercent, formatRelative } from '@/lib/format';
import { takePendingFocusApp } from '@/lib/navigation';
import { store, useAppData } from '@/lib/store';
import type { FocusOptions, FocusPreview } from '@/lib/types';
import { cn } from '@/lib/utils';

const OPTIONS_KEY = 'tenrec-focus-options';
const DEFAULT_OPTIONS: FocusOptions = {
  efficiencyMode: true,
  lowerPriority: true,
  lowIo: true,
  lowMemory: false,
  efficiencyCores: false,
  preferGpu: false,
  exclusions: [],
  powerMode: null,
  limitUploadApps: [],
  uploadLimitKbps: 2000,
  blockApps: [],
};

function loadOptions(): FocusOptions {
  try {
    const raw = localStorage.getItem(OPTIONS_KEY);
    return raw ? { ...DEFAULT_OPTIONS, ...JSON.parse(raw) } : DEFAULT_OPTIONS;
  } catch {
    return DEFAULT_OPTIONS;
  }
}

function saveOptions(o: FocusOptions) {
  try {
    localStorage.setItem(OPTIONS_KEY, JSON.stringify(o));
  } catch {
    /* preference only lasts this session */
  }
}

type BoolKey = 'efficiencyMode' | 'lowerPriority' | 'lowIo' | 'lowMemory' | 'efficiencyCores' | 'preferGpu';
const TOGGLES: { key: BoolKey; label: string; detail: string; needsHybrid?: boolean }[] = [
  { key: 'efficiencyMode', label: 'Efficiency mode for background apps', detail: 'The same EcoQoS setting Task Manager uses. Windows runs them at lower clock speeds and schedules them last.' },
  { key: 'lowerPriority', label: 'Lower background priority', detail: 'Normal and above-normal background processes drop to Below normal. Nothing is ever raised except the focus app.' },
  { key: 'lowIo', label: 'Low disk priority for background apps', detail: 'Their file reads and writes wait behind the focus app’s.' },
  { key: 'lowMemory', label: 'Low memory priority for background apps', detail: 'Windows reclaims their memory first when RAM runs short. Can make switching back to them slower.' },
  { key: 'efficiencyCores', label: 'Keep background apps on efficiency cores', detail: 'Hybrid CPUs only. Background apps run on E-cores so P-cores stay free for the focus app.', needsHybrid: true },
  { key: 'preferGpu', label: 'Prefer the high-performance GPU for the focus app', detail: 'Windows graphics setting. Takes effect the next time the app starts.' },
];

function ActiveSession() {
  const focus = useAppData((s) => s.focus)!;
  const [stopping, setStopping] = React.useState(false);
  const toast = useToast();

  const stop = async () => {
    setStopping(true);
    try {
      const r = await api.stopFocus();
      toast({
        tone: r.failed ? 'error' : 'ok',
        title: r.failed ? `Focus ended, ${r.failed} change(s) could not be undone` : 'Focus ended, everything restored',
        detail: r.messages.join('\n') || undefined,
      });
    } catch (e) {
      toast({ tone: 'error', title: 'Could not end focus', detail: (e as Error).message });
    } finally {
      setStopping(false);
      void store.refreshAfterAction();
    }
  };

  const { stats } = focus;
  return (
    <div className="flex flex-col gap-4">
      <Section>
        <div className="flex flex-wrap items-center gap-4">
          <div className="flex size-10 items-center justify-center rounded-lg bg-primary/12 text-primary">
            <Crosshair className="size-5" />
          </div>
          <div className="min-w-0 flex-1">
            <div className="text-xs font-medium text-muted-foreground">Focusing on</div>
            <div className="truncate text-lg font-semibold">{focus.appName}</div>
            <div className="text-xs text-muted-foreground">Started {formatRelative(focus.startedAt)} · new processes are handled as they start</div>
          </div>
          <Button variant="outline" onClick={stop} disabled={stopping}>
            <Square /> {stopping ? 'Ending…' : 'End focus and restore'}
          </Button>
        </div>
      </Section>
      <div className="grid grid-cols-2 gap-3 lg:grid-cols-4">
        <Stat label="Processes adjusted" value={focus.tunedProcesses} sub={focus.failedProcesses ? `${focus.failedProcesses} could not be changed` : 'all changes recorded'} />
        <Stat label="Background CPU before" value={formatPercent(stats.backgroundCpuBefore, 1)} sub="average, last 30 s before start" />
        <Stat label="Background CPU during" value={formatPercent(stats.backgroundCpuDuring, 1)} sub={`average over ${stats.samples} samples`} />
        <Stat label="Focus app CPU" value={formatPercent(stats.focusCpuDuring, 1)} sub="average during focus" />
      </div>
      <p className="text-xs text-muted-foreground">
        Background CPU only counts processes TENREC can measure. Lower background CPU is not by itself proof the focus app got faster; it means less competition for the CPU.
        {focus.gpuPreferenceSet && ' The GPU preference applies the next time the focus app starts.'}
      </p>
    </div>
  );
}

function Setup() {
  const apps = useAppData((s) => s.applications);
  const status = useAppData((s) => s.focus);
  const [appId, setAppId] = React.useState<string | null>(() => takePendingFocusApp());
  const [options, setOptions] = React.useState<FocusOptions>(loadOptions);
  const [query, setQuery] = React.useState('');
  const [exclQuery, setExclQuery] = React.useState('');
  const [preview, setPreview] = React.useState<FocusPreview | null>(null);
  const [previewError, setPreviewError] = React.useState<string | null>(null);
  const [starting, setStarting] = React.useState(false);
  const toast = useToast();

  React.useEffect(() => saveOptions(options), [options]);

  React.useEffect(() => {
    if (!appId) return;
    let cancelled = false;
    const t = setTimeout(() => {
      api.previewFocus(appId, options).then(
        (p) => !cancelled && (setPreview(p), setPreviewError(null)),
        (e: Error) => !cancelled && (setPreview(null), setPreviewError(e.message)),
      );
    }, 250);
    return () => {
      cancelled = true;
      clearTimeout(t);
    };
  }, [appId, options]);

  const candidates = apps.filter((a) => a.executablePath && a.restrictedProcessCount < a.processCount);
  const q = query.trim().toLowerCase();
  const list = candidates
    .filter((a) => !q || a.name.toLowerCase().includes(q) || a.executablePath.toLowerCase().includes(q))
    .sort((a, b) => b.cpuPercent - a.cpuPercent || a.name.localeCompare(b.name));
  const selected = apps.find((a) => a.id === appId) ?? null;
  const eq = exclQuery.trim().toLowerCase();
  const exclList = candidates.filter((a) => a.id !== appId && (!eq || a.name.toLowerCase().includes(eq)));
  const hybrid = status?.hybridCpu ?? false;
  const elevated = useAppData((s) => s.status?.elevated ?? false);

  const toggleExclusion = (id: string) =>
    setOptions((o) => ({ ...o, exclusions: o.exclusions.includes(id) ? o.exclusions.filter((x) => x !== id) : [...o.exclusions, id] }));

  const start = async () => {
    if (!appId) return;
    setStarting(true);
    try {
      const s = await api.startFocus(appId, { ...options, efficiencyCores: options.efficiencyCores && hybrid });
      toast({ tone: 'ok', title: `Focusing on ${s.appName}`, detail: `${s.tunedProcesses} process(es) adjusted.` });
      void store.refreshAfterAction();
    } catch (e) {
      toast({ tone: 'error', title: 'Could not start focus', detail: (e as Error).message });
    } finally {
      setStarting(false);
    }
  };

  return (
    <div className="grid grid-cols-[minmax(0,1fr)] gap-4 xl:grid-cols-[minmax(0,1fr)_minmax(0,1fr)]">
      <Section title="1. Choose the app to focus on" bodyClassName="p-0">
        <div className="border-b p-3">
          <SearchInput value={query} onChange={setQuery} placeholder="Find an app" label="Find an app to focus on" />
        </div>
        <ul className="max-h-96 overflow-y-auto" role="listbox" aria-label="Applications">
          {list.map((a) => (
            <li key={a.id} role="option" aria-selected={a.id === appId}>
              <button
                type="button"
                onClick={() => setAppId(a.id)}
                className={cn('flex w-full items-center gap-3 border-b px-4 py-2 text-left last:border-0', a.id === appId ? 'bg-accent' : 'hover:bg-muted/50')}
              >
                <span className="min-w-0 flex-1">
                  <span className="block truncate font-medium">{a.name}</span>
                  <span className="block truncate text-xs text-muted-foreground">{a.executablePath}</span>
                </span>
                <span className="num text-xs text-muted-foreground">{formatPercent(a.cpuPercent, 1)}</span>
              </button>
            </li>
          ))}
        </ul>
        {list.length === 0 && <EmptyState title="No matching apps" />}
      </Section>

      <div className="flex flex-col gap-4">
        <Section title="2. What to change">
          <div className="flex flex-col gap-3">
            {TOGGLES.map((t) => {
              const disabled = t.needsHybrid && !hybrid;
              return (
                <label key={t.key} className={cn('flex gap-3', disabled ? 'opacity-50' : 'cursor-pointer')}>
                  <input
                    type="checkbox"
                    className="mt-0.5 size-4 shrink-0 accent-[var(--primary)]"
                    checked={!disabled && options[t.key]}
                    disabled={disabled}
                    onChange={(e) => setOptions((o) => ({ ...o, [t.key]: e.target.checked }))}
                  />
                  <span>
                    <span className="font-medium">{t.label}</span>
                    {disabled && <Tag className="ml-2">not available on this CPU</Tag>}
                    <span className="block text-muted-foreground">{t.detail}</span>
                  </span>
                </label>
              );
            })}
            <p className="text-xs text-muted-foreground">The focus app is always set to High priority with efficiency mode off.</p>
          </div>
        </Section>

        <Section title="3. Leave these alone" description="For voice chat, music or anything that must stay responsive">
          <SearchInput value={exclQuery} onChange={setExclQuery} placeholder="Find an app" label="Find an app to exclude" />
          <ul className="mt-2 max-h-40 overflow-y-auto">
            {exclList.slice(0, 100).map((a) => (
              <li key={a.id}>
                <label className="flex cursor-pointer items-center gap-2 py-1">
                  <input type="checkbox" className="size-4 accent-[var(--primary)]" checked={options.exclusions.includes(a.id)} onChange={() => toggleExclusion(a.id)} />
                  <span className="truncate">{a.name}</span>
                </label>
              </li>
            ))}
          </ul>
        </Section>

        <Section title="4. Power and network (optional)" description="Network options need administrator rights. Upload limits only cover outgoing traffic; Windows cannot reserve download speed for one app.">
          <div className="flex flex-col gap-3">
            <label className="flex items-center justify-between gap-3">
              <span>
                <span className="font-medium">Power mode during focus</span>
                <span className="block text-muted-foreground">Restored when focus ends.</span>
              </span>
              <select
                className="h-8 rounded-md border border-input bg-background px-2"
                value={options.powerMode ?? ''}
                onChange={(e) => setOptions((o) => ({ ...o, powerMode: (e.target.value || null) as FocusOptions['powerMode'] }))}
              >
                <option value="">Don't change</option>
                <option value="best_performance">Best performance</option>
                <option value="balanced">Balanced</option>
              </select>
            </label>
            <AppPicker
              label="Block from the network during focus"
              detail="Launchers, cloud sync and updaters you don't need right now."
              apps={exclList}
              selected={options.blockApps}
              onChange={(blockApps) => setOptions((o) => ({ ...o, blockApps }))}
            />
            <AppPicker
              label="Limit upload during focus"
              detail="Stops background uploads from causing lag in games and calls."
              apps={exclList}
              selected={options.limitUploadApps}
              onChange={(limitUploadApps) => setOptions((o) => ({ ...o, limitUploadApps }))}
            />
            {options.limitUploadApps.length > 0 && (
              <label className="flex items-center gap-2">
                Limit each to
                <input
                  type="number"
                  min={64}
                  step={100}
                  className="h-8 w-24 rounded-md border border-input bg-background px-2 num"
                  value={options.uploadLimitKbps ?? 2000}
                  onChange={(e) => setOptions((o) => ({ ...o, uploadLimitKbps: Math.max(64, Number(e.target.value) || 64) }))}
                />
                kbit/s
              </label>
            )}
            {(options.blockApps.length > 0 || options.limitUploadApps.length > 0) && !elevated && (
              <Banner tone="warn" title="Administrator rights needed">Focus will not start with network options unless TENREC runs as administrator.</Banner>
            )}
          </div>
        </Section>

        <Section title="5. Review and start">
          {!selected ? (
            <p className="text-muted-foreground">Choose an app first.</p>
          ) : previewError ? (
            <Banner tone="danger" title="Can't focus on this app">{previewError}</Banner>
          ) : preview && preview.appId === appId ? (
            <div className="flex flex-col gap-3">
              <ul className="flex flex-col gap-1">
                <li><span className="num font-medium">{preview.focusProcessCount}</span> process(es) of {preview.appName} raised to High priority</li>
                <li><span className="num font-medium">{preview.backgroundProcessCount}</span> background process(es) adjusted now, plus any that start later</li>
                <li className="text-muted-foreground">
                  Left alone: {preview.skippedExcluded} excluded, {preview.skippedProtected} core Windows, {preview.skippedRestricted} that need administrator rights
                </li>
              </ul>
              <Button onClick={start} disabled={starting} className="self-start">
                <Crosshair /> {starting ? 'Starting…' : `Start focus on ${preview.appName}`}
              </Button>
              <p className="text-xs text-muted-foreground">Ending focus restores every process it changed. If TENREC closes unexpectedly, the changes are undone the next time it starts.</p>
            </div>
          ) : (
            <p className="text-muted-foreground">Checking…</p>
          )}
        </Section>
      </div>
    </div>
  );
}

function AppPicker({ label, detail, apps, selected, onChange }: {
  label: string;
  detail: string;
  apps: { id: string; name: string }[];
  selected: string[];
  onChange: (ids: string[]) => void;
}) {
  const [open, setOpen] = React.useState(selected.length > 0);
  const names = apps.filter((a) => selected.includes(a.id)).map((a) => a.name);
  return (
    <div>
      <button type="button" className="flex w-full items-center justify-between gap-3 text-left" onClick={() => setOpen((o) => !o)} aria-expanded={open}>
        <span>
          <span className="font-medium">{label}</span>
          <span className="block text-muted-foreground">{names.length ? names.join(', ') : detail}</span>
        </span>
        <span className="text-xs text-primary">{open ? 'Done' : 'Choose'}</span>
      </button>
      {open && (
        <ul className="mt-2 max-h-36 overflow-y-auto rounded-md border p-2">
          {apps.slice(0, 150).map((a) => (
            <li key={a.id}>
              <label className="flex cursor-pointer items-center gap-2 py-0.5">
                <input
                  type="checkbox"
                  className="size-4 accent-[var(--primary)]"
                  checked={selected.includes(a.id)}
                  onChange={() => onChange(selected.includes(a.id) ? selected.filter((x) => x !== a.id) : [...selected, a.id])}
                />
                <span className="truncate">{a.name}</span>
              </label>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

export function FocusPage() {
  const active = useAppData((s) => s.focus?.active ?? false);
  return (
    <>
      <PageHeader
        title="Focus"
        description="Give one app priority over everything else you are running, using Windows scheduling hints that are all undone when you end focus. It reduces competition for CPU and disk; it cannot make an app faster than your hardware."
      />
      {active ? <ActiveSession /> : <Setup />}
    </>
  );
}
