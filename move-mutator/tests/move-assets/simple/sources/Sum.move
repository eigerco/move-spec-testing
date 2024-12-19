module TestAccount::Sum {
    fun sum(x: u128, y: u128): u128 {
        let sum_r = x + y;

        sum_r
    }

    #[test]
    fun sum_test() {
        assert!(sum(2, 2) == 4, 0);
        assert!(sum(0, 5) == 5, 0);
        assert!(sum(100, 0) == 100, 0);
        assert!(sum(0, 0) == 0, 0);
    }
}
