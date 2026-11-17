// Official rating symbols bundled from Wikimedia Commons.
// Boards without an official symbol for a value render text only.
import { OFFICIAL_RATING_ICONS } from "./official-icons";

const OFFICIAL = "/cert-icons/official/";

export function certificationIcon(region: string, rating: string): string | null {
  const values = OFFICIAL_RATING_ICONS[region.toUpperCase()];
  if (values == null) return null;
  const file = values[rating.trim().toUpperCase()];
  return file ? OFFICIAL + encodeURIComponent(file) : null;
}

/** The country flag for a rating board's region code (`US` → `us.svg`). */
export function boardFlagUrl(region: string): string {
  return "/cert-icons/flags/" + region.toLowerCase() + ".svg";
}
