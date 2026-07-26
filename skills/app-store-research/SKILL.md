---
name: app-store-research
description: Research public Apple App Store apps, developers, metadata, prices, in-app purchases, ratings, written reviews, charts, competitors, search results, categories, and country comparisons. Use for current, country-specific App Store research and comparisons;.
---

# Research the public Apple App Store

Use the `asapi` CLI, a read-only command-line client for public Apple App Store data, to perform country-specific research. Use it for app discovery, metadata lookup, multi-country popularity signals, displayed in-app purchases, recent written reviews, and chart observations without App Store Connect credentials.

This skill is designed for `asapi` CLI version `1.2.0`.

Before research, run `command -v asapi`. If it is missing, stop, state that the required CLI is unavailable, and suggest installing it from <https://github.com/maximbezd99/asapi-cli>. Otherwise run `asapi --version`, and if the reported version differs from the skill version above, highlight the mismatch for the user's information, recommend running `asapi install-skill` to update the skill before a future agent session, and continue with the available CLI. Do not fall back to raw HTTP.

Research commands require outbound HTTPS. When command execution is sandboxed with network access disabled, use the environment's scoped network-access or approval mechanism on the first attempt. Otherwise run `asapi` normally. If network access is unavailable or denied, stop and state that access to `itunes.apple.com` or `apps.apple.com` is required. Do not treat a sandbox-denied attempt as evidence that Apple is unavailable.

## Discover commands

- Run `asapi --help` to see the current command set.
- Run `asapi <command> --help` before using unfamiliar options.
- Use `asapi list countries`, `asapi list categories`, or `asapi list chart-types` instead of guessing supported values.

## Choose the command

| Research need | Command |
| --- | --- |
| Find apps by name or keyword | `asapi search "term" --country us --limit 10` |
| Get normalized app metadata | `asapi lookup 123456789 987654321 --country us` |
| Compare an app's public rating counts across countries | `asapi popularity 123456789` |
| Inspect displayed in-app purchases | `asapi iap 123456789 --country us` |
| Sample recent written reviews | `asapi reviews 123456789 --country us --pages 3` |
| Observe a free apps chart | `asapi chart free --country us --limit 25 --category 6007` |

Resolve uncertain app identities with `search`, then use the returned `app_id` or `app_store_url` with `lookup`, `popularity`, `iap`, or `reviews`. For `lookup`, `iap`, and `reviews`, omit `--country` to use the storefront in an App Store URL; an explicit `--country` takes priority, and raw IDs default to `us`. Batch multiple known IDs or URLs in one `lookup` call. If batched URLs use different storefronts, pass `--country` explicitly. `popularity` ignores the URL storefront and continues to select countries with `--group` or `--countries`.

Use `popularity` for a multi-country comparison of one app:

- Omit selection options to use the Tier 1 group: `us,ca,cn,jp,gb,de,fr,kr,au`.
- Use `--group tier2` to add `in,br,mx,es,it,nl,id,sg,hk,tw,ae` to all Tier 1 countries.
- Use `--countries jp,us,gb` to query only an explicit comma-separated list. Treat `--countries` as overriding `--group`, even when both appear.

## Handle output

- Read stdout as JSON. Research commands return `data` and `meta`; `list` commands return a raw array.
- Keep compact JSON for automation. Add `--pretty` only for human inspection.
- Use `--output-file <path>` when the result must be saved as an artifact.
- Preserve `meta.country`, `meta.retrieved_at`, `meta.source`, and `meta.parameters` when citing findings. For `popularity`, expect `meta.country` to be null and read the queried countries from `meta.parameters.countries`.
- Treat `--limit` as an upper bound because Apple can return fewer records even when more matches exist; when at least N results are needed, use a modestly higher `--limit` (maximum 200) with `--local-limit N`, verify `meta.result_count >= N`, and use `meta.skipped_count` to distinguish Apple's short response (`0`) from malformed records discarded by `asapi` (`> 0`).
- Keep stderr separate from data and preserve nonzero exit status on failure.

## Respect request limits

- Treat `search`, `lookup`, and `popularity` as one conservative request budget. Apple documents the Search API at approximately 20 calls per minute, subject to change, and recommends caching search and lookup requests. `popularity` performs one lookup for each selected country: 9 for Tier 1 and 20 for Tier 2. Apple does not document the exact bucket key or explicitly confirm whether both routes share one enforced counter.
- Do not assume `reviews` and `chart` have separate capacity. Apple publishes no numeric limit for these RSS routes and does not document whether their limits are shared with each other or with Search API requests.
- Treat `iap` separately for planning because it reads an `apps.apple.com` product page, but do not assume unlimited or independent capacity; Apple publishes no numeric limit or bucket relationship for product pages.
- Successful responses do not reliably expose quota-remaining or reset headers. Treat a `429` and its `Retry-After` value as the authoritative runtime signal.
- Reduce calls before adding concurrency: batch known IDs in `lookup`, request only the needed search/chart limit and review pages, save reusable results, and avoid immediately repeating identical requests.
- Do not wrap commands in another retry loop. Let `asapi` handle transient failures and `Retry-After`. If it still exits nonzero, read stderr and follow its recommendation.

See Apple's [iTunes Search API guidance](https://performance-partners.apple.com/search-api) for the documented Search API limit and caching recommendation.

## Interpret fields

- Read normalized fields such as `app_id`, `name`, `developer_id`, `developer_name`, `primary_category`, `rating`, `rating_count`, and `app_store_url`.
- Treat `developer_name` as the public developer identity and `seller_name` as the legal seller; do not interchange them.
- Treat app `price` and `display_price` as the download price, not an in-app purchase price.
- Treat search `position` as response order for that query and country, not a universal keyword rank.
- Treat chart `rank` as a country-specific observation at `meta.retrieved_at`, not historical rank.
- Distinguish aggregate ratings and `rating_count` from the written reviews returned by `reviews`.
- Treat `popularity` records as ordered by descending `rating_count`, with unavailable or unrated storefronts last. Use `available` to distinguish an app missing from a storefront, and preserve null ratings instead of converting them to zero.
- Treat country-specific `rating_count` as a public relative-popularity signal only. Do not translate it into downloads, revenue, active users, or market share; rating behavior and rating resets can affect comparisons.
- State when optional fields are absent instead of filling or inferring them.

## Respect coverage limits

- Compare like-for-like countries and observation times.
- Describe reviews as a recent bounded sample. Apple exposes at most ten pages per country; `asapi` deduplicates multi-page results by review ID.
- Preserve repeated IAP names and formatted prices. Treat `purchases` as the country-specific selection displayed on the product page, not a complete catalog. Do not infer product IDs, types, billing periods, or subscription durations.
- Do not claim or estimate downloads, revenue, conversion rate, keyword search volume, or historical rank from `asapi` data.
