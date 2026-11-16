import { DataTable, Meter, PageHeader, Section, Tag, Td, Th } from '@/components/common';
import { formatBytes, formatDuration, formatPercent } from '@/lib/format';
import { useAppData } from '@/lib/store';
import type { HardwareCapabilityItem } from '@/lib/types';

const STATE: Record<HardwareCapabilityItem['state'], { label: string; tone: 'ok' | 'warn' | 'neutral' }> = {
  AVAILABLE: { label: 'available', tone: 'ok' },
  REQUIRES_ADMIN: { label: 'needs admin', tone: 'warn' },
  NOT_PRESENT: { label: 'not present', tone: 'neutral' },
  NOT_IMPLEMENTED: { label: 'not implemented', tone: 'neutral' },
};

function Row({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <>
      <dt className="text-muted-foreground">{label}</dt>
      <dd className="min-w-0 break-words">{children}</dd>
    </>
  );
}

export function SystemPage() {
  const snapshot = useAppData((s) => s.snapshot);
  const diagnostics = useAppData((s) => s.diagnostics);
  const capabilities = useAppData((s) => s.capabilities);
  if (!snapshot) return null;
  const { cpu, memory, power } = snapshot;

  return (
    <>
      <PageHeader title="System" description="Hardware and operating system details read from Windows." />

      <div className="grid gap-4 lg:grid-cols-2">
        <Section title="This PC">
          <dl className="grid grid-cols-[max-content_minmax(0,1fr)] gap-x-6 gap-y-1.5">
            <Row label="Name">{snapshot.hostname || '–'}</Row>
            <Row label="Model">{diagnostics ? `${diagnostics.manufacturer} ${diagnostics.model}` : '–'}</Row>
            <Row label="Windows">{snapshot.osName || '–'}</Row>
            <Row label="Uptime">{formatDuration(snapshot.uptimeSeconds)}</Row>
            <Row label="Processor">{diagnostics?.cpuName ?? '–'}</Row>
            <Row label="Cores">{cpu.coreCount} physical · {cpu.logicalCoreCount} logical{cpu.frequencyMhz ? ` · ${cpu.frequencyMhz} MHz` : ''}</Row>
            <Row label="Memory">{formatBytes(memory.totalBytes)}</Row>
            <Row label="Graphics">
              {snapshot.gpu.name}
              {diagnostics?.gpuDriverVersion && <span className="text-muted-foreground"> · driver {diagnostics.gpuDriverVersion}</span>}
            </Row>
            <Row label="Power">
              {power.hasBattery
                ? `${power.batteryPercent ?? '–'}% · ${power.batteryStatus}`
                : power.acConnected ? 'Mains power (no battery)' : power.batteryStatus}
            </Row>
          </dl>
        </Section>

        <Section title="CPU cores" description={`${formatPercent(cpu.usagePercent)} overall`}>
          <div className="grid grid-cols-2 gap-x-6 gap-y-2 sm:grid-cols-4">
            {cpu.perCoreUsage.map((u, i) => (
              <div key={i} className="flex flex-col gap-1">
                <div className="num flex justify-between gap-2 text-xs">
                  <span className="text-muted-foreground">Core {i}</span>
                  <span>{formatPercent(u)}</span>
                </div>
                <Meter value={u} label={`Core ${i} usage`} />
              </div>
            ))}
          </div>
        </Section>

        <Section title="Drives" description="Fixed and removable drives" className="lg:col-span-2" bodyClassName="p-0">
          <DataTable label="Drives" className="rounded-none border-0">
            <thead>
              <tr>
                <Th>Drive</Th>
                <Th>File system</Th>
                <Th className="w-1/3">Used</Th>
                <Th align="right">Free</Th>
                <Th align="right">Size</Th>
              </tr>
            </thead>
            <tbody>
              {snapshot.disk.drives.map((d) => (
                <tr key={d.mountPoint}>
                  <Td>
                    <span className="font-medium">{d.mountPoint}</span>
                    {d.volumeName && <span className="text-muted-foreground"> {d.volumeName}</span>}
                  </Td>
                  <Td>{d.fileSystem || '–'}</Td>
                  <Td>
                    <div className="flex items-center gap-2">
                      <Meter value={d.usagePercent} label={`${d.mountPoint} used`} />
                      <span className="num w-10 text-right text-xs">{formatPercent(d.usagePercent)}</span>
                    </div>
                  </Td>
                  <Td align="right">{formatBytes(d.freeBytes)}</Td>
                  <Td align="right">{formatBytes(d.totalBytes)}</Td>
                </tr>
              ))}
            </tbody>
          </DataTable>
        </Section>

        <Section title="What TENREC can see and do" description="Read straight from the code paths that run on this machine" className="lg:col-span-2" bodyClassName="p-0">
          <DataTable label="Capabilities" className="rounded-none border-0">
            <thead>
              <tr>
                <Th>Capability</Th>
                <Th>Status</Th>
                <Th>Access</Th>
                <Th>Details</Th>
              </tr>
            </thead>
            <tbody>
              {capabilities.map((c) => (
                <tr key={c.id}>
                  <Td className="font-medium">{c.name}</Td>
                  <Td><Tag tone={STATE[c.state].tone}>{STATE[c.state].label}</Tag></Td>
                  <Td className="text-muted-foreground">{c.accessMode === 'READ_ONLY' ? 'read' : 'change'}</Td>
                  <Td className="text-muted-foreground">
                    {c.description}
                    <div className="font-mono text-[11px]">{c.provider}</div>
                  </Td>
                </tr>
              ))}
            </tbody>
          </DataTable>
        </Section>
      </div>
    </>
  );
}
