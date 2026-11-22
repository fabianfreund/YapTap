import { useEffect, useRef } from "react";

const BARS = 9;
// Middle bars move most, the edges least, like a voice meter.
const WEIGHTS = Array.from({ length: BARS }, (_, i) => 0.45 + 0.55 * Math.sin((Math.PI * (i + 0.5)) / BARS));

/**
 * Voice level bars. Reads the latest level from `levelRef` (0–1) and animates
 * at display rate with attack/release smoothing, so it never jitters.
 * Writes straight to the DOM, so React doesn't re-render 60 times a second.
 */
export function LevelBars({ levelRef, active }: { levelRef: React.RefObject<number>; active: boolean }) {
  const barsRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!active) return;
    let raf = 0;
    let smooth = 0;
    const heights = new Array(BARS).fill(0.12);
    const reduceMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;

    const tick = (t: number) => {
      const target = levelRef.current ?? 0;
      // Fast attack, slower release.
      smooth += (target - smooth) * (target > smooth ? 0.45 : 0.12);
      const bars = barsRef.current?.children;
      if (bars) {
        for (let i = 0; i < BARS; i++) {
          const wobble = reduceMotion ? 1 : 0.75 + 0.25 * Math.sin(t / 140 + i * 1.7);
          const goal = 0.12 + 0.88 * smooth * WEIGHTS[i] * wobble;
          heights[i] += (goal - heights[i]) * 0.35;
          (bars[i] as HTMLElement).style.transform = `scaleY(${heights[i].toFixed(3)})`;
        }
      }
      raf = requestAnimationFrame(tick);
    };
    raf = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(raf);
  }, [active, levelRef]);

  return (
    <div className="bars" ref={barsRef} aria-hidden>
      {WEIGHTS.map((_, i) => (
        <span key={i} />
      ))}
    </div>
  );
}
