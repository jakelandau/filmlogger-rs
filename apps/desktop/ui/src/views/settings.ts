// Settings: rating-board selector and database import/export through native
// file dialogs.
import { open, save } from "@tauri-apps/plugin-dialog";
import { el } from "../dom";
import { api } from "../api";
import { getState, reload, rerender, showError } from "../app";
import { CREDIT_GROUPS, TMDB_ATTRIBUTION } from "../credits";

const DB_FILTERS = [{ name: "SQLite database", extensions: ["db"] }];

export function renderSettings(content: HTMLElement): void {
  const board = getState().settings.selected_board;

  content.append(el("h3", {}, "Rating board"));
  const boardBox = el("div", { class: "board-select" });
  for (const option of getState().ratingBoards) {
    const radio = el("input", {
      type: "radio",
      name: "board",
      value: option.region,
      checked: board === option.region,
    });
    radio.addEventListener("change", async () => {
      try {
        await api.setSelectedBoard(option.region);
        await reload();
        rerender();
      } catch (e) {
        showError(e);
      }
    });
    boardBox.append(el("label", {}, radio, " " + option.name));
  }
  content.append(boardBox);

  content.append(el("h3", {}, "Export database"));
  const exportBtn = el(
    "button",
    {
      class: "btn",
      onclick: async () => {
        try {
          const path = await save({
            defaultPath: "film-logger.db",
            filters: DB_FILTERS,
          });
          if (path) {
            await api.exportTo(path);
            alert("Exported to " + path);
          }
        } catch (e) {
          showError(e);
        }
      },
    },
    "Export…"
  );
  content.append(el("div", { class: "io-row" }, exportBtn));

  content.append(el("h3", {}, "Import database"));
  const importBtn = el(
    "button",
    {
      class: "btn",
      onclick: async () => {
        try {
          const path = await open({
            multiple: false,
            directory: false,
            filters: DB_FILTERS,
          });
          if (typeof path === "string" && confirm("Replace all current data?")) {
            await api.importFrom(path);
            await reload();
            rerender();
          }
        } catch (e) {
          showError(e);
        }
      },
    },
    "Import…"
  );
  content.append(el("div", { class: "io-row" }, importBtn));

  content.append(el("h3", {}, "Licenses / credits"));
  content.append(
    el(
      "p",
      { class: "muted" },
      "FilmLogger is built with these third-party libraries. Transitive dependencies retain their own licenses."
    )
  );
  for (const group of CREDIT_GROUPS) {
    content.append(el("h4", {}, group.title));
    const list = el("ul", { class: "credits" });
    for (const credit of group.items) {
      list.append(el("li", {}, el("strong", {}, credit.name), ` — ${credit.license}`));
    }
    content.append(list);
  }

  content.append(el("h3", {}, "TMDB"));
  content.append(el("p", { class: "muted" }, TMDB_ATTRIBUTION));
  content.append(
    el(
      "p",
      {},
      el(
        "a",
        { href: "https://www.themoviedb.org/", target: "_blank", rel: "noreferrer" },
        "themoviedb.org"
      )
    )
  );
}
