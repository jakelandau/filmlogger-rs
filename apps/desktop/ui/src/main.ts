// Entry point. Module scripts are deferred, so the DOM is ready.
import { init } from "./app";

init().catch((e) => {
  const root = document.getElementById("root");
  if (!root) return;
  const pre = document.createElement("pre");
  pre.textContent = String(e instanceof Error ? e.message : e);
  root.append(pre);
});
