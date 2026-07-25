# CRITIQUE.md — Tutorial Review

A critique of the "Node Graph Editor in Rust" tutorial at
https://danpokhrel.github.io/tutorial/, produced while building the editor
end-to-end against the actual `dear-app` / `dear-imgui-rs` / `dear-imnodes`
0.15.1 crates.

Each issue is listed under the **chapter** where it appears, with the
**problem**, the **fix applied** in this project, and **suggested tutorial
improvements**.

---

## Summary

| Severity | Count | Description |
|----------|-------|-------------|
| Critical (won't compile) | 5 | Wrong API signatures, nonexistent types, broken ownership |
| Incorrect/misleading | 3 | Factual errors about crate APIs |
| Minor | 4 | Redundancy, naming, filter mismatch |
| Missing functionality | 3 | Fullscreen canvas, dock tab removal, crisp fonts, file-tree + side panels |

**Verdict:** The tutorial is well-structured and pedagogically sound — the
chapter progression, Rust Book references, data/UI separation, and borrow-pattern
explanations are genuinely good. However, it appears to have been written
without verifying the code against the real crate APIs. Every chapter from 6
onward contains at least one compilation-blocking error, and it omits practical
basics a real editor needs (fullscreen canvas, no dock chrome, a proper font,
side panels for project navigation and property editing). A reader following
along blindly would hit a wall at Chapter 6 and likely not recover without
reading crate source.

---

## Chapter 01 — Introduction

No issues. The chapter list, tool descriptions, and Rust Book references are
accurate and well-motivated.

---

## Chapter 02 — What is dear-imgui-rs?

**Issue: `AddOns::imnodes` is not just an `Option<()>` presence flag**

The tutorial states:

> "`AddOns::imnodes` is a `Option<()>` presence flag — not the actual `Context`
> or `EditorContext`."

This is only true when the `imnodes` feature is **not** enabled. When it **is**
enabled (which the tutorial's own `Cargo.toml` does), the actual definition is:

```rust
#[cfg(feature = "imnodes")]
pub imnodes: Option<&'a imnodes::Context>,
```

So `AddOns::imnodes` is `Option<&imnodes::Context>` — a reference to the real
ImNodes context, managed by `dear-app`. The tutorial uses this claim to justify
creating ImNodes contexts manually in `App::new()`. While creating your own
contexts is valid, it is not necessary — you could use `addons.imnodes.unwrap()`
from the `on_frame` closure instead.

**Fix applied:** Kept the manual-context approach (it's clearer for a tutorial)
but corrected the note in code comments.

**Suggestion:** Correct the note to say: "When the `imnodes` feature is enabled,
`AddOns::imnodes` provides a `&imnodes::Context`. You can use this directly, or
create your own `Context` / `EditorContext` for finer control (e.g., multiple
independent editors)."

---

## Chapter 03 — Project Setup & Structure

**Issue: Dual `lib.rs` + `main.rs` module declarations**

The tutorial's project structure lists both `src/lib.rs` (declaring
`pub mod graph; pub mod ui; ...`) and `src/main.rs` (declaring `mod app; mod
graph; mod theme; mod ui;`). Having both creates two separate module trees — the
binary's and the library's — which is redundant for a binary-only application and
can confuse beginners.

**Fix applied:** Used binary-only structure (modules declared in `main.rs`, no
`lib.rs`). The `crate::` paths work identically.

**Suggestion:** For a tutorial, pick one: either a library + binary (where
`main.rs` does `use imgui_tutorial::...` to reference the library crate), or a
pure binary. Don't declare the same modules in both `lib.rs` and `main.rs`.

---

## Chapter 04 — Your First Window

**Issue: `DockingConfig::default()` already enables docking**

The tutorial explicitly sets:

```rust
docking: DockingConfig { enable: true, auto_dockspace: true, ..Default::default() },
```

But `DockingConfig::default()` already has `enable: true` and
`auto_dockspace: true`. This is redundant (harmless, but misleading — implies
docking is off by default).

**Fix applied:** Set `enable: false` because a single-canvas editor should not
use the dockspace (see Chapter 04 below for the dock-tab issue).

**Suggestion:** Either omit the explicit `docking` field, or add a note that
docking is enabled by default and the explicit config is for clarity / to show
where customization would go.

**Issue: Docking leaves a dockspace tab / title bar on the node window**

The tutorial enables docking + `auto_dockspace` in every `main.rs`. With docking
on, the "Node Graph Editor" window is a dockable window that shows a tab/title
bar (the "dropdown tab") and does not fill the host. For a single-purpose node
editor that should own the whole window, the dockspace adds chrome the user does
not want.

**Fix applied:** Disabled docking entirely and used a borderless fullscreen
window (see Chapter 06 for the viewport-work-area technique):

```rust
docking: DockingConfig { enable: false, ..Default::default() },
```

**Trade-off:** This removes the ability to dock other panels (e.g. an inspector)
as tabs. For this editor, side panels (see Chapter 06 below) are a better fit
than a dockspace.

**Suggestion:** Don't enable docking by default in a tutorial whose only window
is a fullscreen canvas. Either defer docking to a later "multi-panel" chapter, or
show how to dock the main window into the central node and hide its tab bar
(`DockNodeFlags::NO_TAB_BAR`).

---

## Chapter 05 — Application Architecture

No compilation issues in this chapter. The data model (`graph/model.rs`), the
`GraphState` with ID allocation, and the module wiring are all correct and
well-designed.

**Note:** `GraphState::remove_links_for_pin` is included in the tutorial but
never called by any tutorial code — `remove_node` reimplements the same `retain`
logic inline. This triggers a dead-code warning. The clean fix is to have
`remove_node` call `remove_links_for_pin` per pin instead of duplicating the
logic, which both removes the warning and reduces duplication.

---

## Chapter 06 — Introducing the Node Editor

**Issue (Critical): `on_frame` closure has the wrong number of arguments**

The tutorial's `AppBuilder` example uses a single-argument closure:

```rust
.on_frame(move |ui| {            // ← one argument
    if let Some(ref mut a) = app {
        crate::ui::render(ui, a);
    }
})
```

The actual `dear-app` 0.15.1 API signature is:

```rust
pub fn on_frame<F: FnMut(&imgui::Ui, &mut AddOns) + 'static>(mut self, f: F) -> Self
```

The frame callback receives **two** arguments: `(&Ui, &mut AddOns)`.

**Fix applied:** Use `.on_frame(move |ui, _addons| { ... })`.

**Suggestion:** The `run` function (chapters 3–4) correctly uses two arguments
`|ui, _addons|`, so the inconsistency is likely an oversight when the tutorial
switched to `AppBuilder`. Double-check all `on_frame` examples.

---

**Issue (Critical): `Option<App>` shared between `on_setup` and `on_frame` doesn't compile**

The tutorial stashes the app state in a plain `Option`:

```rust
let mut app: Option<App> = None;

AppBuilder::new()
    .on_setup(|ctx| {              // ← borrows `app`
        app = Some(App::new(ctx));
    })
    .on_frame(move |ui| {          // ← moves `app`
        if let Some(ref mut a) = app { ... }
    })
    .run()
```

Both `on_setup` and `on_frame` require `'static` closures. `on_setup` borrows
`app` (to write to it), while `on_frame` moves `app`. The borrow checker rejects
this: `on_setup`'s borrow must be `'static`, but `app` is a stack local that
`on_frame` also needs to own.

**Fix applied:** Use interior mutability — `Rc<RefCell<Option<App>>>` — with a
cloned handle for each closure:

```rust
let app: Rc<RefCell<Option<App>>> = Rc::new(RefCell::new(None));

AppBuilder::new()
    .on_setup({
        let app = Rc::clone(&app);
        move |ctx| { *app.borrow_mut() = Some(App::new(ctx)); }
    })
    .on_frame({
        let app = Rc::clone(&app);
        move |ui, _addons| {
            if let Some(a) = app.borrow_mut().as_mut() {
                crate::ui::render(ui, a);
            }
        }
    })
    .run()
```

**Suggestion:** This is the single biggest blocker. The `Option<App>` pattern is
presented as "the standard `AppBuilder` pattern in 0.15.x," but it cannot work as
written. Either fix the example to use `Rc<RefCell<>>` (or `Arc<Mutex<>>` if
`Send` is needed), or document an alternative approach (e.g., lazy-init in the
first `on_frame` call).

---

**Issue: Node graph window does not fill the OS window**

The tutorial's `render_editor` creates the node window with a fixed fallback size
and only-on-first-use positioning:

```rust
ui.window("Node Graph Editor")
    .size([1000.0, 700.0], Condition::FirstUseEver)
    .position([40.0, 60.0], Condition::FirstUseEver)
    .build(|| { ... });
```

`FirstUseEver` applies the size once and then never again, so the canvas does
*not* track OS-window resizes — resizing the OS window leaves the editor canvas
at its original size.

**Fix applied:** Read the main viewport's **work area** (which already excludes
the main menu bar) and pin position/size every frame with `Condition::Always`,
using borderless flags so there is no title bar/scrollbar fighting the layout:

```rust
let viewport = ui.main_viewport();
let work_pos = viewport.work_pos();
let work_size = viewport.work_size();
ui.window("Node Graph Editor")
    .flags(WindowFlags::NO_DECORATION | WindowFlags::NO_MOVE)
    .position([work_pos[0], work_pos[1]], Condition::Always)
    .size([work_size[0], work_size[1]], Condition::Always)
    .build(|| { ... });
```

Because this runs every frame with `Always`, the canvas resizes in lockstep with
the OS window. `work_pos`/`work_size` (not `pos`/`size`) keep it below the menu
bar.

**Suggestion:** The tutorial should include a "full-window editor" recipe early
(Chapter 6). It is one of the first things a reader will want, and the
`viewport.work_pos()`/`work_size()` API is never mentioned anywhere in the
tutorial despite being the canonical ImGui pattern for this.

---

**Issue: No side panels (file tree / properties inspector)**

A real node editor needs at least a project panel (to browse the project's
files) and a properties panel (to edit the selected node's title and pins). The
tutorial never builds these, leaving the editor as a single canvas with no way
to navigate files or edit node metadata outside of context menus.

**Fix applied:** Added a three-column layout computed every frame from the
viewport work area:

```text
┌────────────┬───────────────────────┬─────────────┐
│  Project   │     Node Graph        │ Properties  │
│  (240px)   │     (flexible)        │  (300px)    │
│            │                       │             │
└────────────┴───────────────────────┴─────────────┘
```

- **Project panel** (left, `src/ui/panels.rs::render_project_panel`): a file
  tree of the project directory. The tree is scanned once and cached in
  `app.ui.file_tree` (`src/ui/file_tree.rs::FileEntry`), with a Refresh button
  to rescan on demand. Directories render as expandable `tree_node`s (root open
  by default); files render as leaf nodes that set `app.ui.selected_file` when
  clicked. Hidden entries and the `target` build-output directory are skipped,
  and entries are sorted directories-first then alphabetically.
- **Properties panel** (right, `render_properties_panel`): edits the selected
  node's title (`input_text`) and each pin's label. Falls back to a hint when no
  node is selected (including minimap-hover info).
- Both panels use `NO_TITLE_BAR | NO_RESIZE | NO_MOVE | NO_COLLAPSE` (not
  `NO_DECORATION`, which also removes scrollbars) so they stay pinned and fill
  the window vertically like the canvas, while long file trees / pin lists can
  scroll.
- Canvas-click selection: left-clicking a node in the canvas syncs
  `app.ui.selected_node` so the properties panel tracks the canvas. Clicking
  empty space clears the selection.

**Design note on caching:** The file tree is a snapshot, not a live view —
scanning the filesystem every frame (60+ fps) would be wasteful and could
stutter on large directories. The tree is built once on first render and only
refreshed when the user clicks Refresh. This is the standard pattern for
filesystem-backed UI in an immediate-mode context.

**Suggestion:** Add a chapter (after Chapter 8, "Links & Connections") that
builds a file-tree project panel and a properties inspector. The file tree
introduces two patterns worth teaching: (1) caching expensive data (filesystem
scans) outside the per-frame loop, and (2) recursive `tree_node` rendering with
`LEAF` flags for files vs. expandable nodes for directories.

---

## Chapter 07 — Building Nodes & Pins

No compilation issues. The node rendering pattern, pin shapes, and initial
positioning are all correct.

**Note:** The borrow-pattern discussion (collect-then-mutate) is accurate and
well-explained. The `classify_link_pins` helper introduced here returns
`(Option<PinId>, Option<PinId>)`, which is awkward — the caller must destructure
and double-check. Returning `Option<(PinId, PinId)>` is cleaner and more
idiomatic.

**Fix applied:** Changed to `Option<(PinId, PinId)>`.

---

## Chapter 08 — Links & Connections

**Issue (Critical): Test helper has a double-mutable-borrow error**

The `make_test_graph()` test helper (which appears in Chapter 12's tests but
originates from the link-handling logic introduced here) writes:

```rust
g.add_link(Link {
    id: g.next_link_id(),   // ← second mutable borrow of `g`
    from: p1_out,
    to: p2_in,
});
```

`g.add_link(...)` takes `&mut self`, and `g.next_link_id()` also takes `&mut
self`. You cannot have two mutable borrows of `g` in the same expression.

**Fix applied:** Allocate the ID first:

```rust
let link_id = g.next_link_id();
g.add_link(Link { id: link_id, from: p1_out, to: p2_in });
```

**Suggestion:** This is a basic borrow-checker error that a beginner following
the tutorial would be confused by. The tutorial even discusses the
"collect-then-mutate" pattern in Chapter 7 but doesn't apply it consistently in
its own test code.

---

## Chapter 09 — Modern Styling & Theming

**Issue (Critical): `ColorElement::NodeBorder` does not exist**

The tutorial's `EditorTheme::apply()` method calls:

```rust
editor.set_color(ColorElement::NodeBorder, self.node_border);
```

The `dear-imnodes` 0.15.1 `ColorElement` enum has no `NodeBorder` variant. The
correct name is `ColorElement::NodeOutline`.

**Fix applied:** Replaced `NodeBorder` with `NodeOutline`.

**Suggestion:** The tutorial should list the actual `ColorElement` variants
(there are ~25) or link to the crate docs. An AI-generated enum variant name is
a common hallucination — always verify against `docs.rs` or the crate source.

---

**Issue (Critical): Theme cannot be applied in `App::new()`**

The tutorial applies the theme inside `App::new()`:

```rust
impl App {
    pub fn new(imgui_context: &mut dear_imgui_rs::Context) -> Self {
        let nodes_context = imnodes::Context::create(imgui_context);
        let editor_context = nodes_context.create_editor_context();
        let theme = crate::theme::EditorTheme::dark();

        let editor = imnodes::editor(&nodes_context, Some(&editor_context));
        theme.apply(&editor);
        editor.end();
        // ...
    }
}
```

`imnodes::editor(...)` is **not a free function**. The `editor()` method lives on
`NodesUi`, which is created via `ui.imnodes(ctx)` — and that requires a `&Ui`
reference. In `App::new()` (called from `on_setup`), only `&mut imgui::Context`
is available, not a `Ui`.

**Fix applied:** Defer theme application to the first editor frame in
`render_editor()`, using a `theme_applied: bool` flag:

```rust
if !app.ui.theme_applied {
    app.theme.apply(&editor);
    app.ui.theme_applied = true;
}
```

The style setters write to the `EditorContext`'s persistent state, so applying
once on the first frame suffices.

**Suggestion:** The tutorial's claim that "style setters are persistent" is
correct, but the code to apply them at construction time is not achievable with
the available API. Either show the per-frame-deferred approach, or explain how
to create a one-shot `Ui` from a `Context` (e.g., `imgui.frame()`) — though that
may interfere with the runner's own frame management.

---

## Chapter 10 — Interactions & UX

**Issue (Critical): `Io` mouse/keyboard state is accessed via methods, not fields**

The tutorial accesses input state as if they were public fields:

```rust
ui.io().mouse_clicked[MouseButton::Right as usize]   // ❌ no such field
ui.io().mouse_pos                                      // ❌ not a field
ui.io().key_ctrl                                       // ❌ not a field
```

In `dear-imgui-rs` 0.15.1, these are **methods**:

```rust
ui.is_mouse_clicked(MouseButton::Right)   // ✓ method on Ui
ui.io().mouse_pos()                        // ✓ method returning [f32; 2]
ui.io().key_ctrl()                         // ✓ method returning bool
```

There is no `mouse_clicked` array on `Io` at all. Mouse click detection is done
via `Ui::is_mouse_clicked(MouseButton)`.

**Fix applied:** See corrected calls above. Used throughout `editor.rs` and
`panels.rs`.

**Suggestion:** The `ui.io().mouse_clicked[...]` pattern looks like it was
copied from C++ ImGui's `io.MouseDown[]` / `io.MouseClicked[]` arrays. The Rust
bindings use strongly-typed methods instead. The tutorial should verify input
APIs against the actual `Io` struct.

---

**Issue: `minimap_with_callback` needs the collect-then-mutate pattern**

The tutorial's minimap-with-callback example (Chapter 10) writes to a local
inside the callback, which is correct, but it doesn't address the borrow
conflict that arises when the callback captures a `&mut` local that you also want
to move into a struct afterwards. The callback closure borrows the local for its
entire lifetime.

**Fix applied:** Use `Cell<Option<NodeId>>` (since `NodeId` is `Copy`) so the
callback writes via `.set()` without a mutable borrow, and read it via `.get()`
after `editor.end()`:

```rust
let minimap_hovered: Cell<Option<NodeId>> = Cell::new(None);
editor.minimap_with_callback(0.25, MiniMapLocation::BottomRight, |node_id| {
    minimap_hovered.set(Some(NodeId(node_id.raw())));
});
// ... after editor.end():
app.ui.minimap_hovered = interactions.minimap_hovered;
```

**Suggestion:** Mention that `Cell`/`RefCell` is the idiomatic escape hatch when
a callback needs to write to captured state without a mutable borrow that
outlives the callback.

---

## Chapter 11 — State Persistence

No compilation issues. The two-layer persistence (ImNodes INI for layout, serde
JSON for graph structure) is well-designed. The "pending action" pattern for
deferred save/load during a frame is correct.

**Issue: `tracing` env filter doesn't match crate name**

This appears in Chapter 12's `main.rs` but originates from the logging setup
implied by Chapter 11's production framing. The tutorial uses
`.with_env_filter("node_editor=debug")`, assuming the crate is named
`node-editor`. If a reader's crate has a different name (e.g., `imgui-tutorial`),
the filter won't match and no log output appears — silently.

**Fix applied:** Use the actual crate name: `.with_env_filter("imgui_tutorial=debug")`.

**Suggestion:** Use `env!("CARGO_PKG_NAME").replace('-', "_")` or just note that
the filter must match the crate name.

---

## Chapter 12 — Production Architecture

**Issue: Feature flags in `Cargo.toml` lack `cfg` guards in code**

The tutorial's production `Cargo.toml` makes `dear-imnodes`, `serde`, and
`serde_json` optional with feature flags (`imnodes`, `serde-support`), but
`serde-support` is not in the `default` feature set. The code uses
`serde::{Serialize, Deserialize}` and `serde_json` unconditionally without
`#[cfg(feature = "serde-support")]` guards, so building without `serde-support`
(the default) would fail.

**Fix applied:** Kept all dependencies non-optional (matching Chapter 3's simpler
`Cargo.toml`). The feature-flag architecture is a valid production pattern but
requires `cfg` guards throughout the code, which the tutorial doesn't show.

**Suggestion:** Either add `serde-support` to `default`, or show the `#[cfg]`
guards needed in `model.rs` and `app.rs` to make the feature flags actually work.

---

**Issue: Custom error type has unused variants**

The tutorial's `AppError` enum includes `ImGui(String)` and `Graph(String)`
variants that are never constructed anywhere in the tutorial code, triggering
dead-code warnings.

**Fix applied:** Annotated the variants with `#[allow(dead_code)]` — they are
part of the designed public error API and may be constructed by future call
sites, which is a common and accepted Rust practice for library-style error
enums.

**Suggestion:** Either wire the variants up (e.g., use `AppError::ImGui` for the
dear-app runner error instead of `AppError::Init`), or add a note that
`#[allow(dead_code)]` is acceptable for a designed-but-not-yet-used error API.

---

**Issue: Default font is pixelated (no font-loading guidance)**

The tutorial (Chapters 2 and 12) never loads a font. dear-imgui-rs therefore
falls back to Dear ImGui's default **ProggyClean** bitmap font, which is a
pixel-art font that looks blocky/pixelated, especially on HiDPI (Retina) displays.
The tutorial mentions `on_fonts` only in a comment in Chapter 2's lifecycle list
and never shows how to use it.

Two compounding factors make text look bad out of the box:
1. ProggyClean is a bitmap font (inherently pixelated-looking), not a vector TTF.
2. The wgpu renderer maps logical pixels to the physical framebuffer via
   `display_framebuffer_scale` (2.0 on Retina). The font atlas is rasterized at
   logical resolution, so glyphs are upscaled → soft.

**Fix applied:** Bundled a modern open-source sans-serif (Inter, OFL) under
`assets/`, loaded it via `AppBuilder::on_fonts`, and made it the default by
clearing the atlas first:

```rust
.on_fonts(|ctx| {
    let font_data = include_bytes!("../assets/Inter-Regular.ttf");
    let mut atlas = ctx.fonts();
    atlas.clear_fonts();
    let cfg = dear_imgui_rs::FontConfig::new().rasterizer_density(2.0);
    atlas.add_font_from_memory_ttf(font_data, 18.0, Some(&cfg), None);
})
```

`rasterizer_density(2.0)` is the key trick: it rasterizes glyphs at 2×
resolution *without* changing the on-screen size, so text stays crisp on HiDPI
displays without needing to read the exact scale factor — which matters here
because `dear-app` calls `on_fonts` **before** `WinitPlatform::attach_window`
sets `display_framebuffer_scale`, so the scale is not yet available inside
`on_fonts`.

**Known limitation:** For *perfect* 1:1-texel crispness on Retina you could
instead rasterize at `size_pixels * framebuffer_scale` and set
`font_global_scale = 1.0 / framebuffer_scale` — but that requires the scale
factor at load time, which `dear-app`'s callback ordering does not expose before
the atlas is built. `rasterizer_density(2.0)` is a robust approximation that
looks crisp on both 1× and 2× displays.

**Suggestion:** The tutorial should ship a "Fonts" section:
- Explain that the default font is a pixel bitmap and looks bad.
- Show `on_fonts` + `add_font_from_memory_ttf` with a bundled TTF.
- Explain `rasterizer_density` for HiDPI crispness.
- Note the `dear-app` callback ordering gotcha (`on_fonts` runs before the scale
  factor is known), which rules out reading `display_framebuffer_scale` there.

---

**Issue: `UiState` manual `Default` impl can be derived**

The tutorial's `UiState` has a hand-written `impl Default` that just sets every
field to its default. Clippy flags this as `derivable_impls`.

**Fix applied:** Replaced with `#[derive(Default)]` on `UiState`.

**Suggestion:** Use `#[derive(Default)]` whenever the manual impl just delegates
to each field's default.

---

**Issue: Collapsible `if` statements (clippy)**

The tutorial nests `if` inside `if` (e.g., `if a { if b { ... } }`) in several
places. Clippy's `collapsible_if` lint flags these. With Rust 2024 edition,
let-chains (`if a && b { ... }` or `if let Some(x) = ... && cond { ... }`) make
collapsing idiomatic.

**Fix applied:** Collapsed nested `if`/`if let` into single conditions with `&&`:

```rust
// Before
if let Some(_m) = ui.begin_menu("View") {
    if ui.menu_item("ImGui Demo Window") { ... }
}
// After
if let Some(_m) = ui.begin_menu("View")
    && ui.menu_item("ImGui Demo Window")
{
    ...
}
```

**Suggestion:** Mention let-chains as the modern idiom for combining a `let`
binding with a condition, since the tutorial targets edition 2021 but the project
uses 2024.

---

## What the Tutorial Does Well

- **Architecture & separation of concerns.** The data-model / UI-rendering /
  app-wiring split is excellent and makes the graph logic testable without a
  GPU. This is the tutorial's strongest point.
- **Rust Book cross-references.** Linking each concept to specific chapters
  (ownership, traits, RAII, error handling, modules) is pedagogically valuable.
- **Borrow-pattern explanations.** The "collect-then-mutate" pattern and the
  discussion of two-phase borrows in Chapter 7 are well-explained and accurate.
- **RAII token explanation.** The coverage of scoped operations and drop-based
  cleanup in Chapter 2 is clear and correct.
- **Progressive complexity.** The chapter progression (window → data model →
  editor → nodes → links → styling → interactions → persistence → production) is
  well-paced.
- **Two-layer persistence.** Splitting layout (ImNodes INI) from structure
  (serde JSON) is the right call and well-explained.

---

## Recommended Actions for the Tutorial Author

1. **Compile every code block.** The single most impactful improvement is to
   actually build the project as written. Five of the issues above are
   compilation blockers that a `cargo build` would have caught immediately.
2. **Run clippy.** Three additional issues (derivable `Default`, collapsible
   `if`, dead code) are surfaced by `cargo clippy` and represent idiomatic-Rust
   gaps.
3. **Verify crate APIs against source or docs.rs.** Issues in Chapters 02, 06,
   09, and 10 are hallucinated or misremembered API details. Always check the
   actual crate version you target.
4. **Fix the `AppBuilder` ownership pattern.** The `Option<App>` shared-closure
   issue (Chapter 06) is the hardest blocker for a reader to solve independently.
   Provide a working `Rc<RefCell<>>` (or equivalent) example.
5. **Cover the practical basics the editor omits:** a fullscreen canvas via
   `viewport.work_pos()`/`work_size()` + `Condition::Always` (Chapter 06), the
   choice between a dockspace and a single borderless window (Chapter 04), font
   loading (`on_fonts` + a bundled TTF + `rasterizer_density` for HiDPI,
   Chapter 12), and side panels for project navigation and property editing
   (Chapter 06/08).
