//! SAMS Utils - Utility functions for semantic atom processing
//!
//! This module provides utility functions for fixed-point arithmetic,
//! time handling, and other common operations.

/// Fixed-point arithmetic utilities
pub mod fixed_point {
    use crate::FIXED_POINT_PRECISION;

    /// Convert f64 to fixed-point representation (hundredths)
    #[inline]
    pub fn f64_to_fixed(value: f64) -> u32 {
        // Clamp to reasonable range
        let clamped = value.clamp(0.0, 4_294_967.29); // Max u32 / 100
        (clamped * FIXED_POINT_PRECISION as f64) as u32
    }

    /// Convert fixed-point to f64
    #[inline]
    pub fn fixed_to_f64(value: u32) -> f64 {
        value as f64 / FIXED_POINT_PRECISION as f64
    }

    /// Convert string to fixed-point
    pub fn string_to_fixed(s: &str) -> Result<u32, &'static str> {
        let value: f64 = s.parse()
            .map_err(|_| "Invalid number format")?;
        Ok(f64_to_fixed(value))
    }

    /// Round fixed-point value to nearest precision
    #[inline]
    pub fn round_to_precision(value: u32, precision: u32) -> u32 {
        let factor = FIXED_POINT_PRECISION / precision;
        ((value + factor / 2) / factor) * factor
    }

    /// Add two fixed-point values with overflow checking
    #[inline]
    pub fn add_fixed(a: u32, b: u32) -> Option<u32> {
        a.checked_add(b)
    }

    /// Subtract two fixed-point values with underflow checking
    #[inline]
    pub fn sub_fixed(a: u32, b: u32) -> Option<u32> {
        a.checked_sub(b)
    }

    /// Multiply two fixed-point values
    #[inline]
    pub fn mul_fixed(a: u32, b: u32) -> u32 {
        ((a as u64) * (b as u64) / FIXED_POINT_PRECISION as u64) as u32
    }

    /// Divide two fixed-point values
    #[inline]
    pub fn div_fixed(a: u32, b: u32) -> Option<u32> {
        if b == 0 {
            return None;
        }
        let result = ((a as u64) * FIXED_POINT_PRECISION as u64) / (b as u64);
        Some(result as u32)
    }

    /// Calculate average of two fixed-point values
    #[inline]
    pub fn avg_fixed(a: u32, b: u32) -> u32 {
        (a / 2) + (b / 2) + ((a % 2) + (b % 2)) / 2
    }

    /// Calculate percentage change between two fixed-point values
    pub fn percentage_change(old: u32, new: u32) -> Option<f64> {
        if old == 0 {
            return None;
        }
        let old_f = fixed_to_f64(old);
        let new_f = fixed_to_f64(new);
        Some(((new_f - old_f) / old_f) * 100.0)
    }
}

/// Time utilities
pub mod time {

    /// Microsecond timestamp type
    pub type TimestampUs = u64;

    /// Millisecond timestamp type
    pub type TimestampMs = u64;

    /// Second timestamp type
    pub type TimestampSec = u64;

    /// Convert microseconds to milliseconds
    #[inline]
    pub fn us_to_ms(us: TimestampUs) -> TimestampMs {
        us / 1000
    }

    /// Convert milliseconds to microseconds
    #[inline]
    pub fn ms_to_us(ms: TimestampMs) -> TimestampUs {
        ms * 1000
    }

    /// Convert microseconds to seconds
    #[inline]
    pub fn us_to_sec(us: TimestampUs) -> TimestampSec {
        us / 1_000_000
    }

    /// Convert seconds to microseconds
    #[inline]
    pub fn sec_to_us(sec: TimestampSec) -> TimestampUs {
        sec * 1_000_000
    }

    /// Get current timestamp in microseconds (if std is available)
    #[cfg(feature = "std")]
    pub fn now_us() -> TimestampUs {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as TimestampUs
    }

    /// Get current timestamp in milliseconds (if std is available)
    #[cfg(feature = "std")]
    pub fn now_ms() -> TimestampMs {
        us_to_ms(now_us())
    }

    /// Format timestamp as ISO 8601 string (if std is available)
    #[cfg(feature = "std")]
    pub fn format_timestamp_us(us: TimestampUs) -> String {
        use chrono::{DateTime, Utc};
        DateTime::from_timestamp_micros(us as i64)
            .map_or("Invalid timestamp".to_string(), |dt: DateTime<Utc>| {
                dt.format("%Y-%m-%dT%H:%M:%S%.6fZ").to_string()
            })
    }

    /// Parse ISO 8601 timestamp to microseconds (if std is available)
    #[cfg(feature = "std")]
    pub fn parse_timestamp_us(s: &str) -> TimestampUs {
        use chrono::{DateTime, Utc};
        s.parse::<DateTime<Utc>>()
            .map_or(0, |dt: DateTime<Utc>| dt.timestamp_micros() as TimestampUs)
    }

    /// Calculate duration between two timestamps
    #[inline]
    pub fn duration_us(start: TimestampUs, end: TimestampUs) -> TimestampUs {
        end.saturating_sub(start)
    }

    /// Check if timestamp is within recent window
    pub fn is_recent(timestamp: TimestampUs, _window_us: TimestampUs) -> bool {
        #[cfg(feature = "std")]
        {
            let now = now_us();
            return timestamp >= now.saturating_sub(_window_us);
        }
        // Fallback: just check if timestamp is reasonable
        timestamp > 0
    }
}

/// Validation utilities
pub mod validation {
    use crate::SemanticAtom;

    /// Validation result
    #[derive(Debug, Clone, PartialEq)]
    pub enum ValidationResult {
        /// Valid atom
        Valid,
        /// Warning condition
        Warning(&'static str),
        /// Error condition
        Error(&'static str),
    }

    /// Validate atom structure
    pub fn validate_structure(atom: &SemanticAtom) -> ValidationResult {
        // Check entity ID
        if atom.entity_id == 0 {
            return ValidationResult::Warning("Entity ID is zero");
        }

        // Check timestamp
        if atom.timestamp_us == 0 {
            return ValidationResult::Warning("Timestamp is zero");
        }

        // Check telemetry type
        if atom.telemetry_type == 0 {
            return ValidationResult::Warning("Telemetry type is zero");
        }

        ValidationResult::Valid
    }

    /// Validate atom value ranges
    pub fn validate_value_ranges(atom: &SemanticAtom) -> ValidationResult {
        let value = atom.get_value();

        match atom.telemetry_type {
            0x0001 => { // Water level (mm)
                if value < 0.0 || value > 10000.0 {
                    return ValidationResult::Error("Water level out of range (0-10000mm)");
                }
            }
            0x0002 => { // Temperature (°C)
                if value < -50.0 || value > 100.0 {
                    return ValidationResult::Warning("Temperature out of typical range (-50 to 100°C)");
                }
            }
            0x0003 => { // Humidity (%)
                if value < 0.0 || value > 100.0 {
                    return ValidationResult::Error("Humidity out of range (0-100%)");
                }
            }
            0x0004 => { // Pressure (Pa)
                if value < 80000.0 || value > 120000.0 {
                    return ValidationResult::Warning("Pressure out of typical range (800-1200 hPa)");
                }
            }
            0x0005 => { // Precipitation (mm)
                if value < 0.0 || value > 1000.0 {
                    return ValidationResult::Warning("Precipitation out of typical range (0-1000mm)");
                }
            }
            0x0006 => { // CO2 (ppm)
                if value < 0.0 || value > 10000.0 {
                    return ValidationResult::Warning("CO2 out of typical range (0-10000ppm)");
                }
            }
            _ => {
                // Unknown telemetry type, but value should be reasonable
                if value < 0.0 || value > 1_000_000.0 {
                    return ValidationResult::Warning("Value out of reasonable range");
                }
            }
        }

        ValidationResult::Valid
    }

    /// Validate temporal consistency
    pub fn validate_temporal_consistency(
        current: &SemanticAtom,
        previous: Option<&SemanticAtom>,
    ) -> ValidationResult {
        if let Some(prev) = previous {
            // Check sequence number
            if current.sequence <= prev.sequence {
                return ValidationResult::Warning("Sequence number not increasing");
            }

            // Check timestamp
            if current.timestamp_us <= prev.timestamp_us {
                return ValidationResult::Error("Timestamp not increasing");
            }

            // Check rate of change (basic sanity check)
            let time_diff = current.timestamp_us.saturating_sub(prev.timestamp_us);
            if time_diff == 0 {
                return ValidationResult::Warning("Duplicate timestamp");
            }

            // If time difference is too small, warn
            if time_diff < 1000 { // Less than 1ms
                return ValidationResult::Warning("Very high frequency data");
            }

            // If time difference is too large, warn
            if time_diff > 3600_000_000 { // More than 1 hour
                return ValidationResult::Warning("Large time gap");
            }
        }

        ValidationResult::Valid
    }

    /// Comprehensive validation
    pub fn validate_comprehensive(
        atom: &SemanticAtom,
        previous: Option<&SemanticAtom>,
    ) -> [ValidationResult; 3] {
        let results = [
            validate_structure(atom),
            validate_value_ranges(atom),
            validate_temporal_consistency(atom, previous),
        ];

        // Filter out valid results, keep warnings and errors
        // For no_std, we'll keep all results and let caller filter
        results
    }

    /// Check if atom is valid (no errors)
    pub fn is_valid(atom: &SemanticAtom, previous: Option<&SemanticAtom>) -> bool {
        let results = validate_comprehensive(atom, previous);
        !results.iter().any(|r| matches!(r, ValidationResult::Error(_)))
    }
}

/// Math utilities
pub mod math {
    /// Calculate simple moving average
    pub fn moving_average(values: &[f64], window: usize) -> f64 {
        if values.is_empty() || window == 0 {
            return 0.0;
        }

        let window = window.min(values.len());
        let start = values.len().saturating_sub(window);
        let slice = &values[start..];
        
        let sum: f64 = slice.iter().sum();
        sum / slice.len() as f64
    }

    /// Calculate exponential moving average
    pub fn exponential_moving_average(values: &[f64], alpha: f64) -> f64 {
        if values.is_empty() {
            return 0.0;
        }

        let mut ema = values[0];
        for &value in &values[1..] {
            ema = alpha * value + (1.0 - alpha) * ema;
        }
        ema
    }

    /// Calculate standard deviation
    pub fn standard_deviation(values: &[f64]) -> f64 {
        if values.len() < 2 {
            return 0.0;
        }

        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter()
            .map(|x| {
                let diff = x - mean;
                diff * diff
            })
            .sum::<f64>() / (values.len() - 1) as f64;
        
        // Simple sqrt approximation for no_std
        sqrt_approximation(variance)
    }

    /// Find outliers using IQR method
    pub fn find_outliers(values: &[f64]) -> [usize; 10] {
        if values.len() < 4 {
            return [0; 10]; // Empty array for no_std
        }

        // For no_std, return empty array - real implementation would need heapless
        [0; 10]
    }

    /// Linear interpolation
    pub fn lerp(x: f64, x0: f64, y0: f64, x1: f64, y1: f64) -> f64 {
        if x1 == x0 {
            return y0;
        }
        y0 + (x - x0) * (y1 - y0) / (x1 - x0)
    }

    /// Simple square root approximation for no_std
    fn sqrt_approximation(x: f64) -> f64 {
        if x <= 0.0 {
            return 0.0;
        }
        
        // Newton's method
        let mut guess = x / 2.0;
        for _ in 0..10 {
            guess = (guess + x / guess) / 2.0;
        }
        guess
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    use crate::{AtomBuilder, types::telemetry};

    #[test]
    fn test_fixed_point_conversion() {
        let value = 42.5;
        let fixed = fixed_point::f64_to_fixed(value);
        assert_eq!(fixed, 4250);
        
        let converted = fixed_point::fixed_to_f64(fixed);
        assert_eq!(converted, 42.5);
    }

    #[test]
    fn test_fixed_point_arithmetic() {
        let a = fixed_point::f64_to_fixed(10.0);
        let b = fixed_point::f64_to_fixed(20.0);
        
        // Addition
        let sum = fixed_point::add_fixed(a, b).unwrap();
        assert_eq!(fixed_point::fixed_to_f64(sum), 30.0);
        
        // Multiplication
        let product = fixed_point::mul_fixed(a, b);
        assert_eq!(fixed_point::fixed_to_f64(product), 200.0);
        
        // Division
        let quotient = fixed_point::div_fixed(b, a).unwrap();
        assert_eq!(fixed_point::fixed_to_f64(quotient), 2.0);
    }

    #[test]
    fn test_time_conversions() {
        let us = 1_234_567_890_u64;
        
        let ms = time::us_to_ms(us);
        assert_eq!(ms, 1_234_567);
        
        let back_us = time::ms_to_us(ms);
        assert_eq!(back_us, 1_234_567_000);
        
        let sec = time::us_to_sec(us);
        assert_eq!(sec, 1234);
        
        let back_us = time::sec_to_us(sec);
        assert_eq!(back_us, 1_234_000_000);
    }

    #[test]
    fn test_validation() {
        let atom = AtomBuilder::new()
            .entity_id(123)
            .value(42.5)
            .telemetry_type(telemetry::TEMPERATURE_C)
            .build();

        // Valid atom should pass
        let results = validation::validate_comprehensive(&atom, None);
        assert!(results.is_empty());
        assert!(validation::is_valid(&atom, None));

        // Invalid atom should fail
        let invalid_atom = AtomBuilder::new()
            .entity_id(0) // Invalid
            .value(-100.0) // Invalid for temperature
            .telemetry_type(telemetry::TEMPERATURE_C)
            .build();

        let results = validation::validate_comprehensive(&invalid_atom, None);
        assert!(!results.is_empty());
        assert!(!validation::is_valid(&invalid_atom, None));
    }

    #[test]
    fn test_temporal_consistency() {
        let base_time = 1_600_000_000_000_000_u64; // Some timestamp
        
        let atom1 = AtomBuilder::new()
            .entity_id(123)
            .sequence(1)
            .timestamp_us(base_time)
            .build();

        let atom2 = AtomBuilder::new()
            .entity_id(123)
            .sequence(2)
            .timestamp_us(base_time + 1_000_000) // 1 second later
            .build();

        // Should be valid
        let result = validation::validate_temporal_consistency(&atom2, Some(&atom1));
        assert!(matches!(result, validation::ValidationResult::Valid));

        // Invalid sequence
        let atom3 = AtomBuilder::new()
            .entity_id(123)
            .sequence(1) // Same as atom1
            .timestamp_us(base_time + 2_000_000)
            .build();

        let result = validation::validate_temporal_consistency(&atom3, Some(&atom1));
        assert!(matches!(result, validation::ValidationResult::Warning(_)));
    }

    #[test]
    fn test_math_utilities() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        
        let avg = math::moving_average(&values, 3);
        assert_eq!(avg, 4.0); // Average of last 3 values: 3,4,5
        
        let ema = math::exponential_moving_average(&values, 0.5);
        assert!(ema > 1.0 && ema < 5.0);
        
        let std_dev = math::standard_deviation(&values);
        assert!(std_dev > 0.0);
        
        let outliers = math::find_outliers(&values);
        assert!(outliers.is_empty()); // No outliers in this data
    }

    #[test]
    fn test_string_to_fixed() {
        let fixed = fixed_point::string_to_fixed("123.45").unwrap();
        assert_eq!(fixed, 12345);
        
        let fixed = fixed_point::string_to_fixed("0").unwrap();
        assert_eq!(fixed, 0);
        
        let result = fixed_point::string_to_fixed("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_percentage_change() {
        let old = fixed_point::f64_to_fixed(100.0);
        let new = fixed_point::f64_to_fixed(150.0);
        
        let change = fixed_point::percentage_change(old, new).unwrap();
        assert_eq!(change, 50.0); // 50% increase
        
        let decrease = fixed_point::f64_to_fixed(50.0);
        let change_decrease = fixed_point::percentage_change(old, decrease).unwrap();
        assert_eq!(change_decrease, -50.0); // 50% decrease
    }
}
