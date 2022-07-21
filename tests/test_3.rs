fn unsat_safe(x: i32) -> () {
    assert!(x == x);
}

fn sat_unsafe(x: i32) -> () {
    assert!(x < 13);
}

fn main() {
    unsat_safe(12);
    sat_unsafe(-250);
}

// clear && RUSTC_LOG=rustc_symbolic_exec=debug rustc +stage1 test_3.rs && ./test_3

// ********************************************************************

// (declare-fun panic () Bool)
// (declare-fun node_common_end () Bool)
// (declare-fun node_3 () Bool)
// (declare-fun _3 () Bool)
// (declare-fun node_2 () Bool)
// (declare-fun node_1 () Bool)
// (declare-fun _4 () Bool)
// (declare-fun _6 () Int)
// (declare-fun _5 () Int)
// (declare-fun _1 () Int)
// (declare-fun node_0 () Bool)
// (assert (= node_common_end (=> (or false true true) (and (not panic) true))))
// (assert (= node_3 (=> (or false true) (and true node_common_end))))
// (assert (= node_2
//    (=> (or false (= _3 false)) (=> (= panic false) (and true node_common_end)))))
// (assert (= node_1 (=> (or false (= _3 true)) (and true node_3))))
// (assert (let ((a!1 (=> (= _4 (= _5 _6)) (=> (= _3 (not _4)) (and true node_1 node_2)))))
// (let ((a!2 (=> true (=> (= _5 _1) (=> (= _6 _1) a!1)))))
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

// (declare-fun node_common_end () Bool)
// (declare-fun node_3 () Bool)
// (declare-fun _3 () Bool)
// (declare-fun node_2 () Bool)
// (declare-fun node_1 () Bool)
// (declare-fun _4 () Bool)
// (declare-fun _5 () Int)
// (declare-fun _1 () Int)
// (declare-fun node_0 () Bool)
// (assert (= node_common_end (=> (or false true true) (and (not panic) true))))
// (assert (= node_3 (=> (or false true) (and true node_common_end))))
// (assert (= node_2
//    (=> (or false (= _3 false)) (=> (= panic false) (and true node_common_end)))))
// (assert (= node_1 (=> (or false (= _3 true)) (and true node_3))))
// (assert (let ((a!1 (< _5
//               (ite (bvslt #x0000000d #x00000000)
//                    (- (bv2int #x0000000d) 4294967296)
//                    (bv2int #x0000000d)))))
// (let ((a!2 (=> (= _4 a!1) (=> (= _3 (not _4)) (and true node_1 node_2)))))
//   (= node_0 (=> true (=> (= _5 _1) a!2))))))
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
