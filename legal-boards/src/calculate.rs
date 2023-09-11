use std::io::{Write, BufWriter};

use rayon::prelude::ParallelIterator;
use rayon::slice::ParallelSliceMut;
use srs_4l::gameplay::{Shape, Board};
use crate::boardgraph::FrozenGigapan;
use crate::queue::{Bag, QueueState, QueueMap};
use hashbrown::{HashMap, HashSet};
use compute::ShardedHashMap;
use smallvec::SmallVec;


type ScanStage = HashMap<Board, (SmallVec<[QueueState; 7]>, SmallVec< [Board; 7]>)>;

use std::time::Instant;

pub fn chance(gigapan: FrozenGigapan, board: Board, bags: &[Bag], total_queues: usize){
    let instant = Instant::now();

    let piece_count: usize = bags.iter().map(|b| b.count as usize).sum();
    let new_mino_count = piece_count as u32 * 4;
    if board.0.count_ones() + new_mino_count - 4 != 40{
        println!("bad queue len");
        return
    }

    let path = build_path(&gigapan, board, bags);
    println!("path calculated");
    
    let mut culled = HashSet::new();
    let mut iter = path.iter().rev();

    if let Some(final_stage) = iter.next() {
        for (&board, (_queues, preds)) in final_stage.iter() {
            culled.insert(board);
            culled.extend(preds);
        }
    }

    for stage in iter {
        for (&board, (_queues, preds)) in stage.iter() {
            if culled.contains(&board) {
                culled.extend(preds);
            }
        }
    }
    println!("culled boards grabbed {}", culled.len());
    print!("chance start: ");

    let mut prev: ShardedHashMap<Board, QueueMap, 20, nohash::BuildNoHashHasher<u64>> = ShardedHashMap::new();
    let first_queues = bags.first().unwrap().init_hold_with_history();

    prev.insert(board, first_queues);
    for (_stage, (bag, i)) in bags
        .iter()
        .flat_map(|b| (0..b.count).into_iter().map(move |i| (b, i)))
        .skip(1)
        .enumerate()
    {
        let mut next = ShardedHashMap::new();

        prev.into_par_iter().for_each(|(old_board, old_queues)|{

            for (shape, new_boards) in gigapan.get(&old_board).unwrap().into_iter().enumerate(){
                let shape = Shape::try_from(shape as u8).unwrap();

                let new_queues = bag.take_with_history(&old_queues, shape, i==0, true);
                
                if new_queues.is_empty() {
                    continue;
                }

                for &new_board in new_boards{
                    if !culled.contains(&new_board){continue;}
                    let mut lock = next.get_shard_guard(&new_board);

                    let next_queues: &mut QueueMap = lock.entry(new_board).or_default();
                    for (&state, queues) in &new_queues {
                        next_queues.entry(state).or_default().extend(queues);
                    }

                }
            }
        });
        print!("b: {}", next.len());
        std::io::stdout().flush().unwrap();

        prev = next;
    }
    println!();
    let queues = prev.into_iter().collect::<Vec<_>>();
    assert!(queues.len()==1);
    let mut map = HashSet::with_capacity(total_queues);
    for (_b, q) in queues{
        for q in q.into_values(){
            map.extend(q);
        }
    }
    println!("chance queues: {}/{total_queues}", map.len());
    println!("{}%", map.len() as f32/total_queues as f32 * 100.0);

    println!("computed in: {:?}", instant.elapsed());

    let mut queues : Vec<_>= map.into_iter().collect();
    queues.par_sort_unstable_by_key(|q| q.natural_order_key());

    let file = std::fs::File::create("passQueues.txt").unwrap();
    let mut buf_writer = BufWriter::new(file);

    for q in queues{
        buf_writer.write_fmt(format_args!("{q}\n")).unwrap();
    }
    buf_writer.flush().unwrap();


}



fn build_path(
    gigapan: &FrozenGigapan,
    start: Board,
    bags: &[Bag],
)-> Vec<ScanStage>{

    let mut stages = Vec::new();
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
                let shape = Shape::try_from(shape as u8).unwrap();
                let new_queues = bag.take(old_queues, shape, i == 0, true);

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
        stages.push(prev);
        prev = next;
    }
    assert!(prev.len()==1);
    stages.push(prev);
    stages
}