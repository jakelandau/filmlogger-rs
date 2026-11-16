// GitHub Releases updater, backed by the Tauri updater plugin.
import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

let checking = false;

/**
 * Checks the configured GitHub Releases endpoint and, when a newer signed
 * release exists, downloads and installs it before restarting the app.
 */
export async function checkForUpdates(silent = false): Promise<string> {
  if (checking) return "An update check is already in progress.";
  checking = true;
  try {
    const update = await check();
    if (update == null) return "FilmLogger is up to date.";
    await update.downloadAndInstall();
    await relaunch();
    return `Installing FilmLogger ${update.version}…`;
  } catch (error) {
    if (silent) return "";
    throw error;
  } finally {
    checking = false;
  }
}
