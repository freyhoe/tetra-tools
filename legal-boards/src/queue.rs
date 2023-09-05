use std::{
    fmt::Display,
    str::FromStr,
};

use hashbrown::{HashMap, HashSet};
use smallvec::SmallVec;

use srs_4l::{gameplay::Shape, queue::Queue};

pub struct QueueGenerator {
    pub bags: Vec<Bag>,
    pub string: String,
    pub total_queues: usize
}
impl QueueGenerator {
    pub fn new() -> Self {
        Self {
            bags: Vec::new(),
            string: "".to_owned(),
            total_queues: 1
        }
    }
    pub fn add_shapes(&mut self, shapes: Vec<Shape>) {
        for shape in shapes {
            self.string.push_str(&format!("{:?}", shape));
            self.bags.push(Bag::new(&[shape], 1));
        }
        self.string.push(',');
    }
    pub fn add_bag(&mut self, shapes: Vec<Shape>, count: Option<u8>, inverted: bool) {

        let new_shapes = if inverted{ 
            let mut new_shapes = Vec::new();
            for shape in Shape::ALL {
                if !shapes.contains(&shape) {
                    new_shapes.push(shape);
                }
            }
            new_shapes
        }else{
            shapes
        };
        let shape_len = new_shapes.len();
        let count = count.unwrap_or(shape_len as u8);
        self.total_queues *= ((shape_len+1-count as usize)..=shape_len).product::<usize>();
        self.bags.push(Bag::new(&new_shapes, count));

        let mut star = false;
        if new_shapes == &Shape::ALL {
            self.string.push('*');
            star = true;
        } else {
            self.string.push('[');
            for shape in &new_shapes {
                self.string.push_str(&format!("{:?}", shape));
            }
            self.string.push(']');
        }

        if count == new_shapes.len() as u8 && !star {
            self.string.push('!');
        } else if count != 1{
            self.string.push_str(&format!("p{count}"));
        }
        self.string.push(',');
    }
}
impl Display for QueueGenerator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self.string.trim_end_matches(',');
        f.write_str(s)?;
        Ok(())
    }
}

#[test]
fn queuegen_string(){
    let input_str = "[^]p3**p1[^JLT]p2JLT[SZ]!";
    let queue = QueueGenerator::from_str(input_str).unwrap();
    println!("input_str: {input_str}, cleaned: {}", queue)
}

#[derive(Debug, PartialEq, Eq)]
pub struct InvalidTokenError;

impl FromStr for QueueGenerator {
    type Err = InvalidTokenError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut queue = QueueGenerator::new();
        let mut open_bracket = false;
        let mut current_bag: Vec<Shape> = Vec::new();
        let s = s.replace("*", "[IJLOSTZ]");
        let mut char_iter = s.chars().peekable();
        let mut inverted = false;
        while let Some(c) = char_iter.next() {
            match c {
                'I' | 'O' | 'T' | 'L' | 'J' | 'S' | 'Z' => {
                    let shape = match c {
                        'I' => Shape::I,
                        'O' => Shape::O,
                        'T' => Shape::T,
                        'L' => Shape::L,
                        'J' => Shape::J,
                        'S' => Shape::S,
                        'Z' => Shape::Z,
                        _ => unreachable!(),
                    };
                    current_bag.push(shape);
                    if !open_bracket{
                        if let Some(c1) = char_iter.peek() {
                            match c1 {
                                'I' | 'O' | 'T' | 'L' | 'J' | 'S' | 'Z' => {}
                                _ => {
                                    queue.add_shapes(current_bag);
                                    current_bag = Vec::new();
                                }
                            }
                        }else{
                            queue.add_shapes(current_bag);
                            current_bag = Vec::new();

                        }
                    }
                }
                '^' => {
                    if !open_bracket {
                        return Err(InvalidTokenError);
                    }
                    inverted = true;
                }
                '[' => {
                    if open_bracket {
                        return Err(InvalidTokenError);
                    }
                    open_bracket = true;
                }
                ']' => {
                    if !open_bracket {
                        return Err(InvalidTokenError);
                    }
                    open_bracket = false;
                    if let Some(c1) = char_iter.peek() {
                        if c1 == &'p' {
                            char_iter.next();
                            let mut numerical_chars = Vec::new();
                            while let Some(c1) = char_iter.peek() {
                                if c1.is_numeric() {
                                    let c1 = char_iter.next().unwrap();
                                    numerical_chars.push(c1);
                                } else {
                                    break;
                                }
                            }
                            let number: String = numerical_chars.iter().collect();
                            match number.parse::<u8>() {
                                Ok(num) => {
                                    queue.add_bag(current_bag, Some(num), inverted);
                                }
                                Err(_) => return Err(InvalidTokenError),
                            }
                        } else if c1==&'!'{
                            queue.add_bag(current_bag, None, inverted);
                        }else {
                            queue.add_bag(current_bag, Some(1), inverted);
                        }

                    }else{
                        queue.add_bag(current_bag, Some(1), inverted);
                    }
                    current_bag = Vec::new();
                    inverted = false;
                }
                _ => {}
            }
        }
        Ok(queue)
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

    pub fn init_hold(&self) -> SmallVec<[QueueState; 7]> {
        let initial = QueueState(self.full);

        Shape::ALL
            .iter()
            .filter_map(|&shape| initial.swap(self, shape))
            .collect()
    }
    pub fn take(
        &self,
        queues: &[QueueState],
        shape: Shape,
        is_first: bool,
        can_hold: bool,
    ) -> SmallVec<[QueueState; 7]> {
        let mut states = SmallVec::new();

        for &queue in queues {
            let queue = if is_first { queue.next(self) } else { queue };

            if queue.hold() == Some(shape) {
                for swap_shape in Shape::ALL {
                    if let Some(new) = queue.swap(self, swap_shape) {
                        if !states.contains(&new) {
                            states.push(new);
                        }
                    }
                }
            } else if can_hold {
                if let Some(new) = queue.take(self, shape) {
                    if !states.contains(&new) {
                        states.push(new);
                    }
                }
            }
        }

        states
    }

    pub fn init_hold_with_history(&self) -> HashMap<QueueState, HashSet<Queue>> {
        let initial = QueueState(self.full);

        Shape::ALL
            .iter()
            .filter_map(|&shape| initial.swap(self, shape))
            .map(|s|{
                let mut set = HashSet::new();
                let q = Queue::empty();
                set.insert(q.push_first(s.hold().unwrap()));
                (s, set)
            })
            .collect()
    }
    pub fn take_with_history(
        &self,
        queues: &HashMap<QueueState, HashSet<Queue>>,
        shape: Shape,
        is_first: bool,
        can_hold: bool,
    ) -> HashMap<QueueState, HashSet<Queue>> {
        let mut states = HashMap::new();
        for (&queue, histories) in queues {
            let queue = if is_first {queue.next(self) } else { queue };

            if queue.hold() == Some(shape) {
                for swap_shape in Shape::ALL {
                    if let Some(new) = queue.swap(self, swap_shape) {
                        let mut new_histories = HashSet::new();
                        for history in histories{
                            new_histories.insert(history.push_first(swap_shape));
                        }
                        let entry: &mut HashSet<Queue> = states.entry(new).or_default();
                        entry.extend(new_histories);
                    }
                }
            } else if can_hold {
                if let Some(new) = queue.take(self, shape) {
                    let mut new_histories = HashSet::new();
                    for history in histories{
                        new_histories.insert(history.push_first(shape));
                    }
                    let entry = states.entry(new).or_default();
                    entry.extend(new_histories);
                }
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