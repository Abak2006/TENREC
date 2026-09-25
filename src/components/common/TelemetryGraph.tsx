import React, { useRef, useEffect } from 'react';

export interface DataSeries {
  name: string;
  color: string;
  data: number[]; // real values in chronological order
}

interface TelemetryGraphProps {
  series: DataSeries[];
  height?: number;
  unit?: string;
  formatValue?: (val: number) => string;
  minPoints?: number;
  emptyMessage?: string;
}

export const TelemetryGraph: React.FC<TelemetryGraphProps> = ({
  series,
  height = 140,
  unit = '',
  formatValue = (v) => v.toFixed(1),
  minPoints = 2,
  emptyMessage = 'Awaiting real telemetry samples...',
}) => {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);

  // Check if we have sufficient real data points
  const maxDataLength = Math.max(...series.map((s) => s.data.length), 0);
  const hasData = maxDataLength >= minPoints;

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas || !hasData) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    // Handle high DPI displays
    const dpr = window.devicePixelRatio || 1;
    const rect = canvas.getBoundingClientRect();
    canvas.width = rect.width * dpr;
    canvas.height = rect.height * dpr;
    ctx.scale(dpr, dpr);

    const width = rect.width;
    const chartHeight = rect.height;
    const padding = { top: 12, right: 14, bottom: 22, left: 45 };
    const graphWidth = width - padding.left - padding.right;
    const graphHeight = chartHeight - padding.top - padding.bottom;

    ctx.clearRect(0, 0, width, chartHeight);

    // Compute global min/max across all series
    let maxVal = 0;
    series.forEach((s) => {
      s.data.forEach((val) => {
        if (val > maxVal) maxVal = val;
      });
    });
    // Add 15% headroom or fallback
    maxVal = maxVal > 0 ? maxVal * 1.15 : 100;

    // Draw horizontal grid lines
    ctx.strokeStyle = 'rgba(37, 51, 78, 0.4)';
    ctx.lineWidth = 1;
    ctx.font = '10px JetBrains Mono';
    ctx.fillStyle = '#64748b';
    ctx.textAlign = 'right';

    const gridLines = 4;
    for (let i = 0; i <= gridLines; i++) {
      const y = padding.top + (graphHeight / gridLines) * i;
      ctx.beginPath();
      ctx.moveTo(padding.left, y);
      ctx.lineTo(width - padding.right, y);
      ctx.stroke();

      const labelVal = maxVal - (maxVal / gridLines) * i;
      ctx.fillText(`${formatValue(labelVal)}${unit}`, padding.left - 6, y + 3);
    }

    // Draw each series
    series.forEach((s) => {
      if (s.data.length < 2) return;

      const stepX = graphWidth / (s.data.length - 1);

      // Line path
      ctx.beginPath();
      s.data.forEach((val, index) => {
        const x = padding.left + index * stepX;
        const normY = (val / maxVal);
        const y = padding.top + graphHeight - normY * graphHeight;
        if (index === 0) {
          ctx.moveTo(x, y);
        } else {
          ctx.lineTo(x, y);
        }
      });

      ctx.strokeStyle = s.color;
      ctx.lineWidth = 1.75;
      ctx.stroke();

      // Subtle gradient fill under curve
      const gradient = ctx.createLinearGradient(0, padding.top, 0, padding.top + graphHeight);
      gradient.addColorStop(0, `${s.color}22`);
      gradient.addColorStop(1, `${s.color}00`);

      ctx.lineTo(padding.left + (s.data.length - 1) * stepX, padding.top + graphHeight);
      ctx.lineTo(padding.left, padding.top + graphHeight);
      ctx.closePath();
      ctx.fillStyle = gradient;
      ctx.fill();
    });
  }, [series, hasData, height, unit, formatValue]);

  if (!hasData) {
    return (
      <div
        style={{
          height,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          background: 'rgba(15, 20, 32, 0.3)',
          borderRadius: 'var(--radius-md)',
          border: '1px dashed var(--border-subtle)',
          color: 'var(--text-muted)',
          fontSize: '11.5px',
          fontFamily: 'var(--font-mono)',
        }}
      >
        {emptyMessage}
      </div>
    );
  }

  return (
    <div>
      <canvas
        ref={canvasRef}
        style={{
          width: '100%',
          height: `${height}px`,
          display: 'block',
        }}
      />
      <div style={{ display: 'flex', gap: 14, justifyContent: 'flex-end', marginTop: 6 }}>
        {series.map((s) => (
          <div key={s.name} style={{ display: 'flex', alignItems: 'center', gap: 6, fontSize: '11px' }}>
            <span style={{ width: 8, height: 8, borderRadius: 2, backgroundColor: s.color }} />
            <span className="text-secondary mono">{s.name}</span>
          </div>
        ))}
      </div>
    </div>
  );
};
