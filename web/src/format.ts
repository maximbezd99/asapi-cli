const compact = new Intl.NumberFormat("en", {
  notation: "compact",
  maximumFractionDigits: 1,
});

export function formatCount(value?: number | null): string {
  return value == null ? "—" : compact.format(value);
}

export function formatDate(value?: string | null): string {
  if (!value) return "Never";
  return new Intl.DateTimeFormat("en", {
    month: "short",
    day: "numeric",
    year: "numeric",
  }).format(new Date(value));
}

export function relativeTime(value?: string | null): string {
  if (!value) return "Never";
  const difference = new Date(value).getTime() - Date.now();
  const absolute = Math.abs(difference);
  const formatter = new Intl.RelativeTimeFormat("en", { numeric: "auto" });
  if (absolute < 60_000) return "just now";
  if (absolute < 3_600_000)
    return formatter.format(Math.round(difference / 60_000), "minute");
  if (absolute < 86_400_000)
    return formatter.format(Math.round(difference / 3_600_000), "hour");
  return formatter.format(Math.round(difference / 86_400_000), "day");
}

export function countryFlag(country: string): string {
  return country
    .toUpperCase()
    .split("")
    .map((letter) => String.fromCodePoint(127397 + letter.charCodeAt(0)))
    .join("");
}

export function countryLabel(country: string): string {
  const names = new Intl.DisplayNames(["en"], { type: "region" });
  return names.of(country.toUpperCase()) ?? country.toUpperCase();
}

export function isStale(value?: string | null): boolean {
  if (!value) return true;
  return Date.now() - new Date(value).getTime() >= 24 * 60 * 60 * 1000;
}
