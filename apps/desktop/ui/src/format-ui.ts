// The viewing-format controls: a cascading projection family → technology →
// auditorium selector for theatre, and a release-medium selector for home.
// Both are rendered from the Rust-generated format catalog, so labels and
// legality rules cannot drift from the backend.
import { el, clear } from "./dom";
import type {
  Format,
  FormatCatalog,
  FormatLegality,
  FormatOption,
  Leaf,
  Projection,
  Venue,
} from "./types";

type Options = ReadonlyArray<readonly [string, string]>;

let catalog: FormatCatalog | null = null;

/** Install the catalog fetched from the backend. */
export function initCatalog(c: FormatCatalog): void {
  catalog = c;
}

function optionsOf(list: FormatOption[]): Options {
  return list.map((o) => [o.key, o.label] as const);
}

/** A venue-appropriate default format. */
export function defaultFormat(venue: Venue): Format {
  if (venue === "home") {
    return { home: { medium: "bluray", three_d: false, hfr: false } };
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

export interface FormatControls {
  root: HTMLElement;
  get: () => Format;
}

/** Build the format controls for `venue`, seeded from `initial`. */
export function formatControls(venue: Venue, initial: Format | null): FormatControls {
  if (catalog === null) {
    return {
      root: el("p", { class: "dim" }, "Loading formats…"),
      get: () => defaultFormat(venue),
    };
  }
  const active = catalog;
  if (venue === "home") {
    return buildHome(active, initial);
  }
  return buildTheatre(active, initial);
}

function parseLeaf(l: Leaf): { value: string; custom: string } {
  if (typeof l === "string") return { value: l, custom: "" };
  return { value: "custom", custom: l.custom };
}

interface LeafControl {
  select: HTMLSelectElement;
  input: HTMLInputElement;
  leaf: () => Leaf;
}

/**
 * A select plus an optional custom-text input. A legacy custom value that the
 * catalog does not know (from an older database) is preserved as an extra
 * `custom` option carrying its text; new formats never offer a blank custom
 * entry.
 */
function leafSelect(
  options: Options,
  initial: { value: string; custom: string },
  withCustom: boolean
): LeafControl {
  const select = el("select", {});
  const input = el("input", { class: "custom-input", placeholder: "Custom…" });
  input.style.display = "none";

  const items: Array<readonly [string, string]> = [...options];
  const legacy = initial.custom !== "" && !options.some(([v]) => v === initial.custom);
  if (withCustom && legacy) items.push(["custom", initial.custom]);
  else if (withCustom && initial.value === "custom") items.push(["custom", "Custom…"]);
  for (const [value, label] of items) {
    select.append(el("option", { value }, label));
  }

  const initialKey = initial.value === "custom" ? (legacy ? "custom" : options[0]?.[0] ?? "") : initial.value;
  select.value = items.some(([v]) => v === initialKey) ? initialKey : (items[0]?.[0] ?? "");
  input.value = initial.custom;

  const sync = () => {
    input.style.display = select.value === "custom" ? "" : "none";
  };
  select.addEventListener("change", sync);
  sync();

  return { select, input, leaf: () => (select.value === "custom" ? { custom: input.value.trim() } : select.value) };
}

function parseProjection(p: Projection): {
  family: string;
  tech: { value: string; custom: string };
} {
  if ("dci" in p) return { family: "dci", tech: parseLeaf(p.dci) };
  if ("imax" in p) return { family: "imax", tech: parseLeaf(p.imax) };
  return { family: "film", tech: parseLeaf(p.film) };
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

function buildTheatre(active: FormatCatalog, initial: Format | null): FormatControls {
  const t = initial && "theatre" in initial ? initial.theatre : null;
  const proj = t ? parseProjection(t.projection) : null;
  const family = proj?.family ?? "dci";
  const initialScreen = t ? parseLeaf(t.screen) : { value: "regular", custom: "" };

  const flags: Record<string, boolean> = {
    three_d: t?.three_d ?? false,
    hfr: t?.hfr ?? false,
    dbox: t?.dbox ?? false,
  };

  const familySel = el(
    "select",
    {},
    ...active.families.map((f) => el("option", { value: f.key }, f.label))
  );
  if (active.families.some((f) => f.key === family)) familySel.value = family;

  const tech = leafSelect(optionsOf(active.technologies[family] ?? []), proj?.tech ?? { value: "", custom: "" }, true);
  const screen = leafSelect(optionsOf(active.screens[family] ?? []), initialScreen, true);

  const row = (label: string, ...controls: HTMLElement[]) =>
    el("div", { class: "field-row" }, el("label", {}, label), ...controls);

  const flagHolder = el("div", { class: "flag-row" });

  function renderFlags(): void {
    const entry = active.legal.find(
      (row) =>
        row.family === familySel.value &&
        row.technology === tech.select.value &&
        row.screen === screen.select.value
    );
    clear(flagHolder);
    if (!entry) {
      flagHolder.append(el("span", { class: "dim" }, "no presentation options"));
      return;
    }
    let any = false;
    for (const flag of active.flag_labels) {
      if (!entry.allowed_flags.includes(flag.key)) {
        flags[flag.key] = false;
        continue;
      }
      any = true;
      const forced = entry.forced_flags.includes(flag.key);
      const cb = el("input", {
        type: "checkbox",
        checked: forced ? true : flags[flag.key],
        disabled: forced,
      });
      cb.addEventListener("change", () => {
        flags[flag.key] = cb.checked;
      });
      flagHolder.append(el("label", { class: "flag" }, cb, " " + flag.label));
    }
    if (!any) flagHolder.append(el("span", { class: "dim" }, "no presentation options"));
  }

  function setFamily(): void {
    const f = familySel.value;
    const techs = active.technologies[f] ?? [];
    const screens = active.screens[f] ?? [];
    clear(tech.select);
    for (const option of techs) tech.select.append(el("option", { value: option.key }, option.label));
    tech.select.value = techs[0]?.key ?? "";
    tech.select.dispatchEvent(new Event("change"));
    clear(screen.select);
    for (const option of screens) {
      screen.select.append(el("option", { value: option.key }, option.label));
    }
    screen.select.value = screens[0]?.key ?? "";
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
    row("Presentation", flagHolder)
  );

  const get = (): Format => ({
    theatre: {
      projection: projectionFrom(familySel.value, tech.leaf()),
      screen: screen.leaf(),
      three_d: flags["three_d"] ?? false,
      hfr: flags["hfr"] ?? false,
      dbox: flags["dbox"] ?? false,
    },
  });

  return { root, get };
}

function buildHome(active: FormatCatalog, initial: Format | null): FormatControls {
  const h = initial && "home" in initial ? initial.home : null;
  const medium = leafSelect(
    optionsOf(active.mediums),
    h ? parseLeaf(h.medium) : { value: "bluray", custom: "" },
    true
  );
  const flags: Record<string, boolean> = {
    three_d: h?.three_d ?? false,
    hfr: h?.hfr ?? false,
  };
  const flagHolder = el("div", { class: "flag-row" });

  function renderFlags(): void {
    const entry = active.home_legal.find((row) => row.medium === medium.select.value);
    clear(flagHolder);
    if (!entry) {
      flagHolder.append(el("span", { class: "dim" }, "no presentation options"));
      return;
    }
    let any = false;
    for (const flag of active.flag_labels) {
      if (!entry.allowed_flags.includes(flag.key)) {
        flags[flag.key] = false;
        continue;
      }
      any = true;
      const forced = entry.forced_flags.includes(flag.key);
      if (forced) flags[flag.key] = true;
      const cb = el("input", {
        type: "checkbox",
        checked: forced ? true : flags[flag.key],
        disabled: forced,
      });
      cb.addEventListener("change", () => {
        flags[flag.key] = cb.checked;
      });
      flagHolder.append(el("label", { class: "flag" }, cb, " " + flag.label));
    }
    if (!any) flagHolder.append(el("span", { class: "dim" }, "no presentation options"));
  }

  medium.select.addEventListener("change", renderFlags);
  renderFlags();

  const root = el(
    "div",
    { class: "format-home" },
    el(
      "div",
      { class: "field-row" },
      el("label", {}, "Release medium"),
      medium.select,
      medium.input
    ),
    el(
      "div",
      { class: "field-row" },
      el("label", {}, "Presentation"),
      flagHolder
    )
  );
  return {
    root,
    get: () => ({
      home: {
        medium: medium.leaf(),
        three_d: flags["three_d"] ?? false,
        hfr: flags["hfr"] ?? false,
      },
    }),
  };
}
