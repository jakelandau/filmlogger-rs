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

// A venue-appropriate default format, used when a log is created without
// one (a viewing format is required on every log).
export function defaultFormat(venue: Venue): Format {
  if (venue === "home") {
    return { home: { medium: "bluray" } };
  }
  return {
    theatre: {
      projection: { dci: "2k_lamp" },
      screen: "regular",
      three_d: false,
      hfr: false,
      dbox: false,
    },
  };
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

// The technology options for a projection family.
function techOptions(family: string): Options {
  return family === "imax" ? IMAX : family === "film" ? FILM : DCI;
}

// The auditorium options legal for a projection family, mirroring the Rust
// `Projection::runs_on`: IMAX runs only in IMAX auditoria, film only in
// regular ones, DCI in everything except IMAX.
function screensForFamily(family: string): Options {
  if (family === "imax") {
    return SCREENS.filter(([v]) => v === "imax_class" || v === "imax_retrofit");
  }
  if (family === "film") {
    return SCREENS.filter(([v]) => v === "regular");
  }
  return SCREENS.filter(([v]) => v !== "imax_class" && v !== "imax_retrofit");
}

// Mirrors the Rust `Projection::allows` and `ScreenClass::allows` tables:
// whether the given flag may be on for this projection × screen.
function flagAllowed(flag: string, family: string, tech: string, screen: string): boolean {
  let projectionAllows: boolean;
  if (family === "dci") projectionAllows = true;
  else if (family === "film") projectionAllows = flag === "3d";
  else projectionAllows = tech === "70mm" ? false : flag !== "dbox";
  if (!projectionAllows) return false;
  if (screen === "screenx") return false;
  if (
    screen === "vip" ||
    screen === "4dx" ||
    screen === "ultra_4dx" ||
    screen === "imax_class" ||
    screen === "imax_retrofit"
  ) {
    return flag !== "dbox";
  }
  return true;
}

// Mirrors the Rust `ScreenClass::forces`: 4DX and Ultra 4DX auditoria are
// 3D by definition.
function screenForcesFlag(screen: string, flag: string): boolean {
  return flag === "3d" && (screen === "4dx" || screen === "ultra_4dx");
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

// The theatre cascade, with the Rust legality tables mirrored: the screen
// options are restricted to the projection family, and the presentation-flag
// checkboxes are restricted to what the selected technology and screen
// allow (forced flags are checked and disabled). The flag state is the
// source of truth, surviving each rebuild of the checkboxes.
function buildTheatre(initial: Format | null): { root: HTMLElement; get: () => Format } {
  const t = initial && "theatre" in initial ? initial.theatre : null;
  const proj = t ? parseProjection(t.projection) : null;
  const initialScreen = t ? parseLeaf(t.screen) : null;
  const family = proj?.family ?? "dci";

  const flags = {
    three_d: t?.three_d ?? false,
    hfr: t?.hfr ?? false,
    dbox: t?.dbox ?? false,
  };

  const familySel = el("select", {}, ...FAMILIES.map(([v, l]) => el("option", { value: v }, l)));
  familySel.value = family;
  const tech = leafSelect(
    techOptions(family),
    proj?.tech ?? "2k_lamp",
    proj?.techCustom ?? "",
  );
  const screen = leafSelect(
    screensForFamily(family),
    initialScreen?.value ?? "regular",
    initialScreen?.custom ?? "",
  );

  const row = (label: string, ...controls: HTMLElement[]) =>
    el("div", { class: "field-row" }, el("label", {}, label), ...controls);

  const FLAG_DEFS = [
    ["3d", "three_d", "3D"],
    ["hfr", "hfr", "HFR"],
    ["dbox", "dbox", "D-BOX"],
  ] as const;
  const flagHolder = el("div", { class: "flag-row" });

  // Rebuild the flag checkboxes for the current family/technology/screen:
  // legal flags only, forced flags checked and disabled.
  function renderFlags(): void {
    const f = familySel.value;
    const techValue = tech.select.value;
    const screenValue = screen.select.value;
    clear(flagHolder);
    let any = false;
    for (const [key, prop, label] of FLAG_DEFS) {
      if (!flagAllowed(key, f, techValue, screenValue)) {
        flags[prop] = false;
        continue;
      }
      any = true;
      const forced = screenForcesFlag(screenValue, key);
      const cb = el("input", { type: "checkbox", checked: forced ? true : flags[prop] });
      if (forced) cb.disabled = true;
      cb.addEventListener("change", () => {
        flags[prop] = cb.checked;
      });
      flagHolder.append(el("label", { class: "flag" }, cb, " " + label));
    }
    if (!any) flagHolder.append(el("span", { class: "dim" }, "no presentation options"));
  }

  // Cascade: family → technology and screen options, then the flags.
  function setFamily(): void {
    const f = familySel.value;
    const opts = techOptions(f);
    clear(tech.select);
    for (const [v, l] of opts) tech.select.append(el("option", { value: v }, l));
    tech.select.value = opts[0][0];
    tech.select.dispatchEvent(new Event("change"));

    const screenOpts = screensForFamily(f);
    clear(screen.select);
    for (const [v, l] of screenOpts) screen.select.append(el("option", { value: v }, l));
    if (!screenOpts.some(([v]) => v === screen.select.value)) {
      screen.select.value = screenOpts[0][0];
    }
    screen.select.dispatchEvent(new Event("change"));
    renderFlags();
  }

  familySel.addEventListener("change", setFamily);
  tech.select.addEventListener("change", renderFlags);
  screen.select.addEventListener("change", renderFlags);
  renderFlags();

  const root = el(
    "div",
    { class: "format-theatre" },
    row("Projection family", familySel),
    row("Technology", tech.select, tech.input),
    row("Auditorium", screen.select, screen.input),
    row("Presentation", flagHolder),
  );

  const get = (): Format => ({
    theatre: {
      projection: projectionFrom(familySel.value, tech.leaf()),
      screen: screen.leaf(),
      three_d: flags.three_d,
      hfr: flags.hfr,
      dbox: flags.dbox,
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
