# Mouse Click Cursor Position Fix Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix mouse click cursor positioning so that single clicks place the cursor exactly at the clicked position, regardless of the current SelectionMode.

**Architecture:** Add an `expand` parameter to `set_cursor_position` function. When `expand=false`, skip SelectionMode's Current movement and create an exact empty selection at the clicked position. This aligns with standard editor behavior where single click positions cursor and double click selects.

**Tech Stack:** Rust, anyhow for error handling, ropey for buffer management, existing SelectionMode trait

---

## File Structure

| File | Responsibility | Changes |
|-------|---------------|----------|
| `src/components/editor.rs` | Core editor logic, mouse handling, cursor positioning | Add `expand` parameter to `set_cursor_position`, add conditional logic, update `handle_mouse_click` call |

No new files created. Changes are confined to a single file with a focused responsibility change.

---

## Chunk 1: Test Updates (Prepare for implementation)

### Task 1: Update test_click_respects_line_mode

**Files:**
- Modify: `src/components/editor.rs:5108-5130`

**Context:** Current test expects mouse click in Line mode to select the entire line. After fix, single clicks should position cursor exactly (empty selection).

- [ ] **Step 1: Read current test to understand structure**

Run: `cargo test test_click_respects_line_mode -- --nocapture`
Expected: Current test passes with line selection

- [ ] **Step 2: Write updated test expecting exact positioning**

Update the test at line 5108-5130 to verify that clicking in Line mode results in an empty selection (cursor at exact position), not line selection.

The test should:
1. Create editor with content "hello\nworld"
2. Set SelectionMode to LineFull
3. Click in middle of "world" (column 8, row 0 after line number width)
4. Verify selection is empty (cursor at clicked position)
5. Verify selected_text returns empty string or single character

- [ ] **Step 3: Run test to verify it fails**

Run: `cargo test test_click_respects_line_mode -- --nocapture`
Expected: FAIL - current implementation still expands selection

- [ ] **Step 4: Commit test changes**

```bash
git add src/components/editor.rs
git commit -m "test(line mode): update test to expect exact positioning on mouse click

Single mouse clicks should position cursor exactly, not expand selection.
This aligns with standard editor behavior.

johncming@126.com"
```

---

### Task 2: Update test_click_respects_word_mode

**Files:**
- Modify: `src/components/editor.rs:5139-5161`

**Context:** Current test expects mouse click in Word mode to select the clicked word. After fix, single clicks should position cursor exactly.

- [ ] **Step 1: Read current test to understand structure**

Run: `cargo test test_click_respects_word_mode -- --nocapture`
Expected: Current test passes with word selection

- [ ] **Step 2: Write updated test expecting exact positioning**

Update the test at line 5139-5161 to verify that clicking in Word mode results in an empty selection (cursor at exact position), not word selection.

The test should:
1. Create editor with content "hello world"
2. Set SelectionMode to Word
3. Click in middle of "world"
4. Verify selection is empty (cursor at clicked position)

- [ ] **Step 3: Run test to verify it fails**

Run: `cargo test test_click_respects_word_mode -- --nocapture`
Expected: FAIL - current implementation still expands selection

- [ ] **Step 4: Commit test changes**

```bash
git add src/components/editor.rs
git commit -m "test(word mode): update test to expect exact positioning on mouse click

Single mouse clicks should position cursor exactly, not expand selection.
This aligns with standard editor behavior.

johncming@126.com"
```

---

### Task 3: Add edge case test for last line

**Files:**
- Modify: `src/components/editor.rs` (add new test after existing mouse click tests, around line 5175)

**Context:** Last line in buffer has no trailing newline, behavior may differ.

- [ ] **Step 1: Write test for clicking on last line**

```rust
#[test]
fn test_click_on_last_line_positions_correctly() {
    let mut editor = create_test_editor("hello\nworld");  // "world" has no trailing \n
    editor.rectangle = Rectangle {
        origin: Position::new(0, 0),
        width: 20,
        height: 10,
    };
    editor.selection_set = SelectionSet::default()
        .set_mode(SelectionMode::Line(IfCurrentNotFound::LookForward))
        .set_selections(NonEmpty::new(
            Selection::new((CharIndex(0)..CharIndex(0)).into())
        ));

    let context = Context::default();
    // Click on last line "world" at column 2 ('r')
    let result = editor.handle_mouse_click(4, 1, &context);

    assert!(result.is_ok());
    // Cursor should be at exact position, not expand to include non-existent next line
    let position = get_cursor_position(&editor).unwrap();
    assert_eq!(position.line, 1);
    assert_eq!(position.column, 2);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test test_click_on_last_line_positions_correctly -- --nocapture`
Expected: FAIL - current implementation extends selection

- [ ] **Step 3: Commit test**

```bash
git add src/components/editor.rs
git commit -m "test: add edge case test for clicking on last line

Verify cursor positioning works correctly on last line which has
no trailing newline character.

johncming@126.com"
```

---

### Task 4: Add edge case test for line boundary

**Files:**
- Modify: `src/components/editor.rs` (add new test after line 5175)

**Context:** Clicking at exact line boundary (line length position).

- [ ] **Step 1: Write test for clicking at line boundary**

```rust
#[test]
fn test_click_at_line_boundary_positions_correctly() {
    let mut editor = create_test_editor("hello\nworld");
    editor.rectangle = Rectangle {
        origin: Position::new(0, 0),
        width: 20,
        height: 10,
    };

    let context = Context::default();
    // Click at position after "hello" (column 5 + line_number_width)
    let result = editor.handle_mouse_click(7, 0, &context);

    assert!(result.is_ok());
    // Cursor should be at position 5 (end of "hello"), not extend
    let position = get_cursor_position(&editor).unwrap();
    assert_eq!(position.line, 0);
    assert_eq!(position.column, 5);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test test_click_at_line_boundary_positions_correctly -- --nocapture`
Expected: FAIL - current implementation extends selection

- [ ] **Step 3: Commit test**

```bash
git add src/components/editor.rs
git commit -m "test: add edge case test for clicking at line boundary

Verify cursor positioning works correctly when clicking at exact
end of line position.

johncming@126.com"
```

---

## Chunk 2: Implementation

### Task 5: Modify set_cursor_position signature

**Files:**
- Modify: `src/components/editor.rs:2144-2148`

**Context:** Add `expand: bool` parameter to control selection expansion.

- [ ] **Step 1: Write failing compilation test**

Change the function signature to include `expand: bool` parameter:

```rust
pub fn set_cursor_position(
    &mut self,
    row: usize,
    column: usize,
    expand: bool,  // NEW parameter
    context: &Context,
) -> anyhow::Result<Dispatches> {
```

- [ ] **Step 2: Run cargo check to verify compilation error**

Run: `cargo check`
Expected: FAIL - no callers provide the `expand` parameter

- [ ] **Step 3: Note the compilation error**

Expected error: function declaration changed but call site at line 2241 doesn't match

---

### Task 6: Add expand=false branch to set_cursor_position

**Files:**
- Modify: `src/components/editor.rs:2160-2183`

**Context:** When expand=false, create exact selection without SelectionMode expansion.

- [ ] **Step 1: Wrap existing logic in if expand block**

Wrap the existing Current movement logic (lines 2150-2182) in an `if expand` block:

```rust
pub fn set_cursor_position(
    &mut self,
    row: usize,
    column: usize,
    expand: bool,
    context: &Context,
) -> anyhow::Result<Dispatches> {
    let char_index = self.buffer.borrow().line_to_char(row)? + column;

    if expand {
        // Existing logic: Use SelectionMode's Current movement
        // ... (keep all existing code here)
    }
    // else branch added in next task
    Ok(Dispatches::default())
}
```

- [ ] **Step 2: Run cargo check**

Run: `cargo check`
Expected: OK or warnings about unused `char_index` when expand=true

---

### Task 7: Add expand=false implementation

**Files:**
- Modify: `src/components/editor.rs:2183` (add after if expand block)

**Context:** When expand=false, create exact empty selection.

- [ ] **Step 1: Add else branch for exact positioning**

Add the else branch after the if expand block:

```rust
    } else {
        // Use exact position without expansion
        let exact_selection = Selection::new((char_index..char_index).into());
        let new_selection_set = self.selection_set
            .clone()
            .set_selections(NonEmpty::new(exact_selection));
        return Ok(self.update_selection_set(new_selection_set, true, context));
    }
```

Note: Remove the trailing `Ok(Dispatches::default())` from Task 6's else placeholder.

- [ ] **Step 2: Run cargo check**

Run: `cargo check`
Expected: OK - function compiles correctly

- [ ] **Step 3: Commit implementation**

```bash
git add src/components/editor.rs
git commit -m "feat(cursor): add expand parameter to set_cursor_position

When expand=false, cursor is positioned exactly without SelectionMode
expansion. This enables precise mouse click positioning.

johncming@126.com"
```

---

### Task 8: Update handle_mouse_click call

**Files:**
- Modify: `src/components/editor.rs:2241`

**Context:** Pass expand=false for exact positioning on mouse click.

- [ ] **Step 1: Update the function call**

Change the call at line 2241 from:
```rust
self.set_cursor_position(buffer_row, buffer_col, context)
```

To:
```rust
self.set_cursor_position(buffer_row, buffer_col, false, context)
```

- [ ] **Step 2: Run tests to verify they pass**

Run: `cargo test mouse -- --nocapture`
Expected: New tests pass (Tasks 1-4), existing cursor test passes

- [ ] **Step 3: Commit**

```bash
git add src/components/editor.rs
git commit -m "fix(mouse): use exact positioning for mouse clicks

Single mouse clicks now position cursor exactly at clicked location,
regardless of SelectionMode. Aligns with standard editor behavior.

Fixes: mouse clicks in Line mode extending selection to next line.

johncming@126.com"
```

---

## Chunk 3: Verification and Documentation

### Task 9: Run all mouse-related tests

**Files:**
- Test: all test files

- [ ] **Step 1: Run full test suite for mouse functionality**

Run: `cargo test --lib mouse`
Expected: All mouse-related tests pass

- [ ] **Step 2: Run full editor test suite**

Run: `cargo test --lib editor`
Expected: All editor tests pass, no regressions

- [ ] **Step 3: Note any failing tests**

If any tests fail, document which ones and why. These may need investigation or may be expected failures due to behavior change.

---

### Task 10: Verify keyboard navigation still works

**Files:**
- Test: manual verification

- [ ] **Step 1: Manual test keyboard navigation**

Test the following to ensure SelectionMode expansion still works for keyboard:
1. Open editor with content
2. Set SelectionMode to Line
3. Use arrow keys (Up, Down, Left, Right)
4. Verify selection expands to lines as expected
5. Set SelectionMode to Word
6. Use arrow keys
7. Verify selection expands to words as expected

- [ ] **Step 2: Document any issues**

If keyboard navigation is broken, this indicates the expand parameter handling has a bug.

---

### Task 11: Verify different SelectionModes

**Files:**
- Test: manual verification

- [ ] **Step 1: Test mouse click in each SelectionMode**

Test single mouse click in each mode:
1. Character mode - click positions exactly ✓
2. Word mode - click positions exactly ✓
3. Subword mode - click positions exactly ✓
4. Line mode - click positions exactly (no extension) ✓
5. Syntax node mode - click positions exactly ✓

- [ ] **Step 2: Verify multi-cursor is cleared**

1. Create multiple cursors (if possible in current implementation)
2. Click somewhere
3. Verify all but primary cursor are cleared (existing behavior at line 2236)

---

### Task 12: Final integration test

**Files:**
- Test: comprehensive scenario

- [ ] **Step 1: Create end-to-end test scenario**

Test the complete user journey:
1. Open file with multiple lines
2. Set SelectionMode to Line
3. Click in middle of first line
4. Verify cursor at exact position
5. Type some text
6. Verify text inserted at correct location
7. Use arrow keys to navigate
8. Verify SelectionMode expansion still works for keyboard

- [ ] **Step 2: Document results**

Record success or failure of each step.

- [ ] **Step 3: Final commit if any adjustments needed**

If any issues found and fixed during testing:

```bash
git add src/components/editor.rs
git commit -m "fix: adjust implementation based on testing

Address issues found during verification testing.

johncming@126.com"
```

---

## Summary

**Total Tasks:** 12
**Estimated Time:** 2-3 hours for implementation, 1 hour for testing

**Test Changes Required:**
- `test_click_respects_line_mode` - expect empty selection
- `test_click_respects_word_mode` - expect empty selection
- New test: `test_click_on_last_line_positions_correctly`
- New test: `test_click_at_line_boundary_positions_correctly`

**Code Changes:**
- `set_cursor_position` signature: add `expand: bool`
- `set_cursor_position` body: add conditional logic
- `handle_mouse_click`: pass `expand=false`

**Success Criteria:**
- All new tests pass
- All existing mouse tests pass (except two which now expect different behavior)
- Keyboard navigation with SelectionMode still works correctly
- Manual testing confirms expected behavior
