import * as React from 'react';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { DataTable, EmptyState, PageHeader, SearchInput, Sparkline, Stat, Tag, Td, Th } from '@/components/common';
import { endpoint, formatBytes, formatRate, formatRelative } from '@/lib/format';
import { useAppData } from '@/lib/store';

export function NetworkPage() {
  const throughput = useAppData((s) => s.throughput);
  const history = useAppData((s) => s.history);
  const connections = useAppData((s) => s.connections);
  const listening = useAppData((s) => s.listening);
  const destinations = useAppData((s) => s.destinations);
  const [query, setQuery] = React.useState('');
  const [publicOnly, setPublicOnly] = React.useState(true);
  const [exposedOnly, setExposedOnly] = React.useState(false);

  const q = query.trim().toLowerCase();
  const match = (...fields: (string | number)[]) => !q || fields.some((f) => String(f).toLowerCase().includes(q));

  const conns = connections
    .filter((c) => (!publicOnly || c.isPublicRemote) && match(c.processName, c.applicationName, c.remoteAddress, c.remotePort, c.pid))
    .sort((a, b) => Number(b.isNew) - Number(a.isNew) || a.processName.localeCompare(b.processName));
  const ports = listening.filter((l) => (!exposedOnly || l.allInterfaces) && match(l.processName, l.port, l.bindAddress, l.pid));
  const dests = destinations.filter((d) => match(d.destination, ...d.applications));
  const publicCount = connections.filter((c) => c.isPublicRemote).length;

  return (
    <>
      <PageHeader
        title="Network"
        description="Live TCP connections and listening ports with the process that owns them, IPv4 and IPv6. Hostnames are not looked up, so nothing is sent on your behalf."
        actions={<SearchInput value={query} onChange={setQuery} placeholder="Filter by process, address or port" label="Filter network" />}
      />

      <div className="mb-4 grid grid-cols-2 gap-3 lg:grid-cols-4">
        <Stat label="Download" value={formatRate(throughput?.bytesInSec)} sub={`${formatBytes(throughput?.totalBytesIn)} since boot`}>
          <Sparkline values={history.netIn} label="Download rate, last two minutes" colorVar="--chart-2" />
        </Stat>
        <Stat label="Upload" value={formatRate(throughput?.bytesOutSec)} sub={`${formatBytes(throughput?.totalBytesOut)} since boot`}>
          <Sparkline values={history.netOut} label="Upload rate, last two minutes" />
        </Stat>
        <Stat label="Connections" value={connections.length} sub={`${publicCount} to public addresses`} />
        <Stat label="Listening" value={listening.length} sub={`${listening.filter((l) => l.allInterfaces).length} on all interfaces`} />
      </div>

      <Tabs defaultValue="connections">
        <TabsList>
          <TabsTrigger value="connections">Connections</TabsTrigger>
          <TabsTrigger value="listening">Listening ports</TabsTrigger>
          <TabsTrigger value="destinations">Destinations</TabsTrigger>
        </TabsList>

        <TabsContent value="connections" className="mt-3">
          <label className="mb-2 flex items-center gap-2 text-sm">
            <input type="checkbox" className="size-4 accent-[var(--primary)]" checked={publicOnly} onChange={(e) => setPublicOnly(e.target.checked)} />
            Public addresses only (hide local network and loopback)
          </label>
          <DataTable label="Connections" className="max-h-[calc(100vh-22rem)]">
            <thead>
              <tr>
                <Th>Process</Th>
                <Th>Remote</Th>
                <Th className="hidden xl:table-cell">Local</Th>
                <Th>State</Th>
              </tr>
            </thead>
            <tbody>
              {conns.map((c) => (
                <tr key={c.id}>
                  <Td>
                    <div className="flex items-center gap-2">
                      <span className="font-medium">{c.processName}</span>
                      {c.isNew && <Tag tone="accent" title="First connection to this address since TENREC started">new</Tag>}
                    </div>
                    <div className="text-xs text-muted-foreground">PID {c.pid} · {c.applicationName}</div>
                  </Td>
                  <Td mono>{endpoint(c.remoteAddress, c.remotePort)}</Td>
                  <Td mono className="hidden text-muted-foreground xl:table-cell">{endpoint(c.localAddress, c.localPort)}</Td>
                  <Td><Tag>{c.state.toLowerCase().replace('_', ' ')}</Tag></Td>
                </tr>
              ))}
            </tbody>
          </DataTable>
          {conns.length === 0 && <EmptyState title="No connections match" />}
        </TabsContent>

        <TabsContent value="listening" className="mt-3">
          <label className="mb-2 flex items-center gap-2 text-sm">
            <input type="checkbox" className="size-4 accent-[var(--primary)]" checked={exposedOnly} onChange={(e) => setExposedOnly(e.target.checked)} />
            Only ports reachable from other machines (bound to all interfaces)
          </label>
          <DataTable label="Listening ports" className="max-h-[calc(100vh-22rem)]">
            <thead>
              <tr>
                <Th align="right">Port</Th>
                <Th>Protocol</Th>
                <Th>Bound to</Th>
                <Th>Process</Th>
              </tr>
            </thead>
            <tbody>
              {ports.map((l) => (
                <tr key={`${l.protocol}-${l.bindAddress}-${l.port}-${l.pid}`}>
                  <Td align="right" className="font-medium">{l.port}</Td>
                  <Td>{l.protocol} · IPv{l.ipVersion}</Td>
                  <Td mono>
                    {l.bindAddress} {l.allInterfaces && <Tag tone="warn" title="Reachable from other machines unless the firewall blocks it">all interfaces</Tag>}
                  </Td>
                  <Td>
                    <span className="font-medium">{l.processName}</span>
                    <span className="text-xs text-muted-foreground"> · PID {l.pid}</span>
                  </Td>
                </tr>
              ))}
            </tbody>
          </DataTable>
          {ports.length === 0 && <EmptyState title="No listening ports match" />}
        </TabsContent>

        <TabsContent value="destinations" className="mt-3">
          <p className="mb-2 text-sm text-muted-foreground">Public addresses contacted since TENREC started, most active first.</p>
          <DataTable label="Destinations" className="max-h-[calc(100vh-22rem)]">
            <thead>
              <tr>
                <Th>Address</Th>
                <Th>Applications</Th>
                <Th align="right">Open now</Th>
                <Th align="right">First seen</Th>
                <Th align="right">Last seen</Th>
              </tr>
            </thead>
            <tbody>
              {dests.map((d) => (
                <tr key={d.destination}>
                  <Td mono>{d.destination}</Td>
                  <Td className="max-w-96 truncate">{d.applications.join(', ')}</Td>
                  <Td align="right">{d.openConnections || '–'}</Td>
                  <Td align="right" className="text-muted-foreground">{formatRelative(d.firstSeen)}</Td>
                  <Td align="right" className="text-muted-foreground">{d.openConnections ? 'now' : formatRelative(d.lastSeen)}</Td>
                </tr>
              ))}
            </tbody>
          </DataTable>
          {dests.length === 0 && <EmptyState title="No public destinations yet" />}
        </TabsContent>
      </Tabs>
    </>
  );
}
