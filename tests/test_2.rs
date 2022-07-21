fn neg_abs(mut x: i32) -> i32 {
    if x > 0 {
        x = -1 * x;
    }

    assert!(x <= 0);
    return x;
}

// fn abs(mut x: i32) -> i32 {
//     if x < 0 {
//         x = -1 * x;
//     }
//     return x;
// }

// fn plus(a: i32, b:i32) -> i32 {
//     return a + b;
// }

fn main() {
    println!("{}", neg_abs(2));
    // println!("{}", abs(-2));
    // println!("{}", plus(-2, 2));
}

// clear && RUSTC_LOG=rustc_symbolic_exec=debug rustc +stage1 test_2.rs && ./test_2

// ********************************************************************

// (declare-fun panic () Bool)
// (declare-fun node_common_end () Bool)
// (declare-fun _7_1 () Bool)
// (declare-fun node_5 () Bool)
// (declare-fun _1 () Int)
// (declare-fun _0 () Int)
// (declare-fun node_4 () Bool)
// (declare-fun _4 () Bool)
// (declare-fun node_3 () Bool)
// (declare-fun _7_0 () Int)
// (declare-fun node_2 () Bool)
// (declare-fun _6 () Int)
// (declare-fun node_1 () Bool)
// (declare-fun _5 () Int)
// (declare-fun node_0 () Bool)
// (assert (= node_common_end (=> (or false true true) (and (not panic) true))))
// (assert (= node_5 (=> (or false (= _7_1 true)) (and true node_common_end))))
// (assert (let ((a!1 (=> (or false true true)
//                (=> (= panic false) (=> (= _0 _1) (and true node_common_end))))))
//   (= node_4 a!1)))
// (assert (= node_3 (=> (or false (= _4 false)) (and true node_4))))
// (assert (= node_2 (=> (or false (= _7_1 false)) (=> (= _1 _7_0) (and true node_4)))))
// (assert (let ((a!1 (* (ite (bvslt #xffffffff #x00000000)
//                    (- (bv2int #xffffffff) 4294967296)
//                    (bv2int #xffffffff))
//               _6)))
// (let ((a!2 (> a!1
//               (ite (bvslt #x7fffffff #x00000000)
//                    (- (bv2int #x7fffffff) 4294967296)
//                    (bv2int #x7fffffff))))
//       (a!3 (< a!1
//               (ite (bvslt #x80000000 #x00000000)
//                    (- (bv2int #x80000000) 4294967296)
//                    (bv2int #x80000000)))))
// (let ((a!4 (=> (and (= _7_0 a!1) (= _7_1 (or a!2 a!3)))
//                (and true node_5 node_2))))
//   (= node_1 (=> (or false (= _4 true)) (=> (= _6 _1) a!4)))))))
// (assert (let ((a!1 (> _5
//               (ite (bvslt #x00000000 #x00000000)
//                    (- (bv2int #x00000000) 4294967296)
//                    (bv2int #x00000000)))))
// (let ((a!2 (=> true (=> (= _5 _1) (=> (= _4 a!1) (and true node_3 node_1))))))
//   (= node_0 a!2))))
// (assert (let ((a!1 (>= _1
//                (ite (bvslt #x80000000 #x00000000)
//                     (- (bv2int #x80000000) 4294967296)
//                     (bv2int #x80000000))))
//       (a!2 (<= _1
//                (ite (bvslt #x7fffffff #x00000000)
//                     (- (bv2int #x7fffffff) 4294967296)
//                     (bv2int #x7fffffff)))))
//   (and a!1 a!2)))
// (assert (not node_0))

// ********************************************************************

// (declare-fun panic () Bool)
// (declare-fun node_common_end () Bool)
// (declare-fun _7_1 () Bool)
// (declare-fun node_5 () Bool)
// (declare-fun _1 () Int)
// (declare-fun _0 () Int)
// (declare-fun node_4 () Bool)
// (declare-fun _4 () Bool)
// (declare-fun node_3 () Bool)
// (declare-fun _7_0 () Int)
// (declare-fun node_2 () Bool)
// (declare-fun _6 () Int)
// (declare-fun node_1 () Bool)
// (declare-fun _5 () Int)
// (declare-fun node_0 () Bool)
// (assert (= node_common_end (=> (or false true true) (and (not panic) true))))
// (assert (= node_5 (=> (or false (= _7_1 true)) (and true node_common_end))))
// (assert (let ((a!1 (=> (or false true true)
//                (=> (= panic false) (=> (= _0 _1) (and true node_common_end))))))
//   (= node_4 a!1)))
// (assert (= node_3 (=> (or false (= _4 false)) (and true node_4))))
// (assert (= node_2 (=> (or false (= _7_1 false)) (=> (= _1 _7_0) (and true node_4)))))
// (assert (let ((a!1 (* (ite (bvslt #xffffffff #x00000000)
//                    (- (bv2int #xffffffff) 4294967296)
//                    (bv2int #xffffffff))
//               _6)))
// (let ((a!2 (> a!1
//               (ite (bvslt #x7fffffff #x00000000)
//                    (- (bv2int #x7fffffff) 4294967296)
//                    (bv2int #x7fffffff))))
//       (a!3 (< a!1
//               (ite (bvslt #x80000000 #x00000000)
//                    (- (bv2int #x80000000) 4294967296)
//                    (bv2int #x80000000)))))
// (let ((a!4 (=> (and (= _7_0 a!1) (= _7_1 (or a!2 a!3)))
//                (and true node_5 node_2))))
//   (= node_1 (=> (or false (= _4 true)) (=> (= _6 _1) a!4)))))))
// (assert (let ((a!1 (< _5
//               (ite (bvslt #x00000000 #x00000000)
//                    (- (bv2int #x00000000) 4294967296)
//                    (bv2int #x00000000)))))
// (let ((a!2 (=> true (=> (= _5 _1) (=> (= _4 a!1) (and true node_3 node_1))))))
//   (= node_0 a!2))))
// (assert (let ((a!1 (>= _1
//                (ite (bvslt #x80000000 #x00000000)
//                     (- (bv2int #x80000000) 4294967296)
//                     (bv2int #x80000000))))
//       (a!2 (<= _1
//                (ite (bvslt #x7fffffff #x00000000)
//                     (- (bv2int #x7fffffff) 4294967296)
//                     (bv2int #x7fffffff)))))
//   (and a!1 a!2)))
// (assert (not node_0))
