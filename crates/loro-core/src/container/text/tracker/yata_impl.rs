use crdt_list::{
    crdt::{ListCrdt, OpSet},
    yata::Yata,
};
use rle::{
    range_map::{RangeMap, WithStartEnd},
    rle_tree::{iter::IterMut, SafeCursorMut},
};

use crate::id::{Counter, ID};

use super::{
    cursor_map::make_notify,
    y_span::{YSpan, YSpanTreeTrait},
    Tracker,
};

#[derive(Default, Debug)]
pub struct OpSpanSet {
    map: RangeMap<u128, WithStartEnd<u128, bool>>,
}

impl OpSet<YSpan, ID> for OpSpanSet {
    fn insert(&mut self, value: &YSpan) {
        self.map.set(
            value.id.into(),
            WithStartEnd {
                start: value.id.into(),
                end: value.id.inc(value.len as i32).into(),
                value: true,
            },
        )
    }

    fn contain(&self, id: ID) -> bool {
        self.map.has(id.into())
    }

    fn clear(&mut self) {
        self.map.clear();
    }
}

pub struct YataImpl;

impl ListCrdt for YataImpl {
    type OpUnit = YSpan;

    type OpId = ID;

    type Container = Tracker;

    type Set = OpSpanSet;

    type Cursor<'a> = SafeCursorMut<'a, YSpan, YSpanTreeTrait>;

    type Iterator<'a> = IterMut<'a, YSpan, YSpanTreeTrait>;

    fn iter(
        container: &mut Self::Container,
        from: Option<Self::OpId>,
        to: Option<Self::OpId>,
    ) -> Self::Iterator<'_> {
        let from = from
            .and_then(|x| {
                container
                    .id_to_cursor
                    .get(x.into())
                    .and_then(|m| m.as_cursor(x))
            })
            .and_then(|x| x.shift(1));
        let to = to.and_then(|x| {
            container
                .id_to_cursor
                .get(x.into())
                .and_then(|m| m.as_cursor(x))
        });

        // dbg!(&from, &to);
        // dbg!(&container.content);
        // SAFETY: loosen lifetime requirement here. It's safe because the function
        // signature can limit the lifetime of the returned iterator
        unsafe { std::mem::transmute(container.content.iter_mut_in(from, to)) }
    }

    fn id(op: &Self::OpUnit) -> Self::OpId {
        op.id
    }

    fn cmp_id(op_a: &Self::OpUnit, op_b: &Self::OpUnit) -> std::cmp::Ordering {
        op_a.id.client_id.cmp(&op_b.id.client_id)
    }

    fn contains(op: &Self::OpUnit, id: Self::OpId) -> bool {
        op.id.contains(op.len as Counter, id)
    }
}

impl Yata for YataImpl {
    fn left_origin(op: &Self::OpUnit) -> Option<Self::OpId> {
        op.origin_left
    }

    fn right_origin(op: &Self::OpUnit) -> Option<Self::OpId> {
        op.origin_right
    }

    fn insert_after(container: &mut Self::Container, anchor: Self::Cursor<'_>, op: Self::OpUnit) {
        let mut notify = make_notify(&mut container.id_to_cursor);
        anchor.insert_after_notify(op, &mut notify)
    }

    fn insert_after_id(container: &mut Self::Container, id: Option<Self::OpId>, op: Self::OpUnit) {
        if let Some(id) = id {
            let left = container.id_to_cursor.get(id.into()).unwrap();
            let left = left.as_cursor(id).unwrap();
            let mut notify = make_notify(&mut container.id_to_cursor);
            // SAFETY: we own the tree here
            unsafe {
                left.unwrap()
                    .shift(1)
                    .unwrap()
                    .insert_notify(op, &mut notify);
            }
        } else {
            let mut notify = make_notify(&mut container.id_to_cursor);
            container.content.insert_at_first(op, &mut notify);
        }
    }
}

#[cfg(test)]
mod test {
    use crdt_list::crdt::OpSet;

    use crate::{
        container::text::tracker::y_span::{Status, YSpan},
        id::ID,
    };

    use super::OpSpanSet;

    #[test]
    fn test() {
        let mut set = OpSpanSet::default();
        set.insert(&YSpan {
            id: ID::new(1, 10),
            len: 10,
            origin_left: Some(ID::new(0, 1)),
            origin_right: Some(ID::new(0, 2)),
            status: Status::new(),
        });
        assert!(set.contain(ID::new(1, 10)));
        assert!(set.contain(ID::new(1, 11)));
        assert!(set.contain(ID::new(1, 18)));
        assert!(set.contain(ID::new(1, 19)));

        assert!(!set.contain(ID::new(1, 8)));
        assert!(!set.contain(ID::new(1, 9)));
        assert!(!set.contain(ID::new(1, 20)));
        assert!(!set.contain(ID::new(1, 21)));
    }
}

#[cfg(feature = "fuzzing")]
pub mod fuzz {
    #![allow(unused_imports)]
    use crdt_list::{
        test::{Action, TestFramework},
        yata::Yata,
    };
    use rle::RleVec;

    use crate::{
        container::text::tracker::{y_span::StatusChange, Tracker},
        id::{ClientID, ID},
        span::{self, IdSpan},
    };

    use super::YataImpl;

    impl TestFramework for YataImpl {
        fn integrate(container: &mut Self::Container, op: Self::OpUnit) {
            container.head_vv.set_end(op.id.inc(op.len as i32));
            // SAFETY: we know this is safe because in [YataImpl::insert_after] there is no access to shared elements
            unsafe { crdt_list::yata::integrate::<Self>(container, op) };
        }

        #[inline]
        fn can_integrate(container: &Self::Container, op: &Self::OpUnit) -> bool {
            if let Some(value) = op.origin_left {
                if !value.is_unknown() && !container.head_vv.includes_id(value) {
                    return false;
                }
            }

            if let Some(value) = op.origin_right {
                if !value.is_unknown() && !container.head_vv.includes_id(value) {
                    return false;
                }
            }

            if op.id.counter != 0 && !container.head_vv.includes_id(op.id.inc(-1)) {
                return false;
            }

            true
        }

        fn is_content_eq(a: &Self::Container, b: &Self::Container) -> bool {
            let aa = {
                let mut ans = RleVec::new();
                for iter in a.content.iter() {
                    ans.push(iter.as_ref().clone());
                }
                ans
            };
            let bb = {
                let mut ans = RleVec::new();
                for iter in b.content.iter() {
                    ans.push(iter.as_ref().clone());
                }
                ans
            };

            if aa != bb {
                dbg!(a.client_id);
                dbg!(b.client_id);
                dbg!(aa.vec());
                dbg!(bb.vec());
                // dbg!(&a.content);
                // dbg!(&b.content);
            }

            assert_eq!(aa, bb);
            aa == bb
        }

        fn new_container(client_id: usize) -> Self::Container {
            let mut tracker = Tracker::new(Default::default());
            #[cfg(feature = "fuzzing")]
            {
                tracker.client_id = client_id as ClientID;
            }

            tracker
        }

        fn new_op(
            _: &mut impl rand::Rng,
            container: &mut Self::Container,
            pos: usize,
        ) -> Self::OpUnit {
            let ans = container.content.get_yspan_at_pos(
                ID::new(
                    container.client_id,
                    *container.head_vv.get(&container.client_id).unwrap_or(&0),
                ),
                pos % container.content.len(),
                pos % 10 + 1,
            );
            ans
        }

        type DeleteOp = RleVec<IdSpan>;

        fn new_del_op(
            container: &Self::Container,
            mut pos: usize,
            mut len: usize,
        ) -> Self::DeleteOp {
            if container.content.len() == 0 {
                return RleVec::new();
            }

            pos %= container.content.len();
            len = std::cmp::min(len % 10, container.content.len() - pos);
            if len == 0 {
                return RleVec::new();
            }

            container.content.get_id_spans(pos, len)
        }

        fn integrate_delete_op(container: &mut Self::Container, op: Self::DeleteOp) {
            container.update_spans(&op, StatusChange::Delete);
        }
    }

    use Action::*;
    #[test]
    fn issue_1() {
        crdt_list::test::test_with_actions::<YataImpl>(
            5,
            100,
            vec![
                NewOp {
                    client_id: 177,
                    pos: 41,
                },
                Delete {
                    client_id: 255,
                    pos: 255,
                    len: 255,
                },
                NewOp {
                    client_id: 162,
                    pos: 162,
                },
                Delete {
                    client_id: 255,
                    pos: 255,
                    len: 229,
                },
                Delete {
                    client_id: 150,
                    pos: 227,
                    len: 1,
                },
                NewOp {
                    client_id: 255,
                    pos: 255,
                },
                Delete {
                    client_id: 255,
                    pos: 255,
                    len: 0,
                },
                Sync { from: 0, to: 0 },
                Sync { from: 0, to: 48 },
                Sync { from: 0, to: 0 },
                NewOp {
                    client_id: 150,
                    pos: 150,
                },
                Delete {
                    client_id: 63,
                    pos: 229,
                    len: 229,
                },
                Delete {
                    client_id: 175,
                    pos: 175,
                    len: 255,
                },
                Delete {
                    client_id: 237,
                    pos: 237,
                    len: 237,
                },
                Delete {
                    client_id: 237,
                    pos: 237,
                    len: 237,
                },
                Delete {
                    client_id: 229,
                    pos: 229,
                    len: 229,
                },
                NewOp {
                    client_id: 58,
                    pos: 255,
                },
                Delete {
                    client_id: 255,
                    pos: 255,
                    len: 255,
                },
                Delete {
                    client_id: 175,
                    pos: 175,
                    len: 175,
                },
                Delete {
                    client_id: 175,
                    pos: 175,
                    len: 175,
                },
                Delete {
                    client_id: 175,
                    pos: 237,
                    len: 237,
                },
                Delete {
                    client_id: 237,
                    pos: 237,
                    len: 237,
                },
                Delete {
                    client_id: 237,
                    pos: 237,
                    len: 237,
                },
                Delete {
                    client_id: 237,
                    pos: 237,
                    len: 237,
                },
                Delete {
                    client_id: 255,
                    pos: 255,
                    len: 255,
                },
                Delete {
                    client_id: 255,
                    pos: 255,
                    len: 255,
                },
                Delete {
                    client_id: 255,
                    pos: 255,
                    len: 255,
                },
                Sync { from: 48, to: 175 },
                Delete {
                    client_id: 175,
                    pos: 175,
                    len: 175,
                },
                Delete {
                    client_id: 175,
                    pos: 175,
                    len: 175,
                },
                Delete {
                    client_id: 175,
                    pos: 237,
                    len: 237,
                },
                Delete {
                    client_id: 237,
                    pos: 237,
                    len: 237,
                },
                Delete {
                    client_id: 175,
                    pos: 175,
                    len: 229,
                },
                Delete {
                    client_id: 237,
                    pos: 237,
                    len: 237,
                },
                Delete {
                    client_id: 237,
                    pos: 237,
                    len: 229,
                },
                Delete {
                    client_id: 229,
                    pos: 114,
                    len: 223,
                },
                Delete {
                    client_id: 255,
                    pos: 229,
                    len: 255,
                },
                Delete {
                    client_id: 175,
                    pos: 175,
                    len: 175,
                },
                Delete {
                    client_id: 175,
                    pos: 175,
                    len: 175,
                },
                Delete {
                    client_id: 175,
                    pos: 175,
                    len: 175,
                },
                Delete {
                    client_id: 237,
                    pos: 237,
                    len: 237,
                },
                Delete {
                    client_id: 237,
                    pos: 237,
                    len: 237,
                },
                Delete {
                    client_id: 255,
                    pos: 255,
                    len: 255,
                },
                Delete {
                    client_id: 255,
                    pos: 255,
                    len: 255,
                },
                Delete {
                    client_id: 196,
                    pos: 150,
                    len: 150,
                },
                Sync { from: 162, to: 162 },
                NewOp {
                    client_id: 162,
                    pos: 162,
                },
                NewOp {
                    client_id: 162,
                    pos: 162,
                },
                NewOp {
                    client_id: 162,
                    pos: 162,
                },
                NewOp {
                    client_id: 162,
                    pos: 162,
                },
                Sync { from: 81, to: 162 },
                Sync { from: 0, to: 0 },
                Sync { from: 162, to: 162 },
                Sync { from: 0, to: 0 },
                Sync { from: 0, to: 0 },
                Sync { from: 0, to: 162 },
                NewOp {
                    client_id: 162,
                    pos: 162,
                },
                NewOp {
                    client_id: 162,
                    pos: 162,
                },
                NewOp {
                    client_id: 233,
                    pos: 162,
                },
                NewOp {
                    client_id: 150,
                    pos: 150,
                },
                Delete {
                    client_id: 255,
                    pos: 255,
                    len: 255,
                },
                Sync { from: 0, to: 0 },
                Delete {
                    client_id: 175,
                    pos: 255,
                    len: 255,
                },
                Delete {
                    client_id: 229,
                    pos: 229,
                    len: 196,
                },
                NewOp {
                    client_id: 162,
                    pos: 162,
                },
                NewOp {
                    client_id: 162,
                    pos: 162,
                },
                NewOp {
                    client_id: 162,
                    pos: 162,
                },
                NewOp {
                    client_id: 162,
                    pos: 1,
                },
                NewOp {
                    client_id: 162,
                    pos: 255,
                },
                Delete {
                    client_id: 229,
                    pos: 229,
                    len: 229,
                },
                Sync { from: 255, to: 255 },
                Sync { from: 255, to: 162 },
                NewOp {
                    client_id: 162,
                    pos: 162,
                },
            ],
        )
    }

    #[test]
    fn normalize() {
        let mut actions = vec![
            NewOp {
                client_id: 129,
                pos: 142,
            },
            NewOp {
                client_id: 0,
                pos: 85,
            },
            Sync { from: 85, to: 86 },
            NewOp {
                client_id: 129,
                pos: 129,
            },
            Sync { from: 129, to: 129 },
            NewOp {
                client_id: 106,
                pos: 106,
            },
            NewOp {
                client_id: 1,
                pos: 0,
            },
            NewOp {
                client_id: 129,
                pos: 106,
            },
        ];

        crdt_list::test::normalize_actions(&mut actions, 5, 100);
        dbg!(actions);
    }
}