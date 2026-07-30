---
name: asapi
description: A muted technical research ledger for inspecting local App Store evidence.
colors:
  paper: "#f1efe8"
  paper-bright: "#faf9f4"
  paper-deep: "#dedcd4"
  paper-muted: "#d2d0c8"
  ink: "#20211e"
  ink-soft: "#4d504a"
  ink-faint: "#6c6f68"
  rule: "#8d8f88"
  rule-soft: "#c4c5bf"
  ochre: "#b49a50"
  ochre-wash: "#e0d3a9"
  olive: "#627563"
  oxide: "#965f55"
  oxide-wash: "#e1cbc6"
  slate: "#647181"
  shadow: "#777872"
typography:
  display:
    fontFamily: 'Arial, "Helvetica Neue", Helvetica, sans-serif'
    fontSize: "34px"
    fontWeight: 700
    lineHeight: 1.02
    letterSpacing: "-0.03em"
  headline:
    fontFamily: 'Arial, "Helvetica Neue", Helvetica, sans-serif'
    fontSize: "28px"
    fontWeight: 800
    lineHeight: 1
    letterSpacing: "-0.03em"
  title:
    fontFamily: 'Arial, "Helvetica Neue", Helvetica, sans-serif'
    fontSize: "13px"
    fontWeight: 800
    lineHeight: 1.2
    letterSpacing: "-0.01em"
  body:
    fontFamily: 'Arial, "Helvetica Neue", Helvetica, sans-serif'
    fontSize: "12px"
    fontWeight: 400
    lineHeight: 1.6
    letterSpacing: "normal"
  label:
    fontFamily: 'ui-monospace, "SFMono-Regular", Menlo, Consolas, monospace'
    fontSize: "9px"
    fontWeight: 700
    lineHeight: 1.2
    letterSpacing: "0.06em"
rounded:
  square: "0px"
spacing:
  xs: "4px"
  sm: "8px"
  md: "12px"
  lg: "18px"
  xl: "24px"
components:
  select-register:
    backgroundColor: "{colors.paper-bright}"
    textColor: "{colors.ink}"
    typography: "{typography.label}"
    rounded: "{rounded.square}"
    padding: "0 29px 0 9px"
    height: "36px"
  button-icon:
    backgroundColor: "{colors.paper}"
    textColor: "{colors.ink}"
    rounded: "{rounded.square}"
    size: "36px"
  app-row-active:
    backgroundColor: "{colors.ink}"
    textColor: "{colors.paper-bright}"
    rounded: "{rounded.square}"
    padding: "8px 14px"
  tab-peer:
    backgroundColor: "{colors.paper}"
    textColor: "{colors.ink-soft}"
    typography: "{typography.label}"
    rounded: "{rounded.square}"
    padding: "0 11px"
    height: "34px"
  tab-peer-active:
    backgroundColor: "{colors.ochre-wash}"
    textColor: "{colors.ink}"
    typography: "{typography.label}"
    rounded: "{rounded.square}"
    padding: "0 11px"
    height: "36px"
  panel-ruled:
    backgroundColor: "{colors.paper-bright}"
    textColor: "{colors.ink}"
    rounded: "{rounded.square}"
---

# Design System: asapi

## Overview

**Creative North Star: "The Working Research Ledger"**

asapi uses muted technical-ledger neobrutalism: warm paper fields, near-black ink, square geometry, ruled registers, compact technical labels, and scarce low-chroma signals. The interface should feel like maintained working evidence, not a generic SaaS dashboard or an analytics gallery.

The visual system serves an agent-populated research workflow with compact human controls for projects, apps, storefronts, and keywords. Dense information stays trustworthy through explicit storefront and freshness context, stable tabular alignment, and direct state language. Expressive detail is limited to registration-dot textures, hatched unavailable cells, and hard-offset depth where an object is genuinely actionable or lifted.

**Key Characteristics:**

- Warm paper surfaces separated by dark, mechanically precise rules.
- Arial for readable content and system monospace for metadata, counts, and state.
- Square controls and containers with compact, consistent working density.
- Overview, Keywords, and Reviews presented as equal working registers.
- Muted ochre, olive, oxide, and slate used as scarce semantic signals.
- Real App Store icons and flags retain their source color.

## Colors

The palette is paper-and-ink first; low-chroma signals annotate state without turning the workspace colorful.

### Primary

- **Registry Ochre** (`ochre`): marks ratings, selection cues, active refresh state, and the compact signal strip.
- **Ochre Wash** (`ochre-wash`): carries selected tabs, text selection, and restrained interactive hover emphasis.

### Secondary

- **Current Olive** (`olive`): communicates improving movement in dense evidence views.
- **Alert Oxide** (`oxide`): appears in the signal strip and anchors the error family.
- **Oxide Wash** (`oxide-wash`): backs inline and global error notices.
- **Technical Slate** (`slate`): a reserved secondary annotation color; it is not a general decorative accent.

### Neutral

- **Ledger Paper** (`paper`): default workspace and control surface.
- **Bright Sheet** (`paper-bright`): primary reading surface, active control face, and raised content.
- **Deep Paper** (`paper-deep`): rails, bands, headings, and recessed working areas.
- **Muted Paper** (`paper-muted`): inactive loader cells.
- **Ledger Ink** (`ink`): primary text, hard rules, focus outlines, and selected register rows.
- **Soft Ink** (`ink-soft`): supporting body copy and secondary interface text.
- **Faint Ink** (`ink-faint`): metadata and intentionally de-emphasized values.
- **Rule Gray** (`rule`): ordinary dividers, dotted separators, and texture.
- **Soft Rule** (`rule-soft`): low-priority row separation.
- **Offset Gray** (`shadow`): hard-offset depth only.

**The Paper Dominance Rule.** Neutral paper and ink own the screen; semantic colors annotate evidence and state rather than filling large regions.

**The Source Color Rule.** App icons and country flags remain unfiltered because they are evidence, not decoration.

## Typography

**Display Font:** Arial (with Helvetica Neue and Helvetica fallbacks)  
**Body Font:** Arial (with Helvetica Neue and Helvetica fallbacks)  
**Label/Mono Font:** UI monospace (with SFMono-Regular, Menlo, and Consolas fallbacks)

**Character:** The sans face keeps app names and review content plain and fast to read. Monospace turns counts, timestamps, storefront labels, table headers, and operational state into a consistent technical annotation layer.

### Hierarchy

- **Display:** reserved for instructional empty-state headlines.
- **Headline:** app identity; it truncates to protect controls and compacts on narrow screens.
- **Title:** panel and register headings.
- **Body:** descriptions and review text, generally constrained to comfortable reading widths.
- **Label:** compact metadata, statuses, counts, and column headers; usually uppercase and tracked.

**The Two-Voice Rule.** Use Arial for the subject matter and monospace for the system speaking about that subject.

**The Density Rule.** Hierarchy comes from weight, family, case, and rules—not oversized dashboard typography.

## Layout

The desktop shell fills the viewport inside a hard outer rule. A fixed 258px project/app register sits beside a flexible dossier; it narrows to 218px below 1040px. The dossier stacks identity, a ruled metadata strip, peer tabs, and one working panel. Overview content is centered up to 1100px with a compact 12px module rhythm; Keywords uses a creation register above a horizontally scrollable table with a sticky header; Reviews uses a two-column internal list scroller with bounded text areas for long entries.

At 760px and below, the document becomes one vertical page: the sidebar becomes a top register, apps become a horizontal strip, and the dossier releases fixed heights and list-level nested overflow. Reviews becomes one column and paginates against the browser viewport, while long review text keeps a small explicit internal scrollbar. The keyword register intentionally remains a bounded horizontal/vertical data viewport because its data structure is not collapsed. At 520px, identity controls stack, the metadata strip becomes one column, tabs remain horizontally scrollable, and popularity records become one column.

**The Context-Before-Content Rule.** Project, app, storefront, and freshness state remain ahead of the selected register at every width.

**The Review Scroll Rule.** The review list uses the document viewport on mobile and an internal list viewport on desktop; individual long review bodies remain bounded and expose a thin square scrollbar.

## Elevation & Depth

The system is flat by default. Depth is structural and uses crisp, blur-free offsets: 2px for compact controls, 3px for app identity and error notices, and 4px for the large empty-project instruction. Panels, table cells, review entries, metadata bands, and selected tabs remain flat and are separated by tone and rules.

### Shadow Vocabulary

- **Compact Control:** `2px 2px 0` with Offset Gray, for selectors and icon actions.
- **Identity Lift:** `3px 3px 0` with Offset Gray, for the selected app icon and global error.
- **Instruction Lift:** `4px 4px 0` with Offset Gray, for the main empty-project panel.

**The Earned Shadow Rule.** Add a hard offset only when a control can be pressed or a bounded object must sit above the ledger; never use ambient blur.

## Shapes

The form language is square. Controls, panels, chips, app icons, rating badges, and table cells use zero radius, straight edges, square line caps, and mitered joins. Outer shell and major boundaries use 2px ink rules; ordinary containers use 1px ink or gray rules; dotted rules divide secondary metadata.

Dotted registration fields may identify recessed working bands. Repeating diagonal hatching is reserved for unavailable storefront data. Machine state is written plainly rather than represented by colored status ornaments.

**The Hard Geometry Rule.** Do not round containers or soften rules; hierarchy must survive as a black-and-white wireframe.

## Components

### Project and storefront selectors

- Native selects sit inside 36px square ruled wrappers with monospace labels, a custom chevron, and compact hard-offset depth.
- Disabled selectors retain structure and reduce opacity.
- A 2px ink `:focus-visible` outline with a 2px offset is the shared keyboard treatment.

### Icon actions

- Refresh and external-link actions are 36px square buttons/links with a 1px ink border and compact hard offset.
- Hover uses Ochre Wash. Pressing translates the face into its shadow; disabled refresh reduces opacity and removes affordance through the native disabled state.
- Refresh rotation is stepped rather than fluid. Reduced-motion preference effectively removes animation and transition duration.

### Project/app register

- App rows are full-width ruled buttons with a 36px icon, truncated name, and compact storefront/rating metadata.
- The selected row reverses to Ledger Ink with Bright Sheet text and gains one small ochre square marker.
- On mobile, the same rows become fixed-width peers in a horizontally scrolling register; they do not turn into cards.

### Working tabs

- Overview, Keywords, and Reviews are semantic peers in one `tablist`; no tab is visually or behaviorally subordinate.
- The selected tab rises by 2px, uses Ochre Wash, and bridges the bottom rule. Optional counts remain small ruled badges.
- Exactly one tab is in the tab order. Left/Right arrows wrap through the tabs; Home and End move to the first and last tab. Activation moves focus and content together.
- Each active panel is labelled by its tab and can receive focus.

### Ruled panels and registers

- Overview sections use flat Bright Sheet panels with an ink border, Deep Paper heading band, and tight internal grids.
- Keyword rows use sticky monospace headers, alternating paper tones, tabular positions, square sparklines, and Ochre Wash on hover. Preserve the full data table with horizontal scrolling instead of stacking fields into cards.
- Review entries form a ruled two-column register on desktop and one column on mobile. Rating badges use the ochre family; metadata is separated by a dotted rule.

### Loading, error, and empty states

- Loading uses three small ruled cells with stepped opacity; visible status text remains present when the loader communicates a named task.
- Errors use Oxide Wash with dark oxide text. Review pagination errors are `role="alert"` and include a square hard-offset Retry action; ongoing review loading is `role="status"`.
- Empty states explain the missing evidence directly and point to the nearby creation control.
- No-data cells and facts use an em dash; unavailable storefront cards use restrained hatching and an explicit label.

## Do's and Don'ts

### Do:

- **Do** preserve explicit project, app, storefront, freshness, and refresh state.
- **Do** use paper tone, 1px/2px rules, alignment, and typography before adding depth or color.
- **Do** keep Overview, Keywords, and Reviews as peer tabs with the shipped keyboard model.
- **Do** keep mobile review pagination on the document scroll while bounding unusually long review bodies.
- **Do** keep real App Store imagery truthful and source-colored.
- **Do** respect a 320px minimum viewport, visible keyboard focus, readable contrast, and reduced-motion preferences.

### Don't:

- **Don't** introduce rounded card stacks, glass, glow, ambient shadows, or atmospheric gradients.
- **Don't** turn the workspace into a marketing surface with oversized display type or invented metrics.
- **Don't** use ochre, olive, oxide, or slate as broad decorative fills.
- **Don't** hide country or freshness context when it changes the meaning of evidence.
- **Don't** collapse the keyword register into unrelated mobile cards.
- **Don't** import mock-only controls, search, pagination chrome, or status claims that the shipped application does not implement.
