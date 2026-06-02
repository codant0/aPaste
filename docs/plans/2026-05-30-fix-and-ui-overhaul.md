# aPaste Fix & UI Overhaul Plan

## Overview

Fix 3 bugs and perform a full UI visual overhaul of the aPaste clipboard manager.

## Tasks

### Task 1: Fix Window Drag — Remove data-tauri-drag-region

**Files:** `src/App.tsx`

- Remove `data-tauri-drag-region` attribute from the title bar div (line 63)
- Keep the Win32 `WM_NCHITTEST` subclass as the sole drag mechanism
- Verify `cursor-grab` CSS still applies (visual hint only, actual drag is Win32)

### Task 2: Fix Double-Click Fullscreen

**Files:** `src-tauri/src/lib.rs`

- Add `WM_NCLBUTTONDBLCLK` (0x00A3) handling in `drag_wnd_proc`
- When the message is received and cursor is in the title bar zone, return `LRESULT(0)` to swallow it
- This prevents Windows from maximizing the window on title bar double-click

### Task 3: Fix Delete Button Visibility

**Files:** `src/components/ResultItem.tsx`

- Add `group` class to the parent div of ResultItem
- The existing `opacity-0 group-hover:opacity-100` on the delete button will then work correctly

### Task 4: Settings View Integration

**Files:** `src/App.tsx`, `src/components/Settings.tsx`, `src-tauri/src/lib.rs`

**Frontend (App.tsx):**
- Add `view` state: `"main" | "settings"` (default: `"main"`)
- Import `Settings` component
- Add gear icon button to title bar (next to "Clear All")
- Click gear toggles to settings view; click back arrow returns to main
- Conditionally render `<Settings />` when `view === "settings"`
- Listen for `show-settings` event from Rust to auto-switch to settings view
- Pass `onBack` prop to Settings so it can return to main view

**Frontend (Settings.tsx):**
- Add `onBack` prop
- Add back arrow button in settings title bar
- Keep existing functionality (load/save settings)

**Rust (lib.rs):**
- In tray menu "settings" handler: show window + emit `show-settings` event
- This way tray "Settings" item opens the window directly to settings view

### Task 5: Full UI Visual Overhaul

**Files:** `src/App.tsx`, `src/components/SearchBar.tsx`, `src/components/ResultItem.tsx`, `src/components/ResultList.tsx`, `src/components/StatusBar.tsx`, `src/App.css`, `src/styles/index.css`

**Design Direction:**
- Modern dark glass-morphism aesthetic
- Rose accent color (keep existing palette)
- Smooth animations and micro-interactions
- Better visual hierarchy and spacing
- Polished, professional feel

**Specific Changes:**

1. **Title Bar**: Add subtle gradient, improve layout, gear icon + clear all button styling
2. **SearchBar**: Improved focus ring animation, better placeholder, subtle glow on focus
3. **ResultItem**:
   - Add `group` class (fix bug)
   - Fade-in animation for new items
   - Better hover/selected states with smooth transitions
   - Improved content preview typography
   - Delete button slides in from right on hover
   - Better time/source metadata layout
4. **ResultList**: Staggered fade-in animation for items, improved empty state with illustration
5. **StatusBar**: Add icons to keyboard hints, improve layout, subtle separator
6. **Overall**: Better spacing, consistent border radius, improved color contrast, CSS animations for transitions

**Animation Details:**
- Items: `@keyframes fadeIn` with staggered `animation-delay`
- Selection: smooth background color transition (150ms)
- Delete button: slide-in from right + fade
- Settings view: slide-in from right transition
- Hover states: subtle scale/brightness changes

### Task 6: Settings View Polish

**Files:** `src/components/Settings.tsx`

- Apply consistent styling with main view (glass-morphism, rose accents)
- Add section headers with icons
- Better toggle switch design
- Smooth transitions between main and settings view
- Add "About" section with app version

## Execution Order

1. Task 1 (drag fix) — quick, no dependencies
2. Task 2 (double-click fix) — quick, no dependencies
3. Task 3 (delete button fix) — quick, no dependencies
4. Task 4 (settings integration) — medium, depends on understanding current structure
5. Task 5 (UI overhaul) — large, the main effort
6. Task 6 (settings polish) — medium, after settings integration works

Tasks 1, 2, 3 can be done in parallel. Task 4 before 5/6 (settings must work before polishing).
