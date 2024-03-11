use tracing::{
    field,
    span::{self, Attributes},
    Event, Metadata,
};

use crate::recording;

pub(crate) trait DispatchProxy {
    type Output;

    // This function matches the values in the provided vec based on the length. It then creates
    // a fixed size array which is passed to the `dispatch` method on this same trait which
    // contains the custom implementation necessary to record the trace with these fields.
    // This is necessary because `tracing` requires a fixed size array. For this reason, we can
    // only support up to a limited number of fields.
    // This also explains why this function has too many lines and needs the clippy allow below.
    #[allow(clippy::too_many_lines)]
    fn dispatch_values(
        &self,
        values: Vec<(field::Field, Option<&dyn tracing::Value>)>,
    ) -> Self::Output {
        match *values.as_slice() {
            [] => self.dispatch([]),
            [(ref f0, v0)] => self.dispatch([(f0, v0)]),
            [(ref f0, v0), (ref f1, v1)] => self.dispatch([(f0, v0), (f1, v1)]),
            [(ref f0, v0), (ref f1, v1), (ref f2, v2)] => {
                self.dispatch([(f0, v0), (f1, v1), (f2, v2)])
            }
            [(ref f0, v0), (ref f1, v1), (ref f2, v2), (ref f3, v3)] => {
                self.dispatch([(f0, v0), (f1, v1), (f2, v2), (f3, v3)])
            }
            [(ref f0, v0), (ref f1, v1), (ref f2, v2), (ref f3, v3), (ref f4, v4)] => {
                self.dispatch([(f0, v0), (f1, v1), (f2, v2), (f3, v3), (f4, v4)])
            }
            [(ref f0, v0), (ref f1, v1), (ref f2, v2), (ref f3, v3), (ref f4, v4), (ref f5, v5)] => {
                self.dispatch([(f0, v0), (f1, v1), (f2, v2), (f3, v3), (f4, v4), (f5, v5)])
            }
            [(ref f0, v0), (ref f1, v1), (ref f2, v2), (ref f3, v3), (ref f4, v4), (ref f5, v5), (ref f6, v6)] => {
                self.dispatch([
                    (f0, v0),
                    (f1, v1),
                    (f2, v2),
                    (f3, v3),
                    (f4, v4),
                    (f5, v5),
                    (f6, v6),
                ])
            }
            [(ref f0, v0), (ref f1, v1), (ref f2, v2), (ref f3, v3), (ref f4, v4), (ref f5, v5), (ref f6, v6), (ref f7, v7)] => {
                self.dispatch([
                    (f0, v0),
                    (f1, v1),
                    (f2, v2),
                    (f3, v3),
                    (f4, v4),
                    (f5, v5),
                    (f6, v6),
                    (f7, v7),
                ])
            }
            [(ref f0, v0), (ref f1, v1), (ref f2, v2), (ref f3, v3), (ref f4, v4), (ref f5, v5), (ref f6, v6), (ref f7, v7), (ref f8, v8)] => {
                self.dispatch([
                    (f0, v0),
                    (f1, v1),
                    (f2, v2),
                    (f3, v3),
                    (f4, v4),
                    (f5, v5),
                    (f6, v6),
                    (f7, v7),
                    (f8, v8),
                ])
            }
            [(ref f0, v0), (ref f1, v1), (ref f2, v2), (ref f3, v3), (ref f4, v4), (ref f5, v5), (ref f6, v6), (ref f7, v7), (ref f8, v8), (ref f9, v9)] => {
                self.dispatch([
                    (f0, v0),
                    (f1, v1),
                    (f2, v2),
                    (f3, v3),
                    (f4, v4),
                    (f5, v5),
                    (f6, v6),
                    (f7, v7),
                    (f8, v8),
                    (f9, v9),
                ])
            }
            [(ref f0, v0), (ref f1, v1), (ref f2, v2), (ref f3, v3), (ref f4, v4), (ref f5, v5), (ref f6, v6), (ref f7, v7), (ref f8, v8), (ref f9, v9), (ref f10, v10)] => {
                self.dispatch([
                    (f0, v0),
                    (f1, v1),
                    (f2, v2),
                    (f3, v3),
                    (f4, v4),
                    (f5, v5),
                    (f6, v6),
                    (f7, v7),
                    (f8, v8),
                    (f9, v9),
                    (f10, v10),
                ])
            }
            [(ref f0, v0), (ref f1, v1), (ref f2, v2), (ref f3, v3), (ref f4, v4), (ref f5, v5), (ref f6, v6), (ref f7, v7), (ref f8, v8), (ref f9, v9), (ref f10, v10), (ref f11, v11)] => {
                self.dispatch([
                    (f0, v0),
                    (f1, v1),
                    (f2, v2),
                    (f3, v3),
                    (f4, v4),
                    (f5, v5),
                    (f6, v6),
                    (f7, v7),
                    (f8, v8),
                    (f9, v9),
                    (f10, v10),
                    (f11, v11),
                ])
            }
            [(ref f0, v0), (ref f1, v1), (ref f2, v2), (ref f3, v3), (ref f4, v4), (ref f5, v5), (ref f6, v6), (ref f7, v7), (ref f8, v8), (ref f9, v9), (ref f10, v10), (ref f11, v11), (ref f12, v12)] => {
                self.dispatch([
                    (f0, v0),
                    (f1, v1),
                    (f2, v2),
                    (f3, v3),
                    (f4, v4),
                    (f5, v5),
                    (f6, v6),
                    (f7, v7),
                    (f8, v8),
                    (f9, v9),
                    (f10, v10),
                    (f11, v11),
                    (f12, v12),
                ])
            }
            [(ref f0, v0), (ref f1, v1), (ref f2, v2), (ref f3, v3), (ref f4, v4), (ref f5, v5), (ref f6, v6), (ref f7, v7), (ref f8, v8), (ref f9, v9), (ref f10, v10), (ref f11, v11), (ref f12, v12), (ref f13, v13)] => {
                self.dispatch([
                    (f0, v0),
                    (f1, v1),
                    (f2, v2),
                    (f3, v3),
                    (f4, v4),
                    (f5, v5),
                    (f6, v6),
                    (f7, v7),
                    (f8, v8),
                    (f9, v9),
                    (f10, v10),
                    (f11, v11),
                    (f12, v12),
                    (f13, v13),
                ])
            }
            [(ref f0, v0), (ref f1, v1), (ref f2, v2), (ref f3, v3), (ref f4, v4), (ref f5, v5), (ref f6, v6), (ref f7, v7), (ref f8, v8), (ref f9, v9), (ref f10, v10), (ref f11, v11), (ref f12, v12), (ref f13, v13), (ref f14, v14)] => {
                self.dispatch([
                    (f0, v0),
                    (f1, v1),
                    (f2, v2),
                    (f3, v3),
                    (f4, v4),
                    (f5, v5),
                    (f6, v6),
                    (f7, v7),
                    (f8, v8),
                    (f9, v9),
                    (f10, v10),
                    (f11, v11),
                    (f12, v12),
                    (f13, v13),
                    (f14, v14),
                ])
            }
            [(ref f0, v0), (ref f1, v1), (ref f2, v2), (ref f3, v3), (ref f4, v4), (ref f5, v5), (ref f6, v6), (ref f7, v7), (ref f8, v8), (ref f9, v9), (ref f10, v10), (ref f11, v11), (ref f12, v12), (ref f13, v13), (ref f14, v14), (ref f15, v15)] => {
                self.dispatch([
                    (f0, v0),
                    (f1, v1),
                    (f2, v2),
                    (f3, v3),
                    (f4, v4),
                    (f5, v5),
                    (f6, v6),
                    (f7, v7),
                    (f8, v8),
                    (f9, v9),
                    (f10, v10),
                    (f11, v11),
                    (f12, v12),
                    (f13, v13),
                    (f14, v14),
                    (f15, v15),
                ])
            }
            [(ref f0, v0), (ref f1, v1), (ref f2, v2), (ref f3, v3), (ref f4, v4), (ref f5, v5), (ref f6, v6), (ref f7, v7), (ref f8, v8), (ref f9, v9), (ref f10, v10), (ref f11, v11), (ref f12, v12), (ref f13, v13), (ref f14, v14), (ref f15, v15), (ref f16, v16)] => {
                self.dispatch([
                    (f0, v0),
                    (f1, v1),
                    (f2, v2),
                    (f3, v3),
                    (f4, v4),
                    (f5, v5),
                    (f6, v6),
                    (f7, v7),
                    (f8, v8),
                    (f9, v9),
                    (f10, v10),
                    (f11, v11),
                    (f12, v12),
                    (f13, v13),
                    (f14, v14),
                    (f15, v15),
                    (f16, v16),
                ])
            }
            [(ref f0, v0), (ref f1, v1), (ref f2, v2), (ref f3, v3), (ref f4, v4), (ref f5, v5), (ref f6, v6), (ref f7, v7), (ref f8, v8), (ref f9, v9), (ref f10, v10), (ref f11, v11), (ref f12, v12), (ref f13, v13), (ref f14, v14), (ref f15, v15), (ref f16, v16), (ref f17, v17)] => {
                self.dispatch([
                    (f0, v0),
                    (f1, v1),
                    (f2, v2),
                    (f3, v3),
                    (f4, v4),
                    (f5, v5),
                    (f6, v6),
                    (f7, v7),
                    (f8, v8),
                    (f9, v9),
                    (f10, v10),
                    (f11, v11),
                    (f12, v12),
                    (f13, v13),
                    (f14, v14),
                    (f15, v15),
                    (f16, v16),
                    (f17, v17),
                ])
            }
            [(ref f0, v0), (ref f1, v1), (ref f2, v2), (ref f3, v3), (ref f4, v4), (ref f5, v5), (ref f6, v6), (ref f7, v7), (ref f8, v8), (ref f9, v9), (ref f10, v10), (ref f11, v11), (ref f12, v12), (ref f13, v13), (ref f14, v14), (ref f15, v15), (ref f16, v16), (ref f17, v17), (ref f18, v18)] => {
                self.dispatch([
                    (f0, v0),
                    (f1, v1),
                    (f2, v2),
                    (f3, v3),
                    (f4, v4),
                    (f5, v5),
                    (f6, v6),
                    (f7, v7),
                    (f8, v8),
                    (f9, v9),
                    (f10, v10),
                    (f11, v11),
                    (f12, v12),
                    (f13, v13),
                    (f14, v14),
                    (f15, v15),
                    (f16, v16),
                    (f17, v17),
                    (f18, v18),
                ])
            }
            [(ref f0, v0), (ref f1, v1), (ref f2, v2), (ref f3, v3), (ref f4, v4), (ref f5, v5), (ref f6, v6), (ref f7, v7), (ref f8, v8), (ref f9, v9), (ref f10, v10), (ref f11, v11), (ref f12, v12), (ref f13, v13), (ref f14, v14), (ref f15, v15), (ref f16, v16), (ref f17, v17), (ref f18, v18), (ref f19, v19)] => {
                self.dispatch([
                    (f0, v0),
                    (f1, v1),
                    (f2, v2),
                    (f3, v3),
                    (f4, v4),
                    (f5, v5),
                    (f6, v6),
                    (f7, v7),
                    (f8, v8),
                    (f9, v9),
                    (f10, v10),
                    (f11, v11),
                    (f12, v12),
                    (f13, v13),
                    (f14, v14),
                    (f15, v15),
                    (f16, v16),
                    (f17, v17),
                    (f18, v18),
                    (f19, v19),
                ])
            }
            [(ref f0, v0), (ref f1, v1), (ref f2, v2), (ref f3, v3), (ref f4, v4), (ref f5, v5), (ref f6, v6), (ref f7, v7), (ref f8, v8), (ref f9, v9), (ref f10, v10), (ref f11, v11), (ref f12, v12), (ref f13, v13), (ref f14, v14), (ref f15, v15), (ref f16, v16), (ref f17, v17), (ref f18, v18), (ref f19, v19), (ref f20, v20)] => {
                self.dispatch([
                    (f0, v0),
                    (f1, v1),
                    (f2, v2),
                    (f3, v3),
                    (f4, v4),
                    (f5, v5),
                    (f6, v6),
                    (f7, v7),
                    (f8, v8),
                    (f9, v9),
                    (f10, v10),
                    (f11, v11),
                    (f12, v12),
                    (f13, v13),
                    (f14, v14),
                    (f15, v15),
                    (f16, v16),
                    (f17, v17),
                    (f18, v18),
                    (f19, v19),
                    (f20, v20),
                ])
            }
            [(ref f0, v0), (ref f1, v1), (ref f2, v2), (ref f3, v3), (ref f4, v4), (ref f5, v5), (ref f6, v6), (ref f7, v7), (ref f8, v8), (ref f9, v9), (ref f10, v10), (ref f11, v11), (ref f12, v12), (ref f13, v13), (ref f14, v14), (ref f15, v15), (ref f16, v16), (ref f17, v17), (ref f18, v18), (ref f19, v19), (ref f20, v20), (ref f21, v21)] => {
                self.dispatch([
                    (f0, v0),
                    (f1, v1),
                    (f2, v2),
                    (f3, v3),
                    (f4, v4),
                    (f5, v5),
                    (f6, v6),
                    (f7, v7),
                    (f8, v8),
                    (f9, v9),
                    (f10, v10),
                    (f11, v11),
                    (f12, v12),
                    (f13, v13),
                    (f14, v14),
                    (f15, v15),
                    (f16, v16),
                    (f17, v17),
                    (f18, v18),
                    (f19, v19),
                    (f20, v20),
                    (f21, v21),
                ])
            }
            [(ref f0, v0), (ref f1, v1), (ref f2, v2), (ref f3, v3), (ref f4, v4), (ref f5, v5), (ref f6, v6), (ref f7, v7), (ref f8, v8), (ref f9, v9), (ref f10, v10), (ref f11, v11), (ref f12, v12), (ref f13, v13), (ref f14, v14), (ref f15, v15), (ref f16, v16), (ref f17, v17), (ref f18, v18), (ref f19, v19), (ref f20, v20), (ref f21, v21), (ref f22, v22)] => {
                self.dispatch([
                    (f0, v0),
                    (f1, v1),
                    (f2, v2),
                    (f3, v3),
                    (f4, v4),
                    (f5, v5),
                    (f6, v6),
                    (f7, v7),
                    (f8, v8),
                    (f9, v9),
                    (f10, v10),
                    (f11, v11),
                    (f12, v12),
                    (f13, v13),
                    (f14, v14),
                    (f15, v15),
                    (f16, v16),
                    (f17, v17),
                    (f18, v18),
                    (f19, v19),
                    (f20, v20),
                    (f21, v21),
                    (f22, v22),
                ])
            }
            [(ref f0, v0), (ref f1, v1), (ref f2, v2), (ref f3, v3), (ref f4, v4), (ref f5, v5), (ref f6, v6), (ref f7, v7), (ref f8, v8), (ref f9, v9), (ref f10, v10), (ref f11, v11), (ref f12, v12), (ref f13, v13), (ref f14, v14), (ref f15, v15), (ref f16, v16), (ref f17, v17), (ref f18, v18), (ref f19, v19), (ref f20, v20), (ref f21, v21), (ref f22, v22), (ref f23, v23)] => {
                self.dispatch([
                    (f0, v0),
                    (f1, v1),
                    (f2, v2),
                    (f3, v3),
                    (f4, v4),
                    (f5, v5),
                    (f6, v6),
                    (f7, v7),
                    (f8, v8),
                    (f9, v9),
                    (f10, v10),
                    (f11, v11),
                    (f12, v12),
                    (f13, v13),
                    (f14, v14),
                    (f15, v15),
                    (f16, v16),
                    (f17, v17),
                    (f18, v18),
                    (f19, v19),
                    (f20, v20),
                    (f21, v21),
                    (f22, v22),
                    (f23, v23),
                ])
            }
            [(ref f0, v0), (ref f1, v1), (ref f2, v2), (ref f3, v3), (ref f4, v4), (ref f5, v5), (ref f6, v6), (ref f7, v7), (ref f8, v8), (ref f9, v9), (ref f10, v10), (ref f11, v11), (ref f12, v12), (ref f13, v13), (ref f14, v14), (ref f15, v15), (ref f16, v16), (ref f17, v17), (ref f18, v18), (ref f19, v19), (ref f20, v20), (ref f21, v21), (ref f22, v22), (ref f23, v23), (ref f24, v24)] => {
                self.dispatch([
                    (f0, v0),
                    (f1, v1),
                    (f2, v2),
                    (f3, v3),
                    (f4, v4),
                    (f5, v5),
                    (f6, v6),
                    (f7, v7),
                    (f8, v8),
                    (f9, v9),
                    (f10, v10),
                    (f11, v11),
                    (f12, v12),
                    (f13, v13),
                    (f14, v14),
                    (f15, v15),
                    (f16, v16),
                    (f17, v17),
                    (f18, v18),
                    (f19, v19),
                    (f20, v20),
                    (f21, v21),
                    (f22, v22),
                    (f23, v23),
                    (f24, v24),
                ])
            }
            [(ref f0, v0), (ref f1, v1), (ref f2, v2), (ref f3, v3), (ref f4, v4), (ref f5, v5), (ref f6, v6), (ref f7, v7), (ref f8, v8), (ref f9, v9), (ref f10, v10), (ref f11, v11), (ref f12, v12), (ref f13, v13), (ref f14, v14), (ref f15, v15), (ref f16, v16), (ref f17, v17), (ref f18, v18), (ref f19, v19), (ref f20, v20), (ref f21, v21), (ref f22, v22), (ref f23, v23), (ref f24, v24), (ref f25, v25)] => {
                self.dispatch([
                    (f0, v0),
                    (f1, v1),
                    (f2, v2),
                    (f3, v3),
                    (f4, v4),
                    (f5, v5),
                    (f6, v6),
                    (f7, v7),
                    (f8, v8),
                    (f9, v9),
                    (f10, v10),
                    (f11, v11),
                    (f12, v12),
                    (f13, v13),
                    (f14, v14),
                    (f15, v15),
                    (f16, v16),
                    (f17, v17),
                    (f18, v18),
                    (f19, v19),
                    (f20, v20),
                    (f21, v21),
                    (f22, v22),
                    (f23, v23),
                    (f24, v24),
                    (f25, v25),
                ])
            }
            [(ref f0, v0), (ref f1, v1), (ref f2, v2), (ref f3, v3), (ref f4, v4), (ref f5, v5), (ref f6, v6), (ref f7, v7), (ref f8, v8), (ref f9, v9), (ref f10, v10), (ref f11, v11), (ref f12, v12), (ref f13, v13), (ref f14, v14), (ref f15, v15), (ref f16, v16), (ref f17, v17), (ref f18, v18), (ref f19, v19), (ref f20, v20), (ref f21, v21), (ref f22, v22), (ref f23, v23), (ref f24, v24), (ref f25, v25), (ref f26, v26)] => {
                self.dispatch([
                    (f0, v0),
                    (f1, v1),
                    (f2, v2),
                    (f3, v3),
                    (f4, v4),
                    (f5, v5),
                    (f6, v6),
                    (f7, v7),
                    (f8, v8),
                    (f9, v9),
                    (f10, v10),
                    (f11, v11),
                    (f12, v12),
                    (f13, v13),
                    (f14, v14),
                    (f15, v15),
                    (f16, v16),
                    (f17, v17),
                    (f18, v18),
                    (f19, v19),
                    (f20, v20),
                    (f21, v21),
                    (f22, v22),
                    (f23, v23),
                    (f24, v24),
                    (f25, v25),
                    (f26, v26),
                ])
            }
            [(ref f0, v0), (ref f1, v1), (ref f2, v2), (ref f3, v3), (ref f4, v4), (ref f5, v5), (ref f6, v6), (ref f7, v7), (ref f8, v8), (ref f9, v9), (ref f10, v10), (ref f11, v11), (ref f12, v12), (ref f13, v13), (ref f14, v14), (ref f15, v15), (ref f16, v16), (ref f17, v17), (ref f18, v18), (ref f19, v19), (ref f20, v20), (ref f21, v21), (ref f22, v22), (ref f23, v23), (ref f24, v24), (ref f25, v25), (ref f26, v26), (ref f27, v27)] => {
                self.dispatch([
                    (f0, v0),
                    (f1, v1),
                    (f2, v2),
                    (f3, v3),
                    (f4, v4),
                    (f5, v5),
                    (f6, v6),
                    (f7, v7),
                    (f8, v8),
                    (f9, v9),
                    (f10, v10),
                    (f11, v11),
                    (f12, v12),
                    (f13, v13),
                    (f14, v14),
                    (f15, v15),
                    (f16, v16),
                    (f17, v17),
                    (f18, v18),
                    (f19, v19),
                    (f20, v20),
                    (f21, v21),
                    (f22, v22),
                    (f23, v23),
                    (f24, v24),
                    (f25, v25),
                    (f26, v26),
                    (f27, v27),
                ])
            }
            [(ref f0, v0), (ref f1, v1), (ref f2, v2), (ref f3, v3), (ref f4, v4), (ref f5, v5), (ref f6, v6), (ref f7, v7), (ref f8, v8), (ref f9, v9), (ref f10, v10), (ref f11, v11), (ref f12, v12), (ref f13, v13), (ref f14, v14), (ref f15, v15), (ref f16, v16), (ref f17, v17), (ref f18, v18), (ref f19, v19), (ref f20, v20), (ref f21, v21), (ref f22, v22), (ref f23, v23), (ref f24, v24), (ref f25, v25), (ref f26, v26), (ref f27, v27), (ref f28, v28)] => {
                self.dispatch([
                    (f0, v0),
                    (f1, v1),
                    (f2, v2),
                    (f3, v3),
                    (f4, v4),
                    (f5, v5),
                    (f6, v6),
                    (f7, v7),
                    (f8, v8),
                    (f9, v9),
                    (f10, v10),
                    (f11, v11),
                    (f12, v12),
                    (f13, v13),
                    (f14, v14),
                    (f15, v15),
                    (f16, v16),
                    (f17, v17),
                    (f18, v18),
                    (f19, v19),
                    (f20, v20),
                    (f21, v21),
                    (f22, v22),
                    (f23, v23),
                    (f24, v24),
                    (f25, v25),
                    (f26, v26),
                    (f27, v27),
                    (f28, v28),
                ])
            }
            [(ref f0, v0), (ref f1, v1), (ref f2, v2), (ref f3, v3), (ref f4, v4), (ref f5, v5), (ref f6, v6), (ref f7, v7), (ref f8, v8), (ref f9, v9), (ref f10, v10), (ref f11, v11), (ref f12, v12), (ref f13, v13), (ref f14, v14), (ref f15, v15), (ref f16, v16), (ref f17, v17), (ref f18, v18), (ref f19, v19), (ref f20, v20), (ref f21, v21), (ref f22, v22), (ref f23, v23), (ref f24, v24), (ref f25, v25), (ref f26, v26), (ref f27, v27), (ref f28, v28), (ref f29, v29)] => {
                self.dispatch([
                    (f0, v0),
                    (f1, v1),
                    (f2, v2),
                    (f3, v3),
                    (f4, v4),
                    (f5, v5),
                    (f6, v6),
                    (f7, v7),
                    (f8, v8),
                    (f9, v9),
                    (f10, v10),
                    (f11, v11),
                    (f12, v12),
                    (f13, v13),
                    (f14, v14),
                    (f15, v15),
                    (f16, v16),
                    (f17, v17),
                    (f18, v18),
                    (f19, v19),
                    (f20, v20),
                    (f21, v21),
                    (f22, v22),
                    (f23, v23),
                    (f24, v24),
                    (f25, v25),
                    (f26, v26),
                    (f27, v27),
                    (f28, v28),
                    (f29, v29),
                ])
            }
            [(ref f0, v0), (ref f1, v1), (ref f2, v2), (ref f3, v3), (ref f4, v4), (ref f5, v5), (ref f6, v6), (ref f7, v7), (ref f8, v8), (ref f9, v9), (ref f10, v10), (ref f11, v11), (ref f12, v12), (ref f13, v13), (ref f14, v14), (ref f15, v15), (ref f16, v16), (ref f17, v17), (ref f18, v18), (ref f19, v19), (ref f20, v20), (ref f21, v21), (ref f22, v22), (ref f23, v23), (ref f24, v24), (ref f25, v25), (ref f26, v26), (ref f27, v27), (ref f28, v28), (ref f29, v29), (ref f30, v30)] => {
                self.dispatch([
                    (f0, v0),
                    (f1, v1),
                    (f2, v2),
                    (f3, v3),
                    (f4, v4),
                    (f5, v5),
                    (f6, v6),
                    (f7, v7),
                    (f8, v8),
                    (f9, v9),
                    (f10, v10),
                    (f11, v11),
                    (f12, v12),
                    (f13, v13),
                    (f14, v14),
                    (f15, v15),
                    (f16, v16),
                    (f17, v17),
                    (f18, v18),
                    (f19, v19),
                    (f20, v20),
                    (f21, v21),
                    (f22, v22),
                    (f23, v23),
                    (f24, v24),
                    (f25, v25),
                    (f26, v26),
                    (f27, v27),
                    (f28, v28),
                    (f29, v29),
                    (f30, v30),
                ])
            }
            [(ref f0, v0), (ref f1, v1), (ref f2, v2), (ref f3, v3), (ref f4, v4), (ref f5, v5), (ref f6, v6), (ref f7, v7), (ref f8, v8), (ref f9, v9), (ref f10, v10), (ref f11, v11), (ref f12, v12), (ref f13, v13), (ref f14, v14), (ref f15, v15), (ref f16, v16), (ref f17, v17), (ref f18, v18), (ref f19, v19), (ref f20, v20), (ref f21, v21), (ref f22, v22), (ref f23, v23), (ref f24, v24), (ref f25, v25), (ref f26, v26), (ref f27, v27), (ref f28, v28), (ref f29, v29), (ref f30, v30), (ref f31, v31), ..] => {
                self.dispatch([
                    (f0, v0),
                    (f1, v1),
                    (f2, v2),
                    (f3, v3),
                    (f4, v4),
                    (f5, v5),
                    (f6, v6),
                    (f7, v7),
                    (f8, v8),
                    (f9, v9),
                    (f10, v10),
                    (f11, v11),
                    (f12, v12),
                    (f13, v13),
                    (f14, v14),
                    (f15, v15),
                    (f16, v16),
                    (f17, v17),
                    (f18, v18),
                    (f19, v19),
                    (f20, v20),
                    (f21, v21),
                    (f22, v22),
                    (f23, v23),
                    (f24, v24),
                    (f25, v25),
                    (f26, v26),
                    (f27, v27),
                    (f28, v28),
                    (f29, v29),
                    (f30, v30),
                    (f31, v31),
                ])
            }
        }
    }

    fn dispatch<const N: usize>(
        &self,
        values: [(&field::Field, Option<&dyn tracing::Value>); N],
    ) -> Self::Output;
}

pub(crate) struct NewSpanProxy<'a> {
    dispatch: &'a tracing::Dispatch,
    metadata: &'static Metadata<'static>,
    parent: &'a recording::Parent,
}

impl<'a> NewSpanProxy<'a> {
    pub(crate) fn new(
        dispatch: &'a tracing::Dispatch,
        metadata: &'static Metadata<'static>,
        parent: &'a recording::Parent,
    ) -> Self {
        Self {
            dispatch,
            metadata,
            parent,
        }
    }
}

impl<'a> DispatchProxy for NewSpanProxy<'a> {
    type Output = span::Id;

    fn dispatch<const N: usize>(
        &self,
        values: [(&field::Field, Option<&dyn tracing::Value>); N],
    ) -> Self::Output {
        let value_set = self.metadata.fields().value_set(&values);
        let attr = match self.parent {
            recording::Parent::Current => Attributes::new(self.metadata, &value_set),
            recording::Parent::Root => Attributes::new_root(self.metadata, &value_set),
            &recording::Parent::Explicit(parent_id) => {
                Attributes::child_of(span::Id::from_u64(parent_id), self.metadata, &value_set)
            }
        };
        self.dispatch.new_span(&attr)
    }
}

pub(crate) struct EventProxy<'a> {
    dispatch: &'a tracing::Dispatch,
    metadata: &'static Metadata<'static>,
    parent: &'a recording::Parent,
}

impl<'a> EventProxy<'a> {
    pub(crate) fn new(
        dispatch: &'a tracing::Dispatch,
        metadata: &'static Metadata<'static>,
        parent: &'a recording::Parent,
    ) -> Self {
        Self {
            dispatch,
            metadata,
            parent,
        }
    }
}

impl<'a> DispatchProxy for EventProxy<'a> {
    type Output = ();

    fn dispatch<const N: usize>(
        &self,
        values: [(&field::Field, Option<&dyn tracing::Value>); N],
    ) -> Self::Output {
        let value_set = self.metadata.fields().value_set(&values);
        let event = match self.parent {
            recording::Parent::Current => Event::new(self.metadata, &value_set),
            recording::Parent::Root => Event::new_child_of(None, self.metadata, &value_set),
            recording::Parent::Explicit(parent_id) => Event::new_child_of(
                Some(span::Id::from_u64(*parent_id)),
                self.metadata,
                &value_set,
            ),
        };
        self.dispatch.event(&event);
    }
}

pub(crate) struct RecordProxy<'a> {
    dispatch: &'a tracing::Dispatch,
    metadata: &'static Metadata<'static>,
    span_id: &'a span::Id,
}

impl<'a> RecordProxy<'a> {
    pub(crate) fn new(
        dispatch: &'a tracing::Dispatch,
        metadata: &'static Metadata<'static>,
        span_id: &'a span::Id,
    ) -> Self {
        Self {
            dispatch,
            metadata,
            span_id,
        }
    }
}

impl<'a> DispatchProxy for RecordProxy<'a> {
    type Output = ();

    fn dispatch<const N: usize>(
        &self,
        values: [(&field::Field, Option<&dyn tracing::Value>); N],
    ) -> Self::Output {
        let value_set = self.metadata.fields().value_set(&values);
        let record = span::Record::new(&value_set);
        self.dispatch.record(self.span_id, &record);
    }
}
