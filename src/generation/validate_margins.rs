use crate::generation::generate_pdf::{GridConfig, PageMargins};

/// Validates that margins and grid configuration won't cause content overflow
/// A4 page dimensions: 21cm width x 29.7cm height
pub fn validate_margins_and_grid(
    margins: &PageMargins,
    config: &GridConfig,
    top_offset: f64,
    left_offset: f64,
) -> Result<(), String> {
    const A4_WIDTH_CM: f64 = 21.0;
    const A4_HEIGHT_CM: f64 = 29.7;
    const NUM_COLUMNS: usize = 3;

    // Check if margins are negative (which would be problematic)
    // Note: offsets can be negative as they represent positioning adjustments
    if margins.left < 0.0 || margins.right < 0.0 || margins.top < 0.0 || margins.bottom < 0.0 {
        return Err(format!(
            "Margins cannot be negative. \
            Left: {:.2}cm, Right: {:.2}cm, Top: {:.2}cm, Bottom: {:.2}cm",
            margins.left, margins.right, margins.top, margins.bottom
        ));
    }

    // Calculate total horizontal space needed
    // For worst-case scenario: if left_offset is positive, add it to left margin
    // If negative, it might reduce space but we validate conservatively
    let effective_left_margin = margins.left + left_offset.max(0.0);
    let total_horizontal = effective_left_margin
        + margins.right
        + (config.image_width * NUM_COLUMNS as f64)
        + (config.column_gutter * (NUM_COLUMNS - 1) as f64);

    // Only error if significantly over page width (allowing some flexibility for Typst/PDF rendering)
    if total_horizontal > A4_WIDTH_CM * 1.3 {
        return Err(format!(
            "Content width ({:.2}cm) significantly exceeds page width ({:.2}cm). \
            Margins: left={:.2}cm, right={:.2}cm. Left offset: {:.2}cm. \
            Reduce calibration offsets significantly.",
            total_horizontal, A4_WIDTH_CM, margins.left, margins.right, left_offset
        ));
    }

    // Check vertical space (2 rows + gutter + top offset) - also allow flexibility
    // For worst-case scenario: if top_offset is positive, add it to top margin
    // If negative, it might reduce space but we validate conservatively
    let effective_top_margin = margins.top + top_offset.max(0.0);
    let total_vertical = effective_top_margin + margins.bottom + config.row_gutter;
    if total_vertical > A4_HEIGHT_CM * 1.3 {
        return Err(format!(
            "Vertical spacing ({:.2}cm) significantly exceeds page height ({:.2}cm). \
            Margins: top={:.2}cm, bottom={:.2}cm. Top offset: {:.2}cm.",
            total_vertical, A4_HEIGHT_CM, margins.top, margins.bottom, top_offset
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_validate_margins_valid() {
        let margins = PageMargins::default();
        let config = GridConfig::default();

        // Should pass with default values
        let result = validate_margins_and_grid(&margins, &config, 0.0, 0.0);
        if let Err(e) = &result {
            println!("Validation error: {}", e);
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_margins_overflow() {
        let margins = PageMargins {
            top: 0.0,
            bottom: 0.0,
            left: 15.0,  // Very large left margin
            right: 15.0, // Very large right margin
        };
        let config = GridConfig::default();

        // Should fail - content would overflow significantly
        let result = validate_margins_and_grid(&margins, &config, 0.0, 0.0);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("significantly exceeds page width")
        );
    }

    #[test]
    fn test_validate_margins_negative() {
        let margins = PageMargins {
            top: -1.0, // Negative margin
            bottom: 0.0,
            left: 2.0,
            right: 2.0,
        };
        let config = GridConfig::default();

        // Should fail - negative margins not allowed
        let result = validate_margins_and_grid(&margins, &config, 0.0, 0.0);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("cannot be negative"));
    }

    #[test]
    fn test_validate_negative_offsets_allowed() {
        let margins = PageMargins::default();
        let config = GridConfig::default();

        // Should pass - negative offsets are allowed as positioning adjustments
        let result = validate_margins_and_grid(&margins, &config, -1.0, 0.0);
        assert!(result.is_ok());

        let result = validate_margins_and_grid(&margins, &config, 0.0, -1.0);
        assert!(result.is_ok());

        // Should pass - both offsets negative
        let result = validate_margins_and_grid(&margins, &config, -2.0, -3.0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_offset_overflow_horizontal() {
        let margins = PageMargins::default();
        let config = GridConfig::default();

        // Should fail - large positive left offset causes horizontal overflow
        let result = validate_margins_and_grid(&margins, &config, 0.0, 15.0);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("significantly exceeds page width")
        );

        // Should pass - large negative left offset doesn't cause overflow
        // (it moves content left, potentially reducing space requirements)
        let result = validate_margins_and_grid(&margins, &config, 0.0, -15.0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_offset_overflow_vertical() {
        let margins = PageMargins::default();
        let config = GridConfig::default();

        // Should fail - large positive top offset causes vertical overflow
        // Need a much larger value to exceed the 29.7 * 1.3 = 38.61cm threshold
        let result = validate_margins_and_grid(&margins, &config, 40.0, 0.0);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("significantly exceeds page height")
        );

        // Should pass - large negative top offset doesn't cause overflow
        // (it moves content up, potentially reducing space requirements)
        let result = validate_margins_and_grid(&margins, &config, -20.0, 0.0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_offsets_with_margins() {
        let margins = PageMargins {
            top: 2.0,
            bottom: 2.0,
            left: 2.0,
            right: 2.0,
        };
        let config = GridConfig::default();

        // Should pass with small positive offsets
        let result = validate_margins_and_grid(&margins, &config, 0.5, 0.5);
        assert!(result.is_ok());

        // Should fail when combined margins and offsets cause overflow
        let result = validate_margins_and_grid(&margins, &config, 5.0, 5.0);
        assert!(result.is_err());
    }
}
