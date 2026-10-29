// The log list: every viewing, most recent first, with edit/delete actions.
import { el } from "../dom";
import { api } from "../api";
import { getState, go, reload, showError } from "../app";
import { formatDisplay } from "../time";
import type { Format, Leaf, RatingBoard, WatchLog } from "../types";

const IMAGE_BASE = "https://image.tmdb.org/t/p/w92";

const DCI_LABEL: Record<string, string> = {
  "2k_lamp": "2K xenon",
  "2k_laser": "2K laser",
  "4k_lamp": "4K xenon",
  "4k_laser": "4K laser",
};
const IMAX_LABEL: Record<string, string> = {
  "2k_xenon": "2K xenon",
  "4k_single_laser": "4K single laser",
  "4k_dual_laser": "4K dual laser",
  "70mm": "70mm",
};
const FILM_LABEL: Record<string, string> = { "35mm": "35mm", "70mm": "70mm" };
const SCREEN_LABEL: Record<string, string> = {
  regular: "Regular",
  plf: "PLF",
  vip: "VIP",
  "4dx": "4DX",
  screenx: "ScreenX",
  imax_class: "IMAX 1.43:1",
  imax_retrofit: "IMAX 1.90:1",
  ultra_4dx: "Ultra 4DX",
};
const MEDIUM_LABEL: Record<string, string> = {
  dvd: "DVD",
  hd_dvd: "HD DVD",
  bluray: "Blu-ray",
  uhd_bluray: "4K UHD Blu-ray",
  laserdisc: "LaserDisc",
  streaming: "Streaming",
  broadcast: "Broadcast",
  telecine: "Telecine",
};

function leafLabel(map: Record<string, string>, leaf: Leaf): string {
  if (typeof leaf === "string") return map[leaf] ?? leaf;
  return leaf.custom || "Custom";
}

function formatSummary(format: Format | null): string {
  if (!format) return "—";
  if ("home" in format) return leafLabel(MEDIUM_LABEL, format.home.medium);
  const t = format.theatre;
  let proj: string;
  if ("dci" in t.projection) proj = leafLabel(DCI_LABEL, t.projection.dci);
  else if ("imax" in t.projection) proj = leafLabel(IMAX_LABEL, t.projection.imax);
  else proj = leafLabel(FILM_LABEL, t.projection.film);
  const screen = leafLabel(SCREEN_LABEL, t.screen);
  const flags = [
    t.three_d && "3D",
    t.hfr && "HFR",
    t.dbox && "D-BOX",
  ].filter(Boolean) as string[];
  const tail = flags.length > 0 ? " · " + flags.join(", ") : "";
  return `${proj} · ${screen}${tail}`;
}

export function renderLogs(content: HTMLElement): void {
  const logs = getState().logs;
  if (logs.length === 0) {
    content.append(
      el(
        "p",
        { class: "empty" },
        "No films logged yet. ",
        el("button", { class: "link", onclick: () => go("add") }, "Add your first film")
      )
    );
    return;
  }

  const list = el("ul", { class: "log-list" });
  for (const log of logs) list.append(renderRow(log));
  content.append(list);
}

/// The film's certification for the user's selected rating board, or null
/// when unknown (the film has no certifications for that board).
function certificationFor(log: WatchLog, board: RatingBoard): string | null {
  const c = log.certifications;
  if (c == null) return null;
  if (board === "mpa") return c.us;
  if (board === "bbfc") return c.gb;
  return c.de;
}

function renderRow(log: WatchLog): HTMLElement {
  const rating =
    log.rating === "up" ? "👍" : log.rating === "down" ? "👎" : null;
  const cert = certificationFor(log, getState().settings.selected_board);
  const venue = log.venue === "theatre" ? "🎭 Theatre" : "🏠 Home";
  const poster = log.poster_path
    ? el("img", { class: "poster", src: IMAGE_BASE + log.poster_path, alt: "" })
    : el("div", { class: "poster placeholder" });

  const actions = el(
    "span",
    { class: "row-actions" },
    el("button", { class: "btn", onclick: () => go("edit", log.id) }, "Edit"),
    el("button", { class: "btn danger", onclick: () => deleteLog(log) }, "Delete")
  );

  return el(
    "li",
    { class: "log-row" },
    poster,
    el(
      "div",
      { class: "log-main" },
      el(
        "div",
        { class: "log-title" },
        log.title,
        log.release_year != null ? el("span", { class: "dim" }, ` (${log.release_year})`) : "",
        el("span", { class: "badge venue-" + log.venue }, venue),
        rating ? el("span", { class: "rating" }, " " + rating) : "",
        el("span", { class: "badge" + (cert ? "" : " unrated") }, cert ?? "unrated"),
        log.rewatch ? el("span", { class: "badge rewatch" }, " rewatch") : ""
      ),
      el(
        "div",
        { class: "log-sub" },
        el("span", { class: "dim" }, formatDisplay(log.watched_at)),
        el("span", { class: "dim" }, " · " + formatSummary(log.format))
      )
    ),
    actions
  );
}

async function deleteLog(log: WatchLog): Promise<void> {
  if (!confirm(`Delete “${log.title}”?`)) return;
  try {
    await api.deleteLog(log.id);
    await reload();
    go("logs");
  } catch (e) {
    showError(e);
  }
}
