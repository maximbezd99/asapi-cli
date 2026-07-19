---
name: asapi
description: Research public Apple App Store apps, developers, metadata, prices, in-app purchases, ratings, written reviews, charts, competitors, search results, categories, and country comparisons with the asapi CLI. Use for current, country-specific App Store research and comparisons; do not use it to infer downloads, revenue, conversion, keyword volume, or historical rank.
---

# Research the public App Store with asapi

Use `asapi` for read-only, country-specific public App Store research. It requires no App Store Connect credentials.

Before research, run `command -v asapi`. If it is missing, stop and state that the required CLI is unavailable. Do not fall back to raw HTTP.

## Discover commands

- Run `asapi --help` to see the current command set.
- Run `asapi <command> --help` before using unfamiliar options.
- Use `asapi list countries`, `asapi list categories`, or `asapi list chart-types` instead of guessing supported values.

## Choose the command

| Research need | Command |
| --- | --- |
| Find apps by name or keyword | `asapi search "term" --country us --limit 10` |
| Get normalized app metadata | `asapi lookup 123456789 987654321 --country us` |
| Inspect displayed in-app purchases | `asapi iap 123456789 --country us` |
| Sample recent written reviews | `asapi reviews 123456789 --country us --pages 3` |
| Observe a free apps chart | `asapi chart free --country us --limit 25 --category 6007` |

Resolve uncertain app identities with `search`, then use the returned `app_id` with `lookup`, `iap`, or `reviews`. Batch multiple known IDs in one `lookup` call.

## Handle output

- Read stdout as JSON. Research commands return `data` and `meta`; `list` commands return a raw array.
- Keep compact JSON for automation. Add `--pretty` only for human inspection.
- Use `--output-file <path>` when the result must be saved as an artifact.
- Preserve `meta.country`, `meta.retrieved_at`, `meta.source`, and `meta.parameters` when citing findings.
- Keep stderr separate from data and preserve nonzero exit status on failure.

## Respect request limits

- Treat `search` and `lookup` as one conservative request budget. Apple documents the Search API at approximately 20 calls per minute, subject to change, and recommends caching both search and lookup requests. Apple does not document the exact bucket key or explicitly confirm whether both routes share one enforced counter.
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
- State when optional fields are absent instead of filling or inferring them.

## Respect coverage limits

- Compare like-for-like countries and observation times.
- Describe reviews as a recent bounded sample. Apple exposes at most ten pages per country; `asapi` deduplicates multi-page results by review ID.
- Preserve repeated IAP names and formatted prices. Treat `purchases` as the country-specific selection displayed on the product page, not a complete catalog. Do not infer product IDs, types, billing periods, or subscription durations.
- Do not claim or estimate downloads, revenue, conversion rate, keyword search volume, or historical rank from `asapi` data.
