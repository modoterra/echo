/**
 * Soft floating color blobs for the home hero.
 *
 * Intentionally CSS-only (radial gradients + transform). The previous SVG
 * stack (feGaussianBlur + feTurbulence + feDisplacementMap) re-filtered every
 * animation frame and caused visible shimmer/flicker.
 */
export function GradientBackground() {
  return (
    <div className="echo-gradient-bg" aria-hidden="true">
      <div className="echo-bg-blob echo-bg-blob-a" />
      <div className="echo-bg-blob echo-bg-blob-b" />
      <div className="echo-bg-blob echo-bg-blob-c" />
    </div>
  );
}
