const proptest = require('proptest');

proptest! {
    #[test]
    fn test_property_example(input: Vec<u8>) {
        // Example property: the length of the input should be non-negative
        assert!(input.len() >= 0);
    }
}