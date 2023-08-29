use srs_4l::gameplay::{Shape, Board};
use crate::boardgraph::FrozenGigapan;
use crate::queue::{Bag, ShapeHistory, QueueSet};
use hashbrown::{HashMap, HashSet};
use rayon::prelude::*;

type ScanStage = HashMap<Board, QueueSet>;

pub fn chance(gigapan: &FrozenGigapan, board: Board){
    scan(gigapan, board, &[Bag::new(&Shape::ALL, 7), Bag::new(&Shape::ALL, 4)]);

}

fn scan(
    gigapan: &FrozenGigapan,
    start: Board,
    bags: &[Bag],
){

    let mut prev: ScanStage = HashMap::new();
    let first_queues = bags.first().unwrap().init_hold();

    prev.insert(start, first_queues);

    for (_stage, (bag, i)) in bags
        .iter()//yo i forgot how this works, sorry, gg go next, <3 hmu at
        .flat_map(|b| (0..b.count).into_iter().map(move |i| (b, i)))
        .skip(1)
        .enumerate()
    {
        let mut next: ScanStage = HashMap::new();

        for (&old_board, old_queues) in prev.iter() {

            for (shape, new_boards) in gigapan.get(&old_board).unwrap().into_iter().enumerate(){
                let new_queues = bag.take(old_queues, Shape::try_from(shape as u8).unwrap(), i == 0);

                if new_queues.is_empty() {
                    continue;
                }

                for &new_board in new_boards{
                    
                    let next_queues = next.entry(new_board).or_default();
                    for (&state, queues) in &new_queues {
                        next_queues.entry(state).or_default();//.extend(queues);
                    }
                }
            }
        }
        prev = next;
    }
    for (board, map) in prev{
        println!("{board}");

        let mut histories : HashSet<ShapeHistory>= HashSet::new();
        histories.extend(map.iter().flat_map(|(_,x)|x));
        println!("min count {}", histories.len() );
    }

}