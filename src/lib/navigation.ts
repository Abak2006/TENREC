import { createContext, useContext } from 'react';

export type PageId = 'overview' | 'processes' | 'applications' | 'network' | 'security' | 'protection' | 'system' | 'settings';

export const NavigateContext = createContext<(p: PageId) => void>(() => {});
export const useNavigate = () => useContext(NavigateContext);
