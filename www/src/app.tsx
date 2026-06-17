import { RiGithubFill } from '@remixicon/react'

function App() {
  return (
    <main className="flex min-h-screen items-center justify-center bg-white p-6 text-slate-950">
      <section className="relative h-[500px] w-full max-w-[624px] sm:h-[520px]">
        <svg
          width="470"
          height="300"
          viewBox="0 0 470 300"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
          role="img"
          aria-labelledby="logo-title logo-desc"
          className="ml-12 w-[calc(100%-3rem)]"
        >
          <title id="logo-title">Echo Programming Language</title>
          <desc id="logo-desc">A stylized Echo Programming Language logo.</desc>
          <defs>
            <linearGradient
              id="echo-gradient"
              x1="0"
              y1="14"
              x2="0"
              y2="286"
              gradientUnits="userSpaceOnUse"
            >
              <stop offset="0%" stopColor="#A855F7" />
              <stop offset="38%" stopColor="#6366F1" />
              <stop offset="68%" stopColor="#0EA5E9" />
              <stop offset="100%" stopColor="#06B6D4" />
            </linearGradient>
            <mask
              id="arc-mask"
              x="300"
              y="0"
              width="160"
              height="300"
              maskUnits="userSpaceOnUse"
            >
              <path
                d="M350 36 C415 86 415 214 350 264"
                fill="none"
                stroke="#fff"
                strokeWidth="32"
                strokeLinecap="butt"
                strokeLinejoin="round"
              />
              <rect
                x="-16"
                y="-6"
                width="32"
                height="12"
                rx="5"
                fill="#fff"
                transform="translate(350 36) rotate(128)"
              />
              <rect
                x="-16"
                y="-6"
                width="32"
                height="12"
                rx="5"
                fill="#fff"
                transform="translate(350 264) rotate(52)"
              />
            </mask>
          </defs>
          <text
            x="0"
            y="220"
            fill="#0B1018"
            fontFamily="Space Grotesk, Inter, system-ui, -apple-system, BlinkMacSystemFont, Segoe UI, sans-serif"
            fontSize="300"
            fontWeight="700"
            letterSpacing="-0.07em"
          >
            xo
          </text>
          <rect
            x="300"
            y="0"
            width="160"
            height="300"
            fill="url(#echo-gradient)"
            mask="url(#arc-mask)"
          />
        </svg>
        <div className="absolute left-0 top-[58%] w-[69%] text-left">
          <p className="text-pretty text-base leading-7 text-slate-600 sm:text-lg sm:leading-8">
            Echo is a Rust powered PHP superset for modern server software. It keeps PHP
            familiar while adding native concurrency, parallel execution, sharper diagnostics,
            stronger tooling, and a path to compiled binaries with predictable performance gains.
            The xo mark nods to Echo by sound rather than spelling, folding /ˈɛkoʊ/ toward a
            compact /ɛk oʊ/ for a small command shaped for the terminal.
          </p>
          <a
            href="https://github.com/modoterra/echo"
            target="_blank"
            rel="noreferrer"
            className="mt-6 inline-flex items-center gap-2 text-sm font-medium text-slate-500 transition hover:text-slate-950"
          >
            <RiGithubFill aria-hidden="true" className="size-5" />
            GitHub
          </a>
          <p className="mt-4 text-sm text-slate-400">© 2026 Modoterra Corporation</p>
        </div>
      </section>
    </main>
  )
}

export default App
