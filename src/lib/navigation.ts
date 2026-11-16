import { createContext, useContext } from 'react';

export type PageId = 'overview' | 'focus' | 'startup' | 'processes' | 'applications' | 'network' | 'security' | 'protection' | 'system' | 'settings';

export const NavigateContext = createContext<(p: PageId) => void>(() => {});
export const useNavigate = () => useContext(NavigateContext);

/** Application chosen elsewhere (e.g. the Applications page) to preselect on the Focus page. */
let pendingFocusApp: string | null = null;
export const setPendingFocusApp = (id: string | null) => {
  pendingFocusApp = id;
};
export const takePendingFocusApp = () => {
  const id = pendingFocusApp;
  pendingFocusApp = null;
  return id;
};
