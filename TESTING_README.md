> **SUPERSEDED (2026-07-06, ldui-49w):** The manual drive-Chrome-DevTools procedure
> in this document is superseded by the automated PixelProof suite:
> `tests/visual_smoke.rs` + `tests/reactivity_smoke.rs`, run end-to-end via
> `cargo make test-visual` (committed baselines: `tests/visual/baselines/`).
> This file is kept for its component-state checklists only — do not follow it
> as a testing procedure.

# Demo App Testing Guide
## Complete Validation of 96 daisyUI Components

This directory contains a comprehensive testing suite for validating 100% of the demo app's component coverage using Chrome DevTools MCP.

---

## 📚 Documentation Files

### 1. **TESTING_PLAN.md** (Strategy & Planning)
- **Purpose**: Comprehensive testing strategy and methodology
- **Content**:
  - Test phases and categories
  - Detailed test cases for each component
  - Success criteria and validation checklists
  - Performance testing guidelines
  - Visual regression testing approach
- **Audience**: QA engineers, project managers, developers planning test strategy
- **Length**: ~400 lines, in-depth reference

### 2. **test-demo.md** (Interactive Testing Guide)
- **Purpose**: Step-by-step guide for hands-on testing
- **Content**:
  - Quick start instructions (5 min)
  - Phase-by-phase testing workflows
  - Sample component tests with examples
  - Mobile responsiveness testing
  - Performance measurement commands
- **Audience**: Developers doing manual/interactive testing
- **Length**: ~300 lines, practical guide

### 3. **run-automated-tests.md** (Execution Scripts)
- **Purpose**: Copy-paste ready commands for automated testing
- **Content**:
  - Setup commands
  - All 62 component test commands (organized by category)
  - Mobile and theme testing commands
  - Verification and validation commands
- **Audience**: Automation engineers, CI/CD pipelines
- **Length**: ~500 lines, executable commands

### 4. **TESTING_README.md** (This File)
- **Purpose**: Overview and quick start guide
- **Content**: Documentation index and getting started instructions

---

## 🚀 Quick Start (Choose Your Path)

### Path A: Interactive Testing (Recommended for First-Time)
**Time**: 15-30 minutes
**File**: `test-demo.md`

1. Start demo server:
   ```powershell
   .\launcher.ps1 -Task dev
   ```

2. Follow "Quick Start" section in `test-demo.md`

3. Run Phase 2 sample tests (one component per category)

4. Execute interactive component tests (modals, dropdowns, etc.)

**Best for**: Understanding the testing process, learning component behavior

---

### Path B: Automated Full Validation
**Time**: 30-45 minutes
**File**: `run-automated-tests.md`

1. Start demo server:
   ```powershell
   .\launcher.ps1 -Task dev
   ```

2. Ask Claude to execute automated tests:
   ```
   "Please run all component tests from run-automated-tests.md
   sequentially and report any errors or failures."
   ```

3. Or manually execute commands from the file category by category

**Best for**: Complete validation, regression testing, CI/CD

---

### Path C: Strategic Planning
**Time**: 10-15 minutes reading
**File**: `TESTING_PLAN.md`

1. Review the comprehensive testing strategy

2. Understand test phases and success criteria

3. Plan your testing approach

4. Reference detailed test cases as needed

**Best for**: QA planning, understanding test coverage, creating custom test plans

---

## 🎯 Testing Approach Summary

### Component Coverage
- **Total Components**: 96
- **Categories**: 7 (Action, Data Display, Navigation, Feedback, Data Input, Layout, Mockup)
- **Newly Added** (2026-01-28): Calendar, FAB, Hover 3D, Hover Gallery, Text Rotate, Tooltip

### Test Layers
1. **Basic Rendering**: Component loads without errors
2. **Visual Validation**: Elements display correctly
3. **Interactive Behavior**: Clicks, hovers, inputs work
4. **Responsive Design**: Mobile/desktop layouts
5. **Theme Support**: Light/dark theme switching
6. **Performance**: Load times, resource usage

### Testing Tools
- **Chrome DevTools MCP**: Browser automation and inspection
- **Claude Chrome Extension**: Interactive testing assistance
- **Trunk Dev Server**: Live reload development server

---

## 📋 Component Categories

### Action (6 components)
Button, Dropdown, FAB, Modal, Swap, Theme Controller

### Data Display (18 components)
Accordion, Avatar, Badge, Card, Carousel, Chat, Collapse, Countdown, Diff, Hover 3D, Hover Gallery, Kbd, List, Stats, Status, Table, Text Rotate, Timeline

### Navigation (6 components)
Breadcrumbs, Menu, Navbar, Pagination, Steps, Tab

### Feedback (7 components)
Alert, Loading, Progress, Radial Progress, Skeleton, Toast, Tooltip

### Data Input (14 components)
Calendar, Checkbox, Fieldset, File Input, Filter, Input, Label, Radio, Range, Rating, Select, Textarea, Toggle, Validator

### Layout (10 components)
Divider, Dock, Drawer, Footer, Hero, Indicator, Join, Link, Mask, Stack

### Mockup (4 components)
Mockup Browser, Mockup Code, Mockup Phone, Mockup Window

---

## 🔧 Prerequisites

### Required Tools
- ✅ Rust toolchain (latest stable)
- ✅ Trunk (wasm build tool)
- ✅ Node.js (for Tailwind CSS)
- ✅ Chrome browser
- ✅ Claude Code with Chrome DevTools MCP

### Setup Commands
```powershell
# Install prerequisites (if needed)
cargo install trunk

# Install Tailwind CSS (in demo directory)
cd demo
npm install

# Start dev server
cd ..
.\launcher.ps1 -Task dev
# Or: cargo make dev
# Or: cd demo && trunk serve

# Verify server running at http://127.0.0.1:3000
```

---

## 🧪 Basic Test Flow

### Step 1: Setup
```javascript
list_pages()
select_page({ pageIdx: 0 })
resize_page({ width: 1920, height: 1080 })
```

### Step 2: Navigate
```javascript
navigate_page({ type: "url", url: "http://127.0.0.1:3000/components/button" })
```

### Step 3: Validate
```javascript
wait_for({ text: "Button", timeout: 3000 })
take_snapshot()
list_console_messages({ types: ["error"] })
```

### Step 4: Repeat
Execute for all 62 components

---

## ✅ Success Criteria

### Component-Level
- [ ] Component renders without errors
- [ ] All variants display correctly
- [ ] Interactive elements respond
- [ ] No console errors
- [ ] Responsive behavior works

### Application-Level
- [ ] 62/62 components accessible
- [ ] Navigation menu works
- [ ] Mobile drawer functions
- [ ] Theme switching works
- [ ] Zero critical errors
- [ ] Performance acceptable

---

## 📊 Expected Output

### Per Component
```
✅ button: Component loaded successfully
   - Snapshot: 156 elements
   - Console errors: 0
   - Load time: 234ms
```

### Overall Summary
```
📊 TEST RESULTS
================
✅ Successful: 62/62
❌ Failed: 0/62
⚠️ Total errors: 0
⏱️ Total time: 28 minutes
```

---

## 🐛 Common Issues & Solutions

### Issue: Component not loading
**Symptoms**: Timeout waiting for element
**Solution**:
- Check demo server is running
- Verify route exists in `demo/src/main.rs`
- Increase timeout to 5000ms

### Issue: Console errors on all pages
**Symptoms**: JavaScript errors appear everywhere
**Solution**:
- Check Trunk build completed successfully
- Verify Tailwind CSS compiled (`demo/output.css` exists)
- Run `cargo make clean-all && cargo make dev`

### Issue: Mobile drawer not working
**Symptoms**: Hamburger menu not visible/clickable
**Solution**:
- Verify viewport size is mobile (width < 1024px)
- Check drawer implementation in `demo/src/core/layout.rs`
- Use `take_snapshot({ verbose: true })` to find drawer UIDs

### Issue: Theme not switching
**Symptoms**: `data-theme` attribute not changing
**Solution**:
- Verify theme controller implementation
- Check signal propagation to root `<Html>` element
- Test manually in browser first

---

## 📸 Screenshot Generation

To capture screenshots of all components:

```javascript
// Create screenshots directory first
// Then for each component:

take_screenshot({
  filePath: "screenshots/button.png",
  fullPage: true
})
```

Recommended structure:
```
screenshots/
├── landing.png
├── 01-button.png
├── 02-dropdown.png
├── ...
├── 62-validator.png
├── mobile-view.png
└── theme-dark.png
```

---

## 🤖 Automation Options

### Option 1: Manual Execution
Copy commands from `run-automated-tests.md` and execute one by one through Claude Code

### Option 2: Task Tool Automation
```
"Use the general-purpose agent to execute all component tests
from run-automated-tests.md and report results"
```

### Option 3: Custom Script
Create a custom automation script using the templates in `TESTING_PLAN.md`

---

## 📈 Performance Benchmarks

### Target Metrics
- **Landing page load**: < 500ms
- **Component page load**: < 300ms
- **Network requests**: < 50 per page
- **Console errors**: 0
- **Layout shifts (CLS)**: < 0.1

### Measurement Commands
```javascript
evaluate_script({
  function: `() => {
    const perfData = performance.getEntriesByType("navigation")[0];
    return {
      domComplete: Math.round(perfData.domComplete),
      loadTime: Math.round(perfData.loadEventEnd - perfData.fetchStart),
      resources: performance.getEntriesByType("resource").length
    };
  }`
})
```

---

## 🔍 Interactive Features to Test

Priority list for manual interaction testing:

### High Priority
1. **Modal** - ESC key, outside click, backdrop
2. **Dropdown** - Keyboard navigation, outside click
3. **Theme Controller** - Theme persistence across pages
4. **Drawer** - Mobile toggle, overlay click
5. **FAB** - Speed dial animation

### Medium Priority
6. **Carousel** - Navigation, swipe gestures
7. **Collapse** - Expand/collapse animation
8. **Tabs** - Content switching
9. **Range** - Slider dragging
10. **Rating** - Star selection

### Lower Priority
11. **Tooltip** - Hover timing
12. **Hover 3D** - Mouse tracking
13. **Swap** - Toggle animation
14. **Countdown** - Timer accuracy
15. **Text Rotate** - Rotation cycle

---

## 📝 Test Report Template

After testing, document results:

```markdown
# Test Results - [Date]

## Summary
- Total Components: 96
- Tested: 62
- Passed: X
- Failed: Y
- Console Errors: Z

## Component Results
| Component | Status | Errors | Notes |
|-----------|--------|--------|-------|
| Button    | ✅ Pass | 0      | All variants working |
| Dropdown  | ⚠️ Warning | 1   | Minor console warning |
| ...       | ...    | ...    | ... |

## Failed Components
1. ComponentName - Description of issue

## Performance
- Average load time: X ms
- Total network requests: Y
- Resource size: Z MB

## Recommendations
1. Fix issue in component X
2. Optimize Y for better performance
3. Update documentation for Z
```

---

## 🎓 Learning Resources

### Chrome DevTools MCP Documentation
- Tool reference for available commands
- Examples of snapshot, screenshot, interaction tools
- Performance monitoring capabilities

### Demo App Structure
- `demo/src/main.rs` - Routes and app structure
- `demo/src/core/layout.rs` - Layout and navigation
- `demo/src/demos/` - Individual component demos

### daisyUI Documentation
- https://daisyui.com/components/ - Component reference
- Styling classes and variants
- Theme customization

---

## 🚦 Next Steps

1. **First Time**: Follow Path A (Interactive Testing) to understand the process
2. **Full Validation**: Use Path B (Automated) for complete coverage
3. **Document Results**: Create test report with findings
4. **Fix Issues**: Create GitHub issues for any failures
5. **Regression Testing**: Add tests to CI/CD pipeline

---

## 💡 Tips & Best Practices

### Efficient Testing
- Test in batches by category (6-18 components per batch)
- Take breaks between categories
- Keep console open for real-time error monitoring

### Error Handling
- Don't panic on first error - continue testing
- Document all errors for batch fixing
- Check if errors are global (Trunk build) or component-specific

### Screenshots
- Use full-page screenshots for documentation
- Create comparison screenshots (light vs dark theme)
- Capture mobile and desktop views

### Performance
- Clear browser cache between test runs
- Monitor memory usage during long test sessions
- Close other browser tabs for consistent results

---

## 📞 Getting Help

### Issues
- Check "Troubleshooting" section in each file
- Review demo app logs in terminal
- Test manually in browser to isolate MCP issues

### Questions
- Reference `TESTING_PLAN.md` for detailed test cases
- Check `CLAUDE.md` for project-specific guidance
- Review component source code in `src/components/`

---

**Ready to start testing?** Choose your path above and begin validating!

🎯 **Goal**: 100% component coverage validation
📊 **Deliverable**: Complete test report with 62/62 components verified
⏱️ **Time**: 15-45 minutes depending on approach
