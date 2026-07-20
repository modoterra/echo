export function GradientBackground() {
  return (
    <div
      className="echo-gradient-bg"
      aria-hidden="true"
      role="img"
      aria-label="Animated gradient background for Echo Programming Language"
    >
      <svg
        viewBox="0 0 1200 700"
        preserveAspectRatio="xMidYMid slice"
        className="absolute inset-0 h-full w-full"
      >
        <defs>
          <filter id="echo-bg-blur">
            <feGaussianBlur stdDeviation="70" />
          </filter>

          <filter id="echo-bg-distort">
            <feTurbulence
              type="fractalNoise"
              baseFrequency="0.006 0.012"
              numOctaves="2"
              seed="8"
              result="noise"
            />
            <feDisplacementMap
              in="SourceGraphic"
              in2="noise"
              scale="28"
              xChannelSelector="R"
              yChannelSelector="G"
            />
          </filter>

          <radialGradient id="echo-bg-g1" cx="50%" cy="50%" r="50%">
            <stop offset="0%" stopColor="#7c3aed" />
            <stop offset="45%" stopColor="#2563eb" />
            <stop offset="100%" stopColor="transparent" />
          </radialGradient>

          <radialGradient id="echo-bg-g2" cx="50%" cy="50%" r="50%">
            <stop offset="0%" stopColor="#06b6d4" />
            <stop offset="55%" stopColor="#3b82f6" />
            <stop offset="100%" stopColor="transparent" />
          </radialGradient>

          <radialGradient id="echo-bg-g3" cx="50%" cy="50%" r="50%">
            <stop offset="0%" stopColor="#f97316" />
            <stop offset="50%" stopColor="#ec4899" />
            <stop offset="100%" stopColor="transparent" />
          </radialGradient>
        </defs>

        <g filter="url(#echo-bg-distort)">
          <g filter="url(#echo-bg-blur)">
            <ellipse
              className="echo-bg-blob echo-bg-blob-a"
              cx="360"
              cy="250"
              rx="310"
              ry="180"
              fill="url(#echo-bg-g1)"
            />
            <ellipse
              className="echo-bg-blob echo-bg-blob-b"
              cx="760"
              cy="320"
              rx="360"
              ry="210"
              fill="url(#echo-bg-g2)"
            />
            <ellipse
              className="echo-bg-blob echo-bg-blob-c"
              cx="560"
              cy="440"
              rx="330"
              ry="170"
              fill="url(#echo-bg-g3)"
            />
          </g>
        </g>
      </svg>
    </div>
  );
}
