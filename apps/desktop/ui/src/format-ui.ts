// The viewing-format controls: a cascading projection family → technology
// selector for theatre, and a release-medium selector for home. The builder
// is imperative (the cascades are inherently stateful) and exposes a
// `get()` that reads the current selection as a serialized `Format`.
import { el, clear } from "./dom";
import type { Format, Leaf, Projection, Venue } from "./types";

type Options = ReadonlyArray<readonly [string, string]>;

const FAMILIES: Options = [
  ["dci", "Digital (DCI)"],
  ["imax", "IMAX"],
  ["film", "Film"],
];

const DCI: Options = [
  ["2k_lamp", "2K xenon"],
  ["2k_laser", "2K laser"],
  ["4k_lamp", "4K xenon"],
  ["4k_laser", "4K laser"],
  ["custom", "Custom…"],
];

const IMAX: Options = [
  ["2k_xenon", "2K xenon"],
  ["4k_single_laser", "4K single laser"],
  ["4k_dual_laser", "4K dual laser"],
  ["70mm", "70mm"],
  ["custom", "Custom…"],
];

const FILM: Options = [
  ["35mm", "35mm"],
  ["70mm", "70mm"],
  ["custom", "Custom…"],
];

const SCREENS: Options = [
  ["regular", "Regular"],
  ["plf", "PLF"],
  ["vip", "VIP"],
  ["4dx", "4DX"],
  ["screenx", "ScreenX"],
  ["imax_class", "IMAX 1.43:1"],
  ["imax_retrofit", "IMAX 1.90:1"],
  ["ultra_4dx", "Ultra 4DX"],
  ["custom", "Custom…"],
];

const MEDIUMS: Options = [
  ["dvd", "DVD"],
  ["hd_dvd", "HD DVD"],
  ["bluray", "Blu-ray"],
  ["uhd_bluray", "4K Ultra HD Blu-ray"],
  ["laserdisc", "LaserDisc"],
  ["streaming", "Streaming"],
  ["broadcast", "Broadcast"],
  ["telecine", "Telecine"],
  ["custom", "Custom…"],
];

export interface FormatControls {
  root: HTMLElement;
  get: () => Format;
}

// Build the format controls for `venue`, seeded from `initial` (when its
// venue matches). `get()` reads the live selection.
export function formatControls(venue: Venue, initial: Format | null): FormatControls {
  const root = el("div", { class: "format-controls" });
  if (venue === "home") {
    const c = buildHome(initial);
    root.append(c.root);
    return { root, get: c.get };
  }
  const c = buildTheatre(initial);
  root.append(c.root);
  return { root, get: c.get };
}

// A select plus an optional custom-text input, shown when "custom" is chosen.
function leafSelect(options: Options, initialValue: string, initialCustom: string): {
  select: HTMLSelectElement;
  input: HTMLInputElement;
  leaf: () => Leaf;
} {
  const select = el("select", {}, ...options.map(([v, l]) => el("option", { value: v }, l)));
  const input = el("input", { class: "custom-input", placeholder: "Custom…" });
  input.style.display = "none";

  const sync = () => {
    input.style.display = select.value === "custom" ? "" : "none";
  };
  select.addEventListener("change", sync);

  select.value = options.some(([v]) => v === initialValue) ? initialValue : "custom";
  input.value = initialCustom;
  sync();

  const leaf = (): Leaf =>
    select.value === "custom" ? { custom: input.value.trim() } : select.value;

  return { select, input, leaf };
}

function parseLeaf(l: Leaf): { value: string; custom: string } {
  if (typeof l === "string") return { value: l, custom: "" };
  return { value: "custom", custom: l.custom };
}

function parseProjection(p: Projection): {
  family: string;
  tech: string;
  techCustom: string;
} {
  if ("dci" in p) {
    const { value, custom } = parseLeaf(p.dci);
    return { family: "dci", tech: value, techCustom: custom };
  }
  if ("imax" in p) {
    const { value, custom } = parseLeaf(p.imax);
    return { family: "imax", tech: value, techCustom: custom };
  }
  const { value, custom } = parseLeaf(p.film);
  return { family: "film", tech: value, techCustom: custom };
}

function projectionFrom(family: string, tech: Leaf): Projection {
  switch (family) {
    case "imax":
      return { imax: tech };
    case "film":
      return { film: tech };
    default:
      return { dci: tech };
  }
}

function buildTheatre(initial: Format | null): { root: HTMLElement; get: () => Format } {
  const t = initial && "theatre" in initial ? initial.theatre : null;
  const proj = t ? parseProjection(t.projection) : null;

  const familySel = el("select", {}, ...FAMILIES.map(([v, l]) => el("option", { value: v }, l)));
  const tech = leafSelect(DCI, proj?.tech ?? "2k_lamp", proj?.techCustom ?? "");
  const screen = leafSelect(SCREENS, proj ? "regular" : "regular", "");
  // Restore screen from initial.
  if (t) {
    const s = parseLeaf(t.screen);
    screen.select.value = SCREENS.some(([v]) => v === s.value) ? s.value : "custom";
    screen.input.value = s.custom;
    screen.select.dispatchEvent(new Event("change"));
  }

  const threeD = el("input", { type: "checkbox", checked: t?.three_d ?? false });
  const hfr = el("input", { type: "checkbox", checked: t?.hfr ?? false });
  const dbox = el("input", { type: "checkbox", checked: t?.dbox ?? false });

  // Cascade: family → technology options.
  const setFamily = () => {
    const opts =
      familySel.value === "imax" ? IMAX : familySel.value === "film" ? FILM : DCI;
    clear(tech.select);
    for (const [v, l] of opts) tech.select.append(el("option", { value: v }, l));
    tech.select.value = "custom";
    tech.select.dispatchEvent(new Event("change"));
  };
  familySel.addEventListener("change", setFamily);
  if (proj) {
    familySel.value = proj.family;
    setFamily();
    tech.select.value = proj.tech;
    tech.input.value = proj.techCustom;
    tech.select.dispatchEvent(new Event("change"));
  }

  const row = (label: string, ...controls: HTMLElement[]) =>
    el("div", { class: "field-row" }, el("label", {}, label), ...controls);

  const root = el(
    "div",
    { class: "format-theatre" },
    row("Projection family", familySel),
    row("Technology", tech.select, tech.input),
    row("Auditorium", screen.select, screen.input),
    el(
      "div",
      { class: "flag-row" },
      el("label", { class: "flag" }, threeD, " 3D"),
      el("label", { class: "flag" }, hfr, " HFR"),
      el("label", { class: "flag" }, dbox, " D-BOX"),
    ),
  );

  const get = (): Format => ({
    theatre: {
      projection: projectionFrom(familySel.value, tech.leaf()),
      screen: screen.leaf(),
      three_d: threeD.checked,
      hfr: hfr.checked,
      dbox: dbox.checked,
    },
  });

  return { root, get };
}

function buildHome(initial: Format | null): { root: HTMLElement; get: () => Format } {
  const h = initial && "home" in initial ? initial.home : null;
  const medium = leafSelect(MEDIUMS, h ? parseLeaf(h.medium).value : "dvd", h ? parseLeaf(h.medium).custom : "");

  const root = el(
    "div",
    { class: "format-home" },
    el("div", { class: "field-row" }, el("label", {}, "Release medium"), medium.select, medium.input),
  );

  const get = (): Format => ({ home: { medium: medium.leaf() } });

  return { root, get };
}
