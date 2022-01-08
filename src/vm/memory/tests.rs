use super::*;
use crate::word_vec;

mod correct_operation {
    use super::*;
    use pretty_assertions::assert_eq;

    macro_rules! correct_operation {
        ($method:ident, $stack:expr, $expected_stack: expr) => {
            #[test]
            fn $method() {
                let mut mem = unsafe { Memory::from_raw_parts($stack, 0, 0, 0, 0, 0) };
                mem.$method().unwrap();
                let mem_expected =
                    unsafe { Memory::from_raw_parts($expected_stack, 0, 0, 1, 0, 0) };
                assert_eq!(mem, mem_expected);
            }
        };
        ($method:ident, $stack:expr, $expected_stack: expr, $name: ident) => {
            #[test]
            fn $name() {
                let mut mem = unsafe { Memory::from_raw_parts($stack, 0, 0, 0, 0, 0) };
                mem.$method().unwrap();
                let mem_expected =
                    unsafe { Memory::from_raw_parts($expected_stack, 0, 0, 1, 0, 0) };
                assert_eq!(mem, mem_expected);
            }
        };
    }

    // Arithmetic
    correct_operation!(add, word_vec![1, 2], word_vec![3]);
    correct_operation!(sub, word_vec![1, 2], word_vec![1]);
    correct_operation!(mul, word_vec![4, 2], word_vec![8]);
    correct_operation!(div, word_vec![2, 6], word_vec![3], div_no_remainer);
    correct_operation!(div, word_vec![2, 7], word_vec![3], div_remainer);
    correct_operation!(rem, word_vec![2, 6], word_vec![0], rem_zero);
    correct_operation!(rem, word_vec![3, 7], word_vec![1], rem_nonzero);
    correct_operation!(neg, word_vec![1, 2], word_vec![1, -2]);
    correct_operation!(inc, word_vec![1], word_vec![2]);
    correct_operation!(dec, word_vec![1], word_vec![0]);
    correct_operation!(abs, word_vec![1], word_vec![1], abs_positive);
    correct_operation!(abs, word_vec![-1], word_vec![1], abs_negative);

    // Logic
    correct_operation!(equal, word_vec![5, 4], word_vec![false], not_eq);
    correct_operation!(equal, word_vec![4, 4], word_vec![true]);
    correct_operation!(gt, word_vec![4, 4], word_vec![false], gt_equal);
    correct_operation!(gt, word_vec![5, 4], word_vec![false], gt_smaller);
    correct_operation!(gt, word_vec![4, 5], word_vec![true], gt_bigger);
    correct_operation!(lt, word_vec![4, 4], word_vec![false], lt_equal);
    correct_operation!(lt, word_vec![5, 4], word_vec![true], lt_smaller);
    correct_operation!(lt, word_vec![4, 5], word_vec![false], lt_bigger);
    correct_operation!(and, word_vec![true, true], word_vec![true], and_true_true);
    correct_operation!(
        and,
        word_vec![false, true],
        word_vec![false],
        and_false_true
    );
    correct_operation!(
        and,
        word_vec![false, false],
        word_vec![false],
        and_false_false
    );
    correct_operation!(or, word_vec![true, true], word_vec![true], or_true_true);
    correct_operation!(or, word_vec![false, true], word_vec![true], or_false_true);
    correct_operation!(
        or,
        word_vec![false, false],
        word_vec![false],
        or_false_false
    );
    correct_operation!(not, word_vec![true], word_vec![false], not_true);
    correct_operation!(not, word_vec![false], word_vec![true], not_false);
}

mod error {
    use super::*;
    use Error::{NotEnoughValues, TypeMismatch};

    macro_rules! erroneous_operation {
        ($method:ident, $stack:expr, $expected_err: expr) => {
            #[test]
            fn $method() {
                let mut mem = unsafe { Memory::from_raw_parts($stack, 0, 0, 0, 0, 0) };

                assert_eq!(mem.$method().unwrap_err(), $expected_err);
            }
        };
        ($method:ident, $stack:expr, $expected_err: expr, $name: ident) => {
            #[test]
            fn $name() {
                let mut mem = unsafe { Memory::from_raw_parts($stack, 0, 0, 0, 0, 0) };

                assert_eq!(mem.$method().unwrap_err(), $expected_err);
            }
        };
    }

    mod type_mismatch {
        use super::*;
        use pretty_assertions::assert_eq;

        // Arithmetic
        erroneous_operation!(add, word_vec![1, true], TypeMismatch);
        erroneous_operation!(sub, word_vec![1, true], TypeMismatch);
        erroneous_operation!(mul, word_vec![1, true], TypeMismatch);
        erroneous_operation!(div, word_vec![1, true], TypeMismatch);
        erroneous_operation!(rem, word_vec![1, true], TypeMismatch);
        erroneous_operation!(neg, word_vec![1, true], TypeMismatch);
        erroneous_operation!(inc, word_vec![1, true], TypeMismatch);
        erroneous_operation!(dec, word_vec![1, true], TypeMismatch);
        erroneous_operation!(abs, word_vec![1, true], TypeMismatch);

        // Logic
        erroneous_operation!(equal, word_vec![1, true], TypeMismatch);
        erroneous_operation!(gt, word_vec![1, true], TypeMismatch);
        erroneous_operation!(lt, word_vec![1, true], TypeMismatch);
        erroneous_operation!(and, word_vec![1, true], TypeMismatch);
        erroneous_operation!(or, word_vec![1, true], TypeMismatch);
        erroneous_operation!(not, word_vec![1], TypeMismatch);
    }

    mod not_enough_values {
        use super::*;
        use pretty_assertions::assert_eq;

        // Arithmetic
        erroneous_operation!(add, word_vec![1], NotEnoughValues);
        erroneous_operation!(sub, word_vec![1], NotEnoughValues);
        erroneous_operation!(mul, word_vec![1], NotEnoughValues);
        erroneous_operation!(div, word_vec![1], NotEnoughValues);
        erroneous_operation!(rem, word_vec![1], NotEnoughValues);
        erroneous_operation!(neg, word_vec![], NotEnoughValues);
        erroneous_operation!(inc, word_vec![], NotEnoughValues);
        erroneous_operation!(dec, word_vec![], NotEnoughValues);
        erroneous_operation!(abs, word_vec![], NotEnoughValues);

        // Logic
        erroneous_operation!(equal, word_vec![1], NotEnoughValues);
        erroneous_operation!(gt, word_vec![1], NotEnoughValues);
        erroneous_operation!(lt, word_vec![1], NotEnoughValues);
        erroneous_operation!(and, word_vec![true], NotEnoughValues);
        erroneous_operation!(or, word_vec![true], NotEnoughValues);
        erroneous_operation!(not, word_vec![], NotEnoughValues);
    }
}

mod data_flow {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn push_external_data() {
        let mut mem = unsafe { Memory::from_raw_parts(word_vec![], 0, 0, 0, 0, 0) };
        mem.push_external(Word::Numeric(0)).unwrap();

        let mem_expected = unsafe { Memory::from_raw_parts(word_vec![0], 0, 0, 1, 0, 0) };

        assert_eq!(mem, mem_expected);
    }

    #[test]
    fn pop_external_data() {
        let mut mem = unsafe { Memory::from_raw_parts(word_vec![2, 6], 0, 0, 0, 0, 0) };

        assert_eq!(mem.pop_external(), Ok(Word::Numeric(6)));
        assert_eq!(mem.pop_external(), Ok(Word::Numeric(2)));
        assert_eq!(mem.pop_external(), Err(Error::NotEnoughValues));

        let mem_expected = unsafe { Memory::from_raw_parts(word_vec![], 0, 0, 2, 0, 0) };

        assert_eq!(mem, mem_expected);
    }

    #[test]
    fn pop_external_no_inc() {
        let mut mem = unsafe { Memory::from_raw_parts(word_vec![2, 6], 0, 0, 0, 0, 0) };

        assert_eq!(mem.pop_external_no_pc_inc(), Ok(Word::Numeric(6)));
        assert_eq!(mem.pop_external_no_pc_inc(), Ok(Word::Numeric(2)));
        assert_eq!(mem.pop_external_no_pc_inc(), Err(Error::NotEnoughValues));

        let mem_expected = unsafe { Memory::from_raw_parts(word_vec![], 0, 0, 0, 0, 0) };

        assert_eq!(mem, mem_expected);
    }

    #[test]
    fn push_local_data() {
        let mut mem = unsafe { Memory::from_raw_parts(word_vec![2, 6, 8], 1, 0, 0, 0, 2) };

        mem.push_local(0).unwrap();
        mem.push_local(1).unwrap();

        let mem_expected =
            unsafe { Memory::from_raw_parts(word_vec![2, 6, 8, 6, 8], 1, 0, 2, 0, 2) };

        assert_eq!(mem, mem_expected);
    }

    #[test]
    fn pop_local_data() {
        let mut mem = unsafe { Memory::from_raw_parts(word_vec![2, 6, 8, 16,], 0, 0, 0, 0, 2) };

        mem.pop_local(0).unwrap();
        mem.pop_local(1).unwrap();

        let mem_expected = unsafe { Memory::from_raw_parts(word_vec![16, 8], 0, 0, 2, 0, 2) };

        assert_eq!(mem, mem_expected);
    }

    #[test]
    fn push_argument_data() {
        let mut mem = unsafe { Memory::from_raw_parts(word_vec![2, 6, 8], 0, 0, 0, 3, 0) };

        mem.push_argument(1).unwrap();
        mem.push_argument(2).unwrap();

        let mem_expected =
            unsafe { Memory::from_raw_parts(word_vec![2, 6, 8, 6, 8], 0, 0, 2, 3, 0) };

        assert_eq!(mem, mem_expected);
    }

    #[test]
    fn pop_argument_data() {
        let mut mem = unsafe { Memory::from_raw_parts(word_vec![2, 6, 8, 16,], 0, 0, 0, 2, 0) };

        mem.pop_argument(0).unwrap();
        mem.pop_argument(1).unwrap();

        let mem_expected = unsafe { Memory::from_raw_parts(word_vec![16, 8], 0, 0, 2, 2, 0) };

        assert_eq!(mem, mem_expected);
    }
}

mod control_flow {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn call() {
        let mut mem = unsafe { Memory::from_raw_parts(word_vec![2, true], 0, 1, 4, 0, 0) };

        mem.call(16, 2).unwrap();

        let mem_expected =
            unsafe { Memory::from_raw_parts(word_vec![2, true, 5, 0, 1, 0, 0], 7, 0, 16, 2, 0) };

        assert_eq!(mem, mem_expected);
    }
    #[test]
    fn call_no_args() {
        let mut mem = unsafe { Memory::from_raw_parts(word_vec![], 0, 0, 4, 0, 0) };

        mem.call(16, 0).unwrap();

        let mem_expected =
            unsafe { Memory::from_raw_parts(word_vec![5, 0, 0, 0, 0], 5, 0, 16, 0, 0) };

        assert_eq!(mem, mem_expected);
    }

    #[test]
    fn fn_return() {
        let mut mem = unsafe {
            Memory::from_raw_parts(
                word_vec![
                    // locals of the previous stack frame
                    1,     // prev_local 0
                    1,     // prev_local 1
                    1,     // prev_local 2
                    2,     // arg 0
                    true,  // arg 1
                    5,     // return address
                    0,     // prev_lcl
                    0,     // prev_arg
                    0,     // prev_n_args
                    3,     // prev_n_locals
                    3,     // loc 0
                    4,     // loc 1
                    5,     // loc 2
                    6,     // loc 3
                    false, // return value of the function
                ],
                10, // lcl
                3,  // arg
                16, // pc
                2,  // n_args
                4,  // n_locals
            )
        };

        mem.fn_return().unwrap();

        let mem_expected = unsafe {
            Memory::from_raw_parts(
                word_vec![
                    1,     // local 0
                    1,     // local 1
                    1,     // local 2
                    false, // return value of the function
                ],
                0, // arg
                0, // lcl
                5, // pc
                0, // n_args
                3, // n_locals
            )
        };

        assert_eq!(mem, mem_expected);
    }

    #[test]
    fn function() {
        let mut mem = unsafe { Memory::from_raw_parts(word_vec![2, true], 2, 0, 16, 0, 0) };

        mem.function(3).unwrap();

        let mem_expected =
            unsafe { Memory::from_raw_parts(word_vec![2, true, 0, 0, 0], 2, 0, 17, 0, 3) };

        assert_eq!(mem, mem_expected);
    }

    mod ifjmp {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn conditional_jump_successful() {
            let mut mem = unsafe { Memory::from_raw_parts(word_vec![true], 0, 0, 0, 0, 0) };

            mem.ifjmp(10).unwrap();

            let mem_expected = unsafe { Memory::from_raw_parts(word_vec![], 0, 0, 10, 0, 0) };

            assert_eq!(mem, mem_expected);
        }

        #[test]
        fn conditional_jump_unsuccessful() {
            let mut mem = unsafe { Memory::from_raw_parts(word_vec![false], 0, 0, 0, 0, 0) };

            mem.ifjmp(10).unwrap();

            let mem_expected = unsafe { Memory::from_raw_parts(word_vec![], 0, 0, 1, 0, 0) };

            assert_eq!(mem, mem_expected);
        }

        #[test]
        fn type_mismatch() {
            let mut mem = unsafe { Memory::from_raw_parts(word_vec![1], 0, 0, 0, 0, 0) };
            assert_eq!(mem.ifjmp(10), Err(Error::TypeMismatch));
        }

        #[test]
        fn empty_stack() {
            let mut mem = unsafe { Memory::from_raw_parts(Vec::<Word>::new(), 0, 0, 0, 0, 0) };
            assert_eq!(mem.ifjmp(10), Err(Error::NotEnoughValues));
        }
    }
}
