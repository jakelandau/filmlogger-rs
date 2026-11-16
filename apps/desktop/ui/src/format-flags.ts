import { el } from "./dom";
import type { Format } from "./types";

export interface FormatFlag {
  key: "3d" | "hfr" | "motion";
  label: string;
}

const FLAG_ICONS: Record<FormatFlag["key"], { d: string; stroke?: boolean }> = {
  "3d": { d: "M2 9h7l3 3 3-3h7v6h-7l-3-3-3 3H2z" },
  hfr: { d: "M13 2 4 14h6l-1 8 9-12h-6z" },
  motion: {
    d: "M2 9c2.5-2.5 5-2.5 7.5 0s5 2.5 7.5 0 5-2.5 7.5 0M2 15c2.5-2.5 5-2.5 7.5 0s5 2.5 7.5 0 5-2.5 7.5 0",
    stroke: true,
  },
};

/** Theatre presentation flags, including 4DX-style motion seating. */
export function formatFlags(format: Format | null): FormatFlag[] {
  if (format == null || !("theatre" in format)) return [];
  const theatre = format.theatre;
  const screen = theatre.screen;
  const motionScreen = screen === "4dx" || screen === "ultra_4dx";
  const flags: FormatFlag[] = [];
  if (theatre.three_d || motionScreen) flags.push({ key: "3d", label: "3D" });
  if (theatre.hfr) flags.push({ key: "hfr", label: "HFR" });
  if (theatre.dbox || motionScreen) flags.push({ key: "motion", label: "Motion" });
  return flags;
}

function icon(key: FormatFlag["key"]): SVGSVGElement {
  const spec = FLAG_ICONS[key];
  const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
  svg.setAttribute("viewBox", "0 0 24 24");
  svg.setAttribute("class", "format-flag-icon");
  svg.setAttribute("aria-hidden", "true");
  const path = document.createElementNS("http://www.w3.org/2000/svg", "path");
  path.setAttribute("d", spec.d);
  if (spec.stroke) {
    path.setAttribute("fill", "none");
    path.setAttribute("stroke", "currentColor");
    path.setAttribute("stroke-width", "1.8");
    path.setAttribute("stroke-linecap", "round");
    path.setAttribute("stroke-linejoin", "round");
  } else {
    path.setAttribute("fill", "currentColor");
  }
  svg.append(path);
  return svg;
}

/** The flag chips for a viewing, or null when the viewing has no flags. */
export function formatFlagsRow(format: Format | null): HTMLElement | null {
  const flags = formatFlags(format);
  if (flags.length === 0) return null;
  return el(
    "div",
    { class: "format-flags" },
    ...flags.map((flag) =>
      el("span", { class: "format-flag" }, icon(flag.key), el("span", {}, flag.label))
    )
  );
}
