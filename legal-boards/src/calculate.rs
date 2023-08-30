use srs_4l::gameplay::{Shape, Board};
use crate::boardgraph::FrozenGigapan;
use crate::queue::{Bag, QueueState};
use hashbrown::{HashMap, HashSet};
use smallvec::SmallVec;

type ScanStage = HashMap<Board, (SmallVec<[QueueState; 7]>, SmallVec<[Board; 7]>)>;

pub fn chance(gigapan: &FrozenGigapan, board: Board, bags: &[Bag]){

    scan(gigapan, board, &bags);

}

fn scan(
    gigapan: &FrozenGigapan,
    start: Board,
    bags: &[Bag],
){

    let mut prev: ScanStage = HashMap::new();
    let first_queues = bags.first().unwrap().init_hold();

    prev.insert(start, (first_queues, SmallVec::new()));

    for (_stage, (bag, i)) in bags
        .iter()
        .flat_map(|b| (0..b.count).into_iter().map(move |i| (b, i)))
        .skip(1)
        .enumerate()
    {
        let mut next: ScanStage = HashMap::new();

        for (&old_board, (old_queues, _)) in prev.iter() {

            for (shape, new_boards) in gigapan.get(&old_board).unwrap().into_iter().enumerate(){
                let new_queues = bag.take(old_queues, Shape::try_from(shape as u8).unwrap(), i == 0, true);

                if new_queues.is_empty() {
                    continue;
                }

                for &new_board in new_boards{
                    
                    let (queues, preds) = next.entry(new_board).or_default();
                    for &queue in &new_queues {
                        if !queues.contains(&queue) {
                            queues.push(queue);
                        }
                    }
                    if !preds.contains(&old_board) {
                        preds.push(old_board);
                    }
                }
            }
        }
        prev = next;
    }

    let boards :  Vec<_>= prev.into_iter().collect();
    assert!(boards.len() ==1);
    print!("SAVE:");
    let mut saves : HashSet<Option<Shape>> = HashSet::new();
    saves.extend(boards[0].1.0.iter().map(|q| q.hold()));

    for save in saves{
        if let Some(save) = save{
            print!(" {:?},",save);
        }
    }
    println!();

}