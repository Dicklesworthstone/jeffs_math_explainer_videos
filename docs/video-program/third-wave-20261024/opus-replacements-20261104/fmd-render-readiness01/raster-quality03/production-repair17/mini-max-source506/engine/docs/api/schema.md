<!--
The API schema summary (§16.2).

GENERATED from API_SCHEMA.tsv + API_OVERLAY.tsv by
fmn_conformance::schema — regenerate, never hand-edit.
Reference pin: 6199a00d4c1b1127ebe45cb629c3f22538b10e13

Regenerate:  UPDATE_API_ARTIFACTS=1 cargo test -p fmn-conformance \
                 --test api_schema
-->

# The one API schema

Generated from `API_SCHEMA.tsv` (extracted from the pinned Reference) and `API_OVERLAY.tsv` (authored). Reference pin `6199a00d4c1b1127ebe45cb629c3f22538b10e13`.

## Surface inventory

| Kind | Total | Wildcard-exported |
|---|---|---|
| class | 257 | 246 |
| method | 1449 | 0 |
| property | 5 | 0 |
| attribute | 105 | 0 |
| function | 220 | 185 |
| constant | 166 | 161 |
| leaked_import | 73 | 73 |

`from manimlib import *` binds 663 unique names from 665 wildcard-exported schema rows. The 2 duplicate rows are `DEFAULT_DOT_RADIUS`, `EPSILON`. The Reference declares no `__all__` (§1.6), so the unique-name count is the *computed* wildcard closure, leaked third-party imports included — enumerating it is the only way to know what the surface actually is.

## Parity Ledger coverage

The single Ledger contains 2495 rows: 2275 Python symbols, 34 Reference CLI flags, 29 FrankenManim-native CLI flags, 4 CLI commands, and 153 config keys. Its reviewed-identity ratchet is `8b78fb1c5f1073b01966f02e1be246b25deb12f9cf21dfe0a1adeed557aa7bf7`.

## Semantic tiers (§16.1)

| Status | Ledger rows |
|---|---|
| same | 2038 |
| improved | 183 |
| tiered | 92 |
| excluded | 182 |
| unreviewed | 0 |

`unreviewed` is the honest default for a surface nobody has adjudicated yet; it is the number the Parity Ledger ratchets down. Every improved row resolves to a Behavior Note; every tiered or excluded row resolves to `docs/api/out_of_tier.tsv`.

## Canonical names (Appendix C, C-9)

26 symbols and 2 parameters carry a public-surface typo. The Rust front door and these docs use the canonical name; `fmn-python` binds both, so source-unedited scenes keep working.

| Reference | Canonical |
|---|---|
| `manimlib.event_handler.event_dispatcher:EventDispatcher.add_listner` | `add_listener` |
| `manimlib.event_handler.event_dispatcher:EventDispatcher.get_listners_count` | `get_listeners_count` |
| `manimlib.event_handler.event_dispatcher:EventDispatcher.remove_listner` | `remove_listener` |
| `manimlib.mobject.geometry:Arrow.tickness_multiplier` | `thickness_multiplier` |
| `manimlib.mobject.mobject:Mobject.add_event_listner` | `add_event_listener` |
| `manimlib.mobject.mobject:Mobject.add_key_press_listner` | `add_key_press_listener` |
| `manimlib.mobject.mobject:Mobject.add_key_release_listner` | `add_key_release_listener` |
| `manimlib.mobject.mobject:Mobject.add_mouse_drag_listner` | `add_mouse_drag_listener` |
| `manimlib.mobject.mobject:Mobject.add_mouse_motion_listner` | `add_mouse_motion_listener` |
| `manimlib.mobject.mobject:Mobject.add_mouse_press_listner` | `add_mouse_press_listener` |
| `manimlib.mobject.mobject:Mobject.add_mouse_release_listner` | `add_mouse_release_listener` |
| `manimlib.mobject.mobject:Mobject.add_mouse_scroll_listner` | `add_mouse_scroll_listener` |
| `manimlib.mobject.mobject:Mobject.clear_event_listners` | `clear_event_listeners` |
| `manimlib.mobject.mobject:Mobject.get_event_listners` | `get_event_listeners` |
| `manimlib.mobject.mobject:Mobject.get_family_event_listners` | `get_family_event_listeners` |
| `manimlib.mobject.mobject:Mobject.get_has_event_listner` | `get_has_event_listener` |
| `manimlib.mobject.mobject:Mobject.init_event_listners` | `init_event_listeners` |
| `manimlib.mobject.mobject:Mobject.remove_event_listner` | `remove_event_listener` |
| `manimlib.mobject.mobject:Mobject.remove_key_press_listner` | `remove_key_press_listener` |
| `manimlib.mobject.mobject:Mobject.remove_key_release_listner` | `remove_key_release_listener` |
| `manimlib.mobject.mobject:Mobject.remove_mouse_drag_listner` | `remove_mouse_drag_listener` |
| `manimlib.mobject.mobject:Mobject.remove_mouse_motion_listner` | `remove_mouse_motion_listener` |
| `manimlib.mobject.mobject:Mobject.remove_mouse_press_listner` | `remove_mouse_press_listener` |
| `manimlib.mobject.mobject:Mobject.remove_mouse_release_listner` | `remove_mouse_release_listener` |
| `manimlib.mobject.mobject:Mobject.remove_mouse_scroll_listner` | `remove_mouse_scroll_listener` |
| `manimlib.mobject.numbers:char_to_cahced_mob` | `char_to_cached_mob` |

## Config keys

153 keys, 142 of them shared with the Reference's `default_config.yml`. Every one is bound to a Rust field by `API_OVERLAY.tsv`, and the binding generates `crates/fmn-config/src/generated.rs`.

