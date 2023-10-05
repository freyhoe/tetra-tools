use std::{
    fmt::{Display, Write},
    str::FromStr,
};

use hashbrown::HashSet;

use srs_4l::gameplay::Shape;
use srs_4l::queue::Queue;
use nohash::IntMap;
pub type QueueMap = IntMap<QueueState, HashSet<Queue>>;


struct BagInput{
    shapes: Vec<Shape>,
    count: u8,
    ordered: bool
}
pub struct QueueGenerator {
    bags: Vec<BagInput>,
    total_queues: usize,
}
impl QueueGenerator {
    pub fn new() -> Self {
        Self {
            bags: Vec::new(),
            total_queues: 1
        }
    }
    pub fn piece_count(&self)->usize{
        self.bags.iter().map(|input|input.count as usize).sum()
    }
    pub fn queue_count(&self)->usize{
        self.total_queues
    }
    pub fn get_bags(&self) -> Vec<Bag>{
        self.bags.iter().map(|input|Bag::new(&input.shapes, input.count)).collect()
    }
    pub fn add_shapes(&mut self, shapes: Vec<Shape>) {
        self.bags.push(BagInput { shapes, count: 1, ordered: true })
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
        self.bags.push(BagInput{shapes: new_shapes, count, ordered: false});
    }
}
impl Display for QueueGenerator{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut iter = self.bags.iter().peekable();
        while let Some(inputs) = iter.next(){
            let star = inputs.shapes == &Shape::ALL;
            if star{
                f.write_char('*')?;
            }else{
                if !inputs.ordered{
                    f.write_char('[')?;
                }
                for shape in inputs.shapes.iter(){
                    f.write_fmt(format_args!("{:?}",shape))?;
                }
                if !inputs.ordered{
                    f.write_char(']')?;
                }
            }
            if !inputs.ordered{
                if inputs.count == inputs.shapes.len() as u8 && !star{
                    f.write_char('!')?;
                }else if inputs.count>1{
                    f.write_fmt(format_args!("p{}", inputs.count))?;
                }
            }
            if let Some(_)= iter.peek(){
                f.write_char(',')?;

            }
        }
        Ok(())
    }
}

#[test]
fn queuegen_string(){
    let input_str = "[^]p3**p7*p1[^JLT]p2JLT[SZ]!";
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
                                    if num==0{
                                        return Err(InvalidTokenError)
                                    }
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

    pub fn init_hold(&self) -> Vec<QueueState> {
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
    ) -> Vec<QueueState> {
        let mut states = Vec::new();

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
    pub fn take_with_push(
        &self,
        queues: &[QueueState],
        shape: Shape,
        is_first: bool,
        can_hold: bool,
    ) -> (Vec<QueueState>, Vec<Shape>) {
        let mut states = Vec::new();
        let mut pieces = Vec::with_capacity(7);


        for &queue in queues {
            let queue = if is_first { queue.next(self) } else { queue };

            if queue.hold() == Some(shape) {
                for swap_shape in Shape::ALL {
                    if let Some(new) = queue.swap(self, swap_shape) {
                        if !states.contains(&new) {
                            states.push(new);
                        }
                        if !pieces.contains(&swap_shape){
                            pieces.push(swap_shape);
                        }
                    }
                }
            } else if can_hold {
                if let Some(new) = queue.take(self, shape) {
                    if !states.contains(&new) {
                        states.push(new);
                    }
                    if !pieces.contains(&shape){
                        pieces.push(shape);
                    }
                }
            } else{
                println!("fail")
            }
        }

        (states, pieces)
    }

    pub fn init_hold_with_history(&self) -> QueueMap {
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
        queues: &QueueMap,
        shape: Shape,
        is_first: bool,
        can_hold: bool,
    ) -> QueueMap {
        let mut states = IntMap::with_capacity_and_hasher(7, nohash::BuildNoHashHasher::default());
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



#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct QueueState(pub u16);

impl std::hash::Hash for QueueState {
    fn hash<H: std::hash::Hasher>(&self, hasher: &mut H) {
        hasher.write_u16(self.0)
    }
}

impl nohash::IsEnabled for QueueState {}

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

    pub fn force_swap(self, shape: Shape)-> Self{
        let mut new = self.0;
        new &= 0b1111111111111;
        new |= (shape as u16) << 13;
        Self(new)
    } 
}
