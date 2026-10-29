// Settings: rating-board selector and database import/export by path.
import { el } from "../dom";
import { api } from "../api";
import { getState, reload, rerender, showError } from "../app";
import type { RatingBoard } from "../types";

const BOARDS: [RatingBoard, string][] = [
  ["mpa", "MPA (US)"],
  ["bbfc", "BBFC (UK)"],
  ["fsk", "FSK (DE)"],
];

export function renderSettings(content: HTMLElement): void {
  const board = getState().settings.selected_board;

  content.append(el("h3", {}, "Rating board"));
  const boardBox = el("div", { class: "board-select" });
  for (const [value, label] of BOARDS) {
    const radio = el("input", { type: "radio", name: "board", value, checked: board === value });
    radio.addEventListener("change", async () => {
      try {
        await api.setSelectedBoard(value);
        await reload();
        rerender();
      } catch (e) {
        showError(e);
      }
    });
    boardBox.append(el("label", {}, radio, " " + label));
  }
  content.append(boardBox);

  content.append(el("h3", {}, "Export database"));
  const exportInput = el("input", { class: "text", type: "text", placeholder: "Destination .db path" });
  const exportBtn = el("button", {
    class: "btn",
    onclick: async () => {
      const path = exportInput.value.trim();
      if (path === "") return;
      try {
        await api.exportTo(path);
        exportInput.value = "";
        alert("Exported to " + path);
      } catch (e) {
        showError(e);
      }
    },
  }, "Export");
  content.append(el("div", { class: "io-row" }, exportInput, exportBtn));

  content.append(el("h3", {}, "Import database"));
  const importInput = el("input", { class: "text", type: "text", placeholder: "Source .db path" });
  const importBtn = el("button", {
    class: "btn",
    onclick: async () => {
      const path = importInput.value.trim();
      if (path === "") return;
      if (!confirm("Replace all current data from " + path + "?")) return;
      try {
        await api.importFrom(path);
        importInput.value = "";
        await reload();
        rerender();
      } catch (e) {
        showError(e);
      }
    },
  }, "Import");
  content.append(el("div", { class: "io-row" }, importInput, importBtn));
}
