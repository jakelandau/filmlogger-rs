// Settings: rating-board selector and database import/export through native
// file dialogs.
import { open, save } from "@tauri-apps/plugin-dialog";
import { el } from "../dom";
import { api } from "../api";
import { getState, reload, rerender, showError } from "../app";
import { CREDIT_GROUPS, TMDB_ATTRIBUTION } from "../credits";
import { boardFlagUrl } from "../cert-icons";
import { clearFilmCaches } from "./films";
import { checkForUpdates } from "../updates";

const DB_FILTERS = [{ name: "SQLite database", extensions: ["db"] }];

export function renderSettings(content: HTMLElement): void {
  const board = getState().settings.selected_board;

  content.append(el("h3", {}, "Rating board"));
  content.append(el("p", { class: "muted" }, "Choose the board shown beside each log."));
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
    boardBox.append(
      el(
        "label",
        {},
        radio,
        " ",
        el("img", {
          class: "board-flag",
          src: boardFlagUrl(option.region),
          alt: option.region,
        }),
        " " + option.name
      )
    );
  }
  content.append(boardBox);

  content.append(el("h3", {}, "Database"));
  content.append(
    el(
      "p",
      { class: "muted" },
      "Export a copy of the database, or replace the current log with a backup."
    )
  );
  const ioStatus = el("p", { class: "io-status muted" });
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
            ioStatus.textContent = "Exported database.";
          }
        } catch (e) {
          showError(e);
        }
      },
    },
    "Export database"
  );
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
            clearFilmCaches();
            await reload();
            rerender();
            ioStatus.textContent = "Imported database.";
          }
        } catch (e) {
          showError(e);
        }
      },
    },
    "Import database"
  );
  content.append(el("div", { class: "io-row" }, exportBtn, importBtn), ioStatus);

  content.append(el("h3", {}, "Updates"));
  const updateStatus = el("p", { class: "muted" });
  const updateBtn = el(
    "button",
    {
      class: "btn",
      onclick: async () => {
        updateBtn.disabled = true;
        updateStatus.textContent = "Checking…";
        try {
          updateStatus.textContent = await checkForUpdates(false);
        } catch (e) {
          updateStatus.textContent = "Update check failed.";
          showError(e);
        } finally {
          updateBtn.disabled = false;
        }
      },
    },
    "Check for Updates"
  );
  content.append(el("div", { class: "io-row" }, updateBtn), updateStatus);

  content.append(el("h3", {}, "Licenses & credits"));
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
  content.append(
    el("img", {
      class: "tmdb-logo",
      src: "/tmdb-logo.svg",
      alt: "The Movie Database (TMDB)",
    })
  );
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
