// One-time randomization for CSS variables on page load
let randomizationApplied = false;

export function applyRandomization() {
  if (randomizationApplied) return;
  randomizationApplied = true;

  const randomPx = (min: number, max: number) =>
    `${Math.round(min + Math.random() * (max - min))}px`;

  const bg = document.querySelector(".echo-gradient-bg");
  if (!bg) return;

  const elementStyle = (bg as HTMLElement).style;

  elementStyle.setProperty("--echo-bg-a-x", randomPx(-90, 90));
  elementStyle.setProperty("--echo-bg-a-y", randomPx(-50, 70));

  elementStyle.setProperty("--echo-bg-b-x", randomPx(-100, 100));
  elementStyle.setProperty("--echo-bg-b-y", randomPx(-70, 70));

  elementStyle.setProperty("--echo-bg-c-x", randomPx(-80, 80));
  elementStyle.setProperty("--echo-bg-c-y", randomPx(-60, 80));
}
