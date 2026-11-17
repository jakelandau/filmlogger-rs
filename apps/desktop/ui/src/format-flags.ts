import { el } from "./dom";
import type { Format } from "./types";

export interface FormatFlag {
  key: "3d" | "hfr" | "motion";
  label: string;
}

interface IconPath {
  d: string;
  stroke?: boolean;
}

const FLAG_ICONS: Record<FormatFlag["key"], IconPath[]> = {
  "3d": [{ d: "M2 9h7l3 3 3-3h7v6h-7l-3-3-3 3H2z" }],
  hfr: [
    { d: "M3.5 14a8.5 8.5 0 0 1 17 0", stroke: true },
    { d: "M12 14l4.5-6.5", stroke: true },
    { d: "M12 13.5a1.5 1.5 0 1 0 0 3a1.5 1.5 0 0 0 0-3z" },
  ],
  motion: [
    { d: "M7 4h10a2 2 0 0 1 2 2v6H5V6a2 2 0 0 1 2-2z" },
    { d: "M3 12h18v5H3z" },
    { d: "M6 19h3v2H6zM15 19h3v2h-3z" },
  ],
};

/** Theatre presentation flags, including 4DX-style motion seating. */
export function formatFlags(format: Format | null): FormatFlag[] {
  if (format == null) return [];
  if ("home" in format) {
    const flags: FormatFlag[] = [];
    if (format.home.three_d) flags.push({ key: "3d", label: "3D" });
    if (format.home.hfr) flags.push({ key: "hfr", label: "HFR" });
    return flags;
  }
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
  const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
  svg.setAttribute("viewBox", "0 0 24 24");
  svg.setAttribute("class", "format-flag-icon");
  svg.setAttribute("aria-hidden", "true");
  for (const spec of FLAG_ICONS[key]) {
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
  }
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
