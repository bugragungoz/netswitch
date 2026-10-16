# What netSwitch adds to Bugra

Bugra was drawn for a website. netSwitch reuses its tokens, fonts and components (copied from the
owner's component-inventory app, which builds them from the design system) and adds the pieces a
live network monitor needs. Each one is built from Bugra tokens and shapes and is marked "(added)" in
the code.

## Tokens (`src/styles/added-tokens.css`)

| Token | Dark | Light | Why |
|---|---|---|---|
| `--series-down` | `--accent-ink` | `--accent-ink` | Download, the series people watch, wears the one accent. |
| `--series-up` | `#a29b91` | `#8a8278` | Upload is a quiet warm gray from the neutral ramp, not a second hue. |
| `--series-down-fill` | accent at 12 % | accent at 10 % | The area under the download line. |
| `--series-up-fill` | gray at 8 % | gray at 10 % | The area under the upload line. |

The pair was checked with a palette validator against each theme's surface: color-vision-deficiency
separation 11.3 (dark) and 12.5 (light), normal-vision separation 20.4 and 18.7, and both lines at
least 3:1 against the surface. The chart also has a legend, so the series never depend on color
alone.

`npm run check:colors` fails on a color value anywhere else in `src`.

## Components

| Piece | Where | Notes |
|---|---|---|
| Button `quiet` and `danger` | `components/Button.tsx` | Quiet has no outline until hovered; danger uses the `warn` outline and text. |
| Badge | `components.css` | A small outlined pill for counts and states ("2 rules", "Local network"). |
| Empty state | `components/Misc.tsx` | Icon, sentence and optional actions, centered in the view. |
| Segmented control | `pages/Settings.tsx`, `.tabs` | A pill group with `role="radiogroup"`; the chosen option wears the accent tint. |
| Snackbar stack | `components/Snackbars.tsx` | Bugra's snackbar, stacked at the bottom left with a close button. |
| Stat tile | `components/Misc.tsx` | Icon, label, one large number and a muted line under it. |
| Traffic chart | `components/TrafficChart.tsx` | Two lines on one axis, 2 px strokes, a legend, a crosshair with a tooltip, and arrow keys, Home, End and Escape when it has focus. The compact form in the app panel has no axes. |
| Share bar | `pages/Overview.tsx` | A thin accent bar under each busiest app and host, as a share of the busiest one; the number beside it carries the value. |
| App avatar | `components/Misc.tsx` | The first letter of the app in a bordered square; Windows icons are not read. |
| Detail panel | `layout.css` | Sits beside the table, floats over it (with the floating-layer shadow) below 1100 px. |
| Status bar | `App.tsx`, `layout.css` | Measuring state, demo badge, problems, live totals, version and the GitHub link. |

## Icons (`components/icons.ts`)

Bugra-style line icons on a 24 px grid, 1.6 px stroke with round caps. netSwitch adds `activity`,
`apps`, `link`, `shield`, `shieldOff` and `globe`, the switch mark beside the name in the
sidebar (`App.tsx`), and the GitHub mark from Primer Octicons (MIT) for the link in the status bar.
