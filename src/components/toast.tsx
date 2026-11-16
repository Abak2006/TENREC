import * as React from 'react';
import { CheckCircle2, X, XCircle } from 'lucide-react';
import { cn } from '@/lib/utils';

interface Toast {
  id: number;
  tone: 'ok' | 'error';
  title: string;
  detail?: string;
}

const ToastContext = React.createContext<(t: Omit<Toast, 'id'>) => void>(() => {});

export function useToast() {
  return React.useContext(ToastContext);
}

export function ToastProvider({ children }: { children: React.ReactNode }) {
  const [toasts, setToasts] = React.useState<Toast[]>([]);
  const nextId = React.useRef(1);

  const dismiss = React.useCallback((id: number) => setToasts((ts) => ts.filter((t) => t.id !== id)), []);
  const show = React.useCallback(
    (t: Omit<Toast, 'id'>) => {
      const id = nextId.current++;
      setToasts((ts) => [...ts.slice(-3), { ...t, id }]);
      // Errors stay until dismissed so they can be read.
      if (t.tone === 'ok') setTimeout(() => dismiss(id), 5000);
    },
    [dismiss],
  );

  return (
    <ToastContext.Provider value={show}>
      {children}
      <div aria-live="polite" className="pointer-events-none fixed right-4 bottom-4 z-[60] flex w-96 max-w-[calc(100vw-2rem)] flex-col gap-2">
        {toasts.map((t) => (
          <div
            key={t.id}
            role={t.tone === 'error' ? 'alert' : 'status'}
            className="pointer-events-auto flex items-start gap-3 rounded-lg border bg-popover p-3 text-popover-foreground shadow-lg"
          >
            {t.tone === 'ok' ? (
              <CheckCircle2 className="mt-0.5 size-4 shrink-0 text-ok" aria-hidden />
            ) : (
              <XCircle className="mt-0.5 size-4 shrink-0 text-danger" aria-hidden />
            )}
            <div className="min-w-0 flex-1">
              <div className="font-medium">{t.title}</div>
              {t.detail && <div className={cn('mt-0.5 whitespace-pre-line text-muted-foreground')}>{t.detail}</div>}
            </div>
            <button type="button" onClick={() => dismiss(t.id)} className="rounded p-0.5 text-muted-foreground hover:text-foreground" aria-label="Dismiss notification">
              <X className="size-4" />
            </button>
          </div>
        ))}
      </div>
    </ToastContext.Provider>
  );
}
