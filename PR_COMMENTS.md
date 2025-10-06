# PR Comments: HTML/CSS PDF Generation PoC

## What Changed

### 1. New PDF Generation Engine
- **Added `generate_pdf_html.rs`**: New module implementing HTML/CSS-based PDF generation
- **Modified `mod.rs`**: Added environment variable toggle (`USE_HTML_PDF`) to switch between Typst and HTML generation
- **Shared Types**: Refactored `RectoVersoImagePair`, `PageMargins`, and `GridConfig` to be shared between both generation methods

### 2. Printer Calibration Support
- **Added `calibration.rs`**: Complete calibration system including:
  - Calibration page HTML template with alignment marks
  - `CalibrationOffsets` struct for storing printer offset measurements
  - Methods to apply offsets to page margins
- **Added `CALIBRATION_SCHEMA.md`**: Comprehensive documentation on how to use the calibration feature
- **Environment Variables**: `CALIBRATION_OFFSET_HORIZONTAL` and `CALIBRATION_OFFSET_VERTICAL` for runtime configuration

### 3. Documentation Updates
- **Updated README.md**: Added sections for:
  - HTML/CSS generation feature
  - Printer calibration instructions
  - Environment variable documentation
  - Building and development instructions

### 4. Code Quality
- **Tests**: All existing tests pass, plus 3 new tests for calibration functionality
- **Type Safety**: Maintained Rust's type safety throughout
- **Backward Compatibility**: Typst generation remains the default; HTML is opt-in

## Advantages Gained

### 1. **Easier Layout Customization**
- **Before (Typst)**: Requires learning Typst's syntax and markup language
- **After (HTML/CSS)**: Web developers can use familiar HTML/CSS to customize layouts
- **Impact**: Lower barrier to entry for contributions and customizations

### 2. **Better Debugging**
- **Before**: Debugging Typst layout issues requires understanding its compilation process
- **After**: Can inspect HTML in a browser, use developer tools, and see exactly what will be rendered
- **Impact**: Faster iteration and troubleshooting

### 3. **Printer Calibration Support**
- **Before**: No way to compensate for printer misalignment
- **After**: 
  - Generate calibration test pages
  - Measure offsets
  - Automatically apply corrections
  - Perfect alignment for double-sided printing
- **Impact**: Professional results even with imperfect printers

### 4. **CSS Flexibility**
- **Before**: Layout constrained by Typst's grid system
- **After**: Full power of CSS Grid, Flexbox, transforms, and modern CSS features
- **Impact**: More layout options, easier to implement complex designs

### 5. **Simpler Mental Model**
- **Before**: Need to understand Typst's document structure and compilation
- **After**: Standard web rendering model that most developers know
- **Impact**: Faster onboarding for new contributors

### 6. **Print-Specific Features**
- **CSS @page rules**: Direct control over page margins, size, and breaks
- **Print media queries**: Different styling for screen vs. print
- **Page-break control**: Fine-grained control over pagination
- **Impact**: Better control over final print output

### 7. **Environment-Based Configuration**
- **Before**: Settings hardcoded in Rust
- **After**: Runtime configuration via environment variables
- **Impact**: Easier deployment, no recompilation needed for different printers

## Trade-offs

### HTML/CSS Advantages:
✅ Familiar syntax (HTML/CSS)
✅ Better debugging tools
✅ Easier calibration support
✅ More flexible styling
✅ Simpler to understand and modify

### HTML/CSS Disadvantages:
❌ Requires external tool (wkhtmltopdf)
❌ Slightly slower than native Typst
❌ Larger dependency footprint
❌ May have subtle rendering differences across platforms

### Typst Advantages:
✅ No external dependencies
✅ Fast compilation
✅ Consistent rendering
✅ Built for precise typesetting

### Typst Disadvantages:
❌ Less familiar syntax
❌ Harder to debug
❌ Less flexible for certain layouts

## Performance Comparison

| Metric | Typst | HTML/CSS |
|--------|-------|----------|
| Compilation Speed | Fast (native) | Moderate (external tool) |
| Memory Usage | Low | Higher (wkhtmltopdf) |
| Setup Complexity | Low (built-in) | Medium (needs wkhtmltopdf) |
| Debugging Ease | Medium | High |
| Customization Ease | Medium | High |

## Why Both?

We kept both methods because:
1. **Different Use Cases**: Typst for speed, HTML/CSS for flexibility
2. **Gradual Migration**: Users can try HTML without losing Typst
3. **Feature Testing**: PoC allows real-world testing before full commitment
4. **User Choice**: Different users have different needs and preferences

## Calibration Feature Deep Dive

### Problem Solved
Many printers have mechanical imperfections that cause slight misalignment between recto (front) and verso (back) pages. This is especially noticeable with:
- Bookmarks that need precise cutting
- Double-sided business cards
- Any material where both sides must align

### Solution Architecture
```
1. Generate calibration page (HTML with alignment marks)
   ↓
2. User prints and measures offsets
   ↓
3. User sets environment variables
   ↓
4. System applies offsets to margins automatically
   ↓
5. Perfect alignment in final PDF
```

### Why HTML/CSS is Better for Calibration
- **Visual Design**: Easier to create precise alignment marks in CSS
- **Measurement Markers**: Can use CSS transforms for exact positioning
- **Print Rules**: @page CSS rules for print-specific styling
- **Iterative Design**: Faster to test and refine calibration page design

## Testing Strategy

### Existing Tests (Still Passing)
- ✅ `test_generated_content_should_be_correct` - Typst generation
- ✅ All generation module tests

### New Tests Added
- ✅ `test_calibration_html_generation` - Calibration page HTML
- ✅ `test_calibration_offsets_default` - Default offset values
- ✅ `test_apply_calibration_to_margins` - Offset calculation
- ✅ `test_generate_html_content` - HTML generation logic

### Integration Testing (Manual)
To fully test:
1. Set `USE_HTML_PDF=true`
2. Generate PDF
3. Compare output with Typst version (should be visually identical)
4. Test calibration page generation
5. Apply offsets and verify they affect output correctly

## Migration Path (If Adopted)

### Phase 1 (Current): PoC
- Both methods available
- HTML/CSS opt-in via environment variable
- Documentation for both methods

### Phase 2: Refinement
- Gather user feedback
- Optimize HTML generation performance
- Improve calibration UX

### Phase 3: Decision Point
- Decide based on user feedback:
  - Option A: Keep both (flexibility)
  - Option B: Deprecate Typst (simplicity)
  - Option C: Make HTML default, keep Typst as fallback

## Questions for Review

1. **Dependency**: Is adding wkhtmltopdf as a system dependency acceptable?
2. **Default**: Should HTML/CSS become the default in the future?
3. **Calibration UI**: Should we add a web UI for calibration instead of env vars?
4. **Performance**: Is the performance trade-off acceptable for the flexibility gained?

## Next Steps

If this PoC is approved:
1. Add Dockerfile changes to install wkhtmltopdf
2. Consider adding a web UI for calibration settings
3. Add more comprehensive integration tests
4. Document printer-specific calibration profiles
5. Potentially explore other HTML-to-PDF engines (headless Chrome, etc.)

## Conclusion

This PoC demonstrates that HTML/CSS-based PDF generation is viable and brings significant advantages in terms of:
- Ease of customization
- Debugging capabilities
- Printer calibration support
- Developer familiarity

The approach maintains backward compatibility while opening new possibilities for features like the calibration system that would be much harder to implement with Typst alone.
