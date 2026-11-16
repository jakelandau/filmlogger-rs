import { el } from "./dom";
import type { Format } from "./types";

export type FormatFamily = "IMAX" | "DCI" | "FILM" | "HOME";

/** The projection family that distinguishes IMAX, DCI, and film screenings. */
export function formatFamily(format: Format | null): FormatFamily | null {
  if (format == null) return null;
  if ("home" in format) return "HOME";
  const projection = format.theatre.projection;
  if ("imax" in projection) return "IMAX";
  if ("dci" in projection) return "DCI";
  return "FILM";
}

export function formatFamilyClass(family: FormatFamily): string {
  return family.toLowerCase();
}

/** A prominent family badge, or null for logs with no stored format. */
export function formatFamilyBadge(format: Format | null): HTMLElement | null {
  const family = formatFamily(format);
  if (family == null) return null;
  return el("span", { class: `format-family ${formatFamilyClass(family)}` }, family);
}
