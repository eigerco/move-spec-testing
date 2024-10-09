module TestAccount::BinaryReplacement {
    fun is_x_eq_to_zero(x: u64): bool {
        if (x == 0)
            return true;

        false
    }

    fun is_zero_eq_to_x(x: u64): bool {
        if (0 == x)
            return true;

        false
    }

    #[test]
    fun test_is_zero() {
        assert!(is_x_eq_to_zero(0), 0);
        assert!(!is_x_eq_to_zero(1), 0);
        assert!(is_zero_eq_to_x(0), 0);
        assert!(!is_zero_eq_to_x(2), 0);
    }

    // If we try to combine above two functions into one function,
    // the move-mutation-test won't be able to kill these mutants:
    // and that would be a clear indication of silly code that has
    // one identical reduntant check.
    fun is_zero_silly_code(x: u64): bool {
        if (0 == x)
            return true;

        // Another check which does the same is silly:
        if (x == 0)
            return true;

        false
    }

    #[test]
    fun test_is_zero_silly() {
        assert!(is_zero_silly_code(0), 0);
        assert!(!is_zero_silly_code(1), 0);
    }

    fun is_x_neq_to_zero(x: u64): bool {
        x != 0
    }

    #[test]
    fun test_is_x_neq_to_zero() {
        assert!(is_x_neq_to_zero(3), 0);
        assert!(!is_x_neq_to_zero(0), 0);
    }

    fun is_zero_neq_to_x(x: u64): bool {
        0 != x
    }

    #[test]
    fun test_is_zero_neq_to_x() {
        assert!(is_zero_neq_to_x(3), 0);
        assert!(!is_zero_neq_to_x(0), 0);
    }

    fun is_x_gt_zero(x: u64): bool {
        x > 0
    }

    #[test]
    fun test_is_x_gt_zero() {
        assert!(is_x_gt_zero(4), 0);
        assert!(!is_x_gt_zero(0), 0);
    }

    fun is_zero_lt_x(x: u64): bool {
        0 < x
    }

    #[test]
    fun test_is_zero_lt_x() {
        assert!(is_zero_lt_x(3), 0);
        assert!(!is_zero_lt_x(0), 0);
    }
}
