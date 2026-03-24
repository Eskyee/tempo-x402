#[cfg(test)]
mod tests {
    use crate::validation::run_consistency_check;

    #[test]
    fn test_consistency_check_passes() {
        // The implementation in validation.rs runs the consistency check
        let result = run_consistency_check();
        assert!(result.is_ok(), "Consistency check should pass, got: {:?}", result.err());
    }
}
