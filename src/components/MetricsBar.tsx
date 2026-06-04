interface MetricsBarProps {
  latencyMs: number;
  chunkProcessingMs: number;
  wordCount: number;
  characterCount: number;
}

function Metric({ value, label }: { value: string | number; label: string }) {
  return (
    <div className="metric">
      <div className="value">{value}</div>
      <div className="label">{label}</div>
    </div>
  );
}

export function MetricsBar({
  latencyMs,
  chunkProcessingMs,
  wordCount,
  characterCount,
}: MetricsBarProps) {
  return (
    <>
      <Metric value={latencyMs > 0 ? `${latencyMs.toFixed(0)}` : "—"} label="Latency (ms)" />
      <Metric value={chunkProcessingMs > 0 ? `${chunkProcessingMs.toFixed(0)}` : "—"} label="Chunk (ms)" />
      <Metric value={wordCount} label="Words" />
      <Metric value={characterCount} label="Characters" />
    </>
  );
}
