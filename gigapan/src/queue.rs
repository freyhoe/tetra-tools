use std::{
    fmt::{Display, Write},
    str::FromStr
};

use hashbrown::HashMap;
use srs_4l::gameplay::Shape;
use std::collections::VecDeque;


struct BagInput{
    shapes: Vec<Shape>,
    count: u8,
    shape_set: HashMap<Shape, usize>,
    ordered: bool
}

fn shape_set_from_list(list: &[Shape])->HashMap<Shape, usize>{
    let mut shape_set = HashMap::with_capacity(7);
    for &shape in list{
        match shape_set.entry(shape){
            hashbrown::hash_map::Entry::Occupied(mut entry) => {
                *entry.get_mut()+=1;
            },
            hashbrown::hash_map::Entry::Vacant(entry) => {
                entry.insert(1);
            },
        }
    }
    shape_set
}


pub struct CombinatoricQueue {
    bags: Vec<BagInput>,
}
impl CombinatoricQueue {
    pub fn new() -> Self {
        Self {
            bags: Vec::new(),
        }
    }
    pub fn queue_count(&self)->usize{
        let mut count = 1;
        for bag_input in &self.bags{
            if bag_input.ordered{continue;}
            let numerator : usize = ((bag_input.shapes.len()-(bag_input.count as usize)+1)..=bag_input.shapes.len()).product();
            let denominator : usize = bag_input.shape_set.values().map(|count|{(1..=*count).product::<usize>()}).product();
            assert!(numerator%denominator==0);
            count *= numerator/denominator
        }
        count
    }
    pub fn get_counted_bags(&self)-> Vec<(u8, Bag)>{
        self.bags.iter().map(|input|{//cursed boxed iterators in order to have Once and Map in parralel 
            let iter: Box<dyn Iterator<Item= (u8, Bag)>> = if input.ordered{
                Box::new(input.shapes.iter().map(|&shape|{
                    (0, Bag::new(&[shape], 1))
                }))
            }else{
                Box::new(
                    std::iter::repeat(Bag::new(&input.shapes, input.count))
                    .take(input.count as usize)
                    .enumerate()
                    .map(|(i, bag)|{
                        (i as u8, bag)
                    })
                )
            };
            iter
        }).flatten().collect()
    }
    pub fn add_shapes(&mut self, shapes: Vec<Shape>) {
        let count = shapes.len() as u8;
        let shape_set = shape_set_from_list(&shapes);
        self.bags.push(BagInput { shapes, count, ordered: true, shape_set})
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

        let shape_set = shape_set_from_list(&new_shapes);

        assert!(count as usize <= shape_len && count > 0, "asserting that count: {} is valid for bag: {:?}", count, new_shapes);
        self.bags.push(BagInput{shapes: new_shapes, count, ordered: false, shape_set});
    }
}


pub fn get_queue_permutations(counted_bags: &[(u8, Bag)], start_state: Option<(usize, QueueState)>, max_depth: Option<usize>)-> Vec<VecDeque<Shape>>{
    let mut queue = VecDeque::new();
    let mut permutations = Vec::new();

    let max_depth = max_depth.unwrap_or(counted_bags.len());
    if let Some((start_depth, queue_state)) = start_state{
        recursive_permute_bags(counted_bags, &mut permutations, start_depth, max_depth, queue_state, &mut queue);
    }else{
        let queue_state = QueueState(counted_bags.first().unwrap().1.full);
        recursive_permute_bags(counted_bags, &mut permutations, 0, max_depth, queue_state, &mut queue);
    }
    permutations
}

fn recursive_permute_bags(bags: &[(u8, Bag)], permutations: &mut Vec<VecDeque<Shape>>, depth: usize, max_depth:usize, state: QueueState, queue: &mut VecDeque<Shape>){
    if depth >= max_depth{
        permutations.push(queue.clone());
        return
    }
    let (bag_placement, bag) = &bags[depth];
    let state = if bag_placement == &0{state.next(&bag)}else{state};
    for shape in Shape::ALL{
        if let Some(state) = state.take(&bag, shape){
            queue.push_back(shape);
            recursive_permute_bags(bags, permutations, depth+1, max_depth, state, queue);
            queue.pop_back();
        }
    }
}

#[test]
fn queuecombo(){
    let input_str = "[IJSZ]!IJ*p3";
    let queue = CombinatoricQueue::from_str(input_str).unwrap();

    let permus = get_queue_permutations(&queue.get_counted_bags(), None, None);
    assert_eq!(permus.len(), queue.queue_count());
    assert_eq!(permus.len(), 5040);

    let input_str = "IZJSSJLIOZT";
    let queue = CombinatoricQueue::from_str(input_str).unwrap();
    assert_eq!(queue.queue_count(), 1);
    assert_eq!(queue.get_counted_bags().len(), 11);

    let input_str = "[SSZZ]!";
    let queue = CombinatoricQueue::from_str(input_str).unwrap();

    let permus = get_queue_permutations(&queue.get_counted_bags(), None, None);
    assert_eq!(permus.len(), queue.queue_count());
    assert_eq!(permus.len(), 6);

}

impl Display for CombinatoricQueue{
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
    let queue = CombinatoricQueue::from_str(input_str).unwrap();
    assert_eq!(queue.to_string(), "*p3,*,*p7,*,[IOSZ]p2,JLT,[SZ]!");
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


}


#[derive(Debug, PartialEq, Eq)]
pub struct InvalidTokenError;

impl FromStr for CombinatoricQueue {
    type Err = InvalidTokenError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut queue = CombinatoricQueue::new();
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
