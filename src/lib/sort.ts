import * as React from 'react';

export type SortDir = 'asc' | 'desc';
export interface SortState<K extends string> {
  key: K;
  dir: SortDir;
}

export function useSort<K extends string>(initial: SortState<K>) {
  const [sort, setSort] = React.useState(initial);
  const toggle = (key: K) =>
    setSort((s) => (s.key === key ? { key, dir: s.dir === 'asc' ? 'desc' : 'asc' } : { key, dir: 'desc' }));
  return { sort, toggle };
}

export function compare(a: unknown, b: unknown): number {
  if (typeof a === 'number' && typeof b === 'number') return a - b;
  return String(a ?? '').localeCompare(String(b ?? ''), undefined, { sensitivity: 'base', numeric: true });
}

export function sortBy<T, K extends string>(rows: T[], sort: SortState<K>, get: (row: T, key: K) => unknown): T[] {
  const sign = sort.dir === 'asc' ? 1 : -1;
  return [...rows].sort((x, y) => sign * compare(get(x, sort.key), get(y, sort.key)));
}
