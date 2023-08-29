use std::fmt::Display;

use srs_4l::gameplay::Shape;

use hashbrown::{HashMap, HashSet};

pub type QueueSet = HashMap<QueueState, HashSet<ShapeHistory>>;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct ShapeHistory(pub u32);

impl ShapeHistory{
    pub fn new(shape:Shape) -> Self{
        Self(shape as u32 + 1)
    }
    pub fn add_shape(&self, shape:Shape) -> Self{
        Self((self.0 << 3) | (shape as u32 + 1))
    }
}

impl Display for ShapeHistory{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut n = self.0;
        if n == 0{
            f.write_str("empty queue")?;
            return Ok(())
        }
        f.write_str("queue:")?;
        let mut shapes = Vec::new();

        while let Some(shape) = Shape::try_from((n&7) as u8 - 1){ //weird hack, grab 3 bits as b111 we offset by 1
            shapes.push(shape);
            n = n >> 3;
            if n == 0{break}
        }
        for i in (0..shapes.len()).rev(){
            f.write_fmt(format_args!(" {:?}",shapes[i]))?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Bag {
    pub count: u8,
    pub full: u16,
    pub masks: [u16; 7],
}

impl Bag {
    pub fn new(shapes: &[Shape], count: u8) -> Bag {
        assert!(count as usize <= shapes.len());
        assert!(shapes.len() <= 13);

        let mut bag = Bag {
            count,
            full: (1 << shapes.len()) - 1,
            masks: [0; 7],
        };

        for (i, &shape) in shapes.iter().enumerate() {
            bag.masks[shape as usize] |= 1 << i;
        }

        bag
    }

    pub fn init_hold(&self) -> QueueSet {
        let initial = QueueState(self.full);

        Shape::ALL
            .iter()
            .filter_map(|&shape| {
                match initial.swap(self, shape){
                    Some(state) => {
                        let mut set = HashSet::new();
                        let history = ShapeHistory::new(shape);
                        set.insert(history);
                        Some((state, set))
                    },
                    None => None,
                }
            }).collect()

    }

    pub fn take(
        &self,
        queues: &QueueSet,
        shape: Shape,
        is_first: bool
    ) -> QueueSet {
        let mut states : QueueSet = HashMap::new();

        for (queue, histories) in queues {
            let queue = if is_first { queue.next(self) } else { *queue };

            if queue.hold() == Some(shape) {
                for swap_shape in Shape::ALL {

                    if let Some(new) = queue.swap(self, swap_shape) {
                        let mut new_histories = HashSet::new();
                        for history in histories{
                            new_histories.insert(history.add_shape(swap_shape));
                        }
                        let entry = states.entry(new).or_default();
                        entry.extend(new_histories);
                    }
                }
            }
            if let Some(new) = queue.take(self, shape) {
                let mut new_histories = HashSet::new();
                for history in histories{
                    new_histories.insert(history.add_shape(shape));
                }
                let entry = states.entry(new).or_default();
                entry.extend(new_histories);
            }
        }

        states
    }

}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct QueueState(pub u16);

impl QueueState {
    pub fn hold(self) -> Option<Shape> {
        match self.0 >> 13 {
            0 => Some(Shape::I),
            1 => Some(Shape::J),
            2 => Some(Shape::L),
            3 => Some(Shape::O),
            4 => Some(Shape::S),
            5 => Some(Shape::T),
            6 => Some(Shape::Z),
            _ => None,
        }
    }

    pub fn next(self, bag: &Bag) -> QueueState {
        QueueState(self.0 & 0b1110000000000000 | bag.full)
    }

    pub fn take(self, bag: &Bag, shape: Shape) -> Option<QueueState> {
        let shape_field = self.0 & bag.masks[shape as usize];

        if shape_field == 0 {
            return None;
        }

        let new_shape_field = shape_field & (shape_field - 1);
        Some(QueueState(self.0 ^ shape_field ^ new_shape_field))
    }

    pub fn swap(self, bag: &Bag, shape: Shape) -> Option<QueueState> {
        let mut new = self.take(bag, shape)?;
        new.0 &= 0b1111111111111;
        new.0 |= (shape as u16) << 13;
        Some(new)
    }
}
