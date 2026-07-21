/** Deterministic CSS color washes behind the homepage hero. */
export function GradientBackground() {
  return (
    <div className="echo-gradient-bg" aria-hidden="true">
      <div className="echo-bg-blob echo-bg-blob-a" />
      <div className="echo-bg-blob echo-bg-blob-b" />
      <div className="echo-bg-blob echo-bg-blob-c" />
    </div>
  );
}
