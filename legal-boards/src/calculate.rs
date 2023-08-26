use srs_4l::gameplay::{Shape, Board};
use smallvec::SmallVec;
use crate::boardgraph::FrozenGigapan;
struct Node{
    bag: Bag,
    states: Vec<(Board, Option<Shape>)>
}

impl Node{
    fn take<'a>(&'a self, gigapan: &'a FrozenGigapan) -> impl Iterator<Item=Self> + '_{
        self.bag.take().filter_map(move |(shape, bag)|{
            let states : Vec<_>= self.states.iter().flat_map(|(board, _)|{
                gigapan.get(board).unwrap()[shape as usize].iter().map(|&b|(b, None))
            }).collect();
            if states.len() == 0{
                None
            }else{
                Some(Self{
                    bag,
                    states
                })
            }
        })
    }
}

struct Bag{
    take: u8,
    shapes: SmallVec<[Shape;7]>
}

impl Bag{
    fn new() -> Self{
        Self { take: 7, shapes:SmallVec::from(Shape::ALL)}
    }
    fn take(&self) -> impl Iterator<Item=(Shape, Self)> + '_{
        (0..self.shapes.len()).map(move |idx|{
            let mut new_shapes = self.shapes.clone();
            let taken = new_shapes.remove(idx);
            let s = Self{
                take: self.take-1,
                shapes:new_shapes
            };
            (taken, s)
        })
    }
}

pub fn chance(gigapan: FrozenGigapan, board: Board){
    let to_place = 10-board.0.count_ones()/4;
    let mut beam = vec![Node{
        bag: Bag::new(),
        states: vec![(board, None)]
    }];
    for depth in 0..to_place{
        let mut new_beam = Vec::new();
        for node in &beam{
            let mut taken = false;
            for child in node.take(&gigapan){
                new_beam.push(child);
                taken = true;
            }
            if !taken {
                println!("FAIL AT DEPTH {depth}")
            }
        }
        beam = new_beam;
        println!("beam len: {}",beam.len());
    }

}