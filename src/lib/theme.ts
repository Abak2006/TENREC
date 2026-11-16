import { useEffect, useState } from 'react';

export type ThemePref = 'system' | 'light' | 'dark';
const KEY = 'tenrec-theme';

function read(): ThemePref {
  try {
    const v = localStorage.getItem(KEY);
    return v === 'light' || v === 'dark' ? v : 'system';
  } catch {
    return 'system';
  }
}

function apply(pref: ThemePref) {
  const dark = pref === 'dark' || (pref === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches);
  document.documentElement.classList.toggle('dark', dark);
  document.documentElement.style.colorScheme = dark ? 'dark' : 'light';
}

export function initTheme() {
  apply(read());
  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => apply(read()));
}

export function useTheme(): [ThemePref, (p: ThemePref) => void] {
  const [pref, setPref] = useState<ThemePref>(read);
  useEffect(() => {
    try {
      localStorage.setItem(KEY, pref);
    } catch {
      /* storage unavailable: preference lasts for this session only */
    }
    apply(pref);
  }, [pref]);
  return [pref, setPref];
}
