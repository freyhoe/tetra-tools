use std::{borrow::Borrow, collections::BTreeSet, iter::FromIterator, fmt::Display};

use srs_4l::gameplay::Shape;

/// A sequence of up to 10 pieces.  The integer inside can be used to refer to
/// this queue by number.  However, it should mostly be treated as opaque data.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Queue(pub u64);

impl Queue {
    /// An empty queue.
    pub fn empty() -> Queue {
        Queue(0)
    }

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Push a shape onto the front of this queue.  The given shape will now be
    /// first.
    #[must_use]
    pub fn push_first(self, shape: Shape) -> Queue {
        let new = (shape as u64) + 1;
        let rest = self.0 << 3;
        Queue(new | rest)
    }

    /// Push a shape as the second into this queue.  The given shape will now be
    /// second.
    #[must_use]
    pub fn push_second(self, shape: Shape) -> Queue {
        assert!(!self.is_empty()); // otherwise this method doesn't make sense

        let first = self.0 & 0b111;
        let new = ((shape as u64) + 1) << 3;
        let rest = (self.0 & !0b111) << 3;
        Queue(first | new | rest)
    }

    /// Push a shape onto the end of this queue.  The given shape will now be
    /// last.
    #[must_use]
    pub fn push_last(self, shape: Shape) -> Queue {
        let next_slot = self.len() * 3;
        let new = ((shape as u64) + 1) << next_slot;

        Queue(self.0 | new)
    }

    pub fn len(self) -> u32 {
        let highest_one = 32 - self.0.leading_zeros();
        (highest_one + 2) / 3
    }

    /// Produce a [`String`] containing the names of the shapes in this queue.
    pub fn to_string(self) -> String {
        let mut s = String::with_capacity(10);
        s.extend(self.map(Shape::name));
        s
    }

    /// Compute all queues which can be transformed into this queue using hold.
    ///
    /// This method assumes that the shapes in the provided queue are intended
    /// to be used exactly in order, without holding.  The returned queues are
    /// all queues which can be used *as though they were the provided queue* by
    /// using holding.

    pub fn unhold(self) -> BTreeSet<Queue> {
        let mut last = BTreeSet::new();

        let mut me: Vec<Shape> = self.collect();

        if let Some(shape) = me.pop() {
            last.insert(Queue::empty().push_first(shape));
        } else {
            last.insert(Queue::empty());
        }

        for &shape in me.iter().rev() {
            let mut next = BTreeSet::new();

            for queue in last {
                next.insert(queue.push_first(shape));
                next.insert(queue.push_second(shape));
            }

            last = next;
        }

        last
    }

    pub fn unhold_many(queues: &[Queue]) -> Vec<Queue> {
        let mut results: Vec<BTreeSet<Entry>> = Vec::new();
        results.resize_with(11, || BTreeSet::new());

        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        struct Entry {
            make: Queue,
            take: Queue,
        }

        for &queue in queues {
            results[queue.len() as usize].insert(Entry {
                make: Queue::empty(),
                take: queue.reverse(),
            });
        }

        for i in (1..=10).into_iter().rev() {
            let (next, this) = results.split_at_mut(i);
            let next = next.last_mut().unwrap();
            let this = this.first().unwrap();

            for entry in this {
                let mut take = entry.take;
                let shape = take.next().unwrap();

                next.insert(Entry {
                    make: entry.make.push_first(shape),
                    take,
                });

                if !entry.make.is_empty() {
                    next.insert(Entry {
                        make: entry.make.push_second(shape),
                        take,
                    });
                }
            }
        }

        let mut results: Vec<Queue> = results[0].iter().map(|e| e.make).collect();
        results.sort_unstable_by_key(|q| q.natural_order_key());
        results
    }

    pub fn natural_order_key(self) -> u64 {
        #![allow(non_snake_case)]

        let jihgfedcba = self.0;
        let hgfedcba = jihgfedcba & 0o77777777;

        let dcba____ = hgfedcba << 12 & 0o77770000;
        let ____hgfe = hgfedcba >> 12;
        let dcbahgfe = dcba____ | ____hgfe;

        let ba__fe__ = dcbahgfe << 6 & 0o77007700;
        let __dc__hg = dcbahgfe >> 6 & 0o00770077;
        let badcfehg = ba__fe__ | __dc__hg;

        let badcfehgji = badcfehg << 6 | jihgfedcba >> 24;

        let a_c_e_g_i_ = badcfehgji << 3 & 0o7070707070;
        let _b_d_f_h_j = badcfehgji >> 3 & 0o0707070707;
        let abcdefghij = a_c_e_g_i_ | _b_d_f_h_j;

        abcdefghij
    }

    #[must_use]
    pub fn reverse(self) -> Queue {
        let x = self.natural_order_key();
        Queue(x >> (x.trailing_zeros() / 3 * 3))
    }
}

impl Display for Queue{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[")?;
        for shape in self.into_iter(){
            f.write_fmt(format_args!("{:?}",shape))?;
        }
        f.write_str("]")?;
        Ok(())
    }
}

impl Iterator for Queue {
    type Item = Shape;

    fn next(&mut self) -> Option<Shape> {
        let first = match self.0 & 0b111 {
            0 => None,
            1 => Some(Shape::I),
            2 => Some(Shape::J),
            3 => Some(Shape::L),
            4 => Some(Shape::O),
            5 => Some(Shape::S),
            6 => Some(Shape::T),
            7 => Some(Shape::Z),
            _ => unreachable!(),
        };

        self.0 = self.0 >> 3;

        first
    }
}

impl<S: Borrow<Shape>> Extend<S> for Queue {
    fn extend<T: IntoIterator<Item = S>>(&mut self, iter: T) {
        for shape in iter {
            if self.len() == 10 {
                break;
            }

            *self = self.push_last(*shape.borrow());
        }
    }
}

impl<S: Borrow<Shape>> FromIterator<S> for Queue {
    fn from_iter<T: IntoIterator<Item = S>>(iter: T) -> Queue {
        let mut queue = Queue::empty();
        queue.extend(iter);
        queue
    }
}
