// Conversions between Rust's RFC 3339 UTC timestamps and the browser's
// `datetime-local` input (local wall time, no zone).

// `iso` is an RFC 3339 UTC string; returns `YYYY-MM-DDTHH:MM` in local time.
export function toLocalInput(iso: string): string {
  const d = new Date(iso);
  const local = new Date(d.getTime() - d.getTimezoneOffset() * 60000);
  return local.toISOString().slice(0, 16);
}

// `value` is a `datetime-local` value; returns RFC 3339 UTC.
export function fromLocalInput(value: string): string {
  return new Date(value).toISOString();
}

export function formatDisplay(iso: string): string {
  const d = new Date(iso);
  return Number.isNaN(d.getTime()) ? iso : d.toLocaleString();
}

export function formatYear(iso: string): string | null {
  const d = new Date(iso);
  return Number.isNaN(d.getTime()) ? null : String(d.getUTCFullYear());
}
