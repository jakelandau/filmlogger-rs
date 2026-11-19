// The age-rating symbol on its shared light panel. Used by the log rows and
// the film page so both show the same size and fallback behaviour.
import { el } from "./dom";
import { certificationIcon } from "./cert-icons";

/** The board symbol, or its text fallback when no symbol is bundled. */
export function certificationBadge(region: string, cert: string | null): HTMLElement | null {
  if (cert == null) return null;
  const badge = el("span", { class: "cert-badge" });
  const icon = certificationIcon(region, cert);
  if (icon == null) {
    badge.append(cert);
    return badge;
  }
  badge.append(
    el("img", {
      class: "cert-icon",
      src: icon,
      alt: cert,
      onerror: (e: Event) => {
        (e.currentTarget as HTMLImageElement).style.display = "none";
      },
    })
  );
  return badge;
}
