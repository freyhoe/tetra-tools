use crate::boardgraph::FrozenGigapan;
use crate::queue::{Bag, QueueMap, QueueState};
use compute::ShardedHashMap;
use hashbrown::{HashMap, HashSet};
use rayon::prelude::ParallelIterator;
use srs_4l::gameplay::{Board, Shape};
use srs_4l::queue::Queue;

type ScanStage = HashMap<Board, (Vec<QueueState>, Vec<Board>)>;

use std::time::Instant;

pub fn chance(gigapan: &FrozenGigapan, board: Board, bags: &[Bag], total_queues: usize) {
    let piece_count: usize = bags.iter().map(|b| b.count as usize).sum();
    let new_mino_count = piece_count as u32 * 4;
    if board.0.count_ones() + new_mino_count - 4 != 40 {
        println!("bad queue len");
        return;
    }

    let culled = build_path(&gigapan, board, bags);
    println!("culled generated");
    let combos = get_blind_chance(
        gigapan,
        &culled,
        bags,
        board,
        &[
            Shape::I,
            Shape::Z,
            Shape::J,
            Shape::S,
            Shape::S,
            Shape::J,
            Shape::L,
        ],
    );
    println!("yaya tea!! {combos}");
}

fn get_blind_chance(
    gigapan: &FrozenGigapan,
    culled: &HashSet<Board>,
    bags: &[Bag],
    start_board: Board,
    start_queue: &[Shape],
) -> usize {
    let mut prev: ShardedHashMap<Board, HashSet<Shape>, 20, nohash::BuildNoHashHasher<u64>> =
        ShardedHashMap::new();
    let first_queues = {
        let mut set = HashSet::with_capacity(1);
        set.insert(*start_queue.first().unwrap());
        set
    };

    prev.insert(start_board, first_queues);
    for shape in start_queue.iter().skip(1) {
        let next = ShardedHashMap::new();

        prev.into_par_iter().for_each(|(old_board, old_queues)| {
            let edges = gigapan.get(&old_board).unwrap();

            if !old_queues.contains(shape) {
                for &new_board in &edges[*shape as usize] {
                    if !culled.contains(&new_board) {
                        continue;
                    }

                    let mut lock = next.get_shard_guard(&new_board);
                    let next_queues: &mut HashSet<Shape> = lock.entry(new_board).or_default();
                    next_queues.extend(&old_queues)
                }
            }

            for hold_piece in old_queues {
                for &new_board in &edges[hold_piece as usize] {
                    if !culled.contains(&new_board) {
                        continue;
                    }

                    let mut lock = next.get_shard_guard(&new_board);
                    let next_queues: &mut HashSet<Shape> = lock.entry(new_board).or_default();
                    next_queues.insert(*shape);
                }
            }
        });
        prev = next;
    }

    if prev.len() == 0 {
        return 0;
    }

    let blank_queue_state = {
        let mut state = QueueState(bags.first().unwrap().full);
        for ((i, bag), shape) in bags
            .iter()
            .flat_map(|b| (0..b.count).into_iter().map(move |i| (i, b)))
            .zip(start_queue)
        {
            let s = if i == 0 { state.next(bag) } else { state };
            state = s.take(bag, *shape).unwrap();
        }
        state
    };
    let mut max = 0;
    let prev_len = prev.len();
    for (i, (board, holds)) in prev.into_iter().enumerate() {
        println!("testing: {i}/{prev_len}");
        let queuemap: QueueMap = holds
            .into_iter()
            .map(|shape| {
                let mut h = HashSet::new();
                h.insert(Queue::empty());
                (blank_queue_state.force_swap(shape), h)
            })
            .collect();

        let mut prev: ShardedHashMap<Board, QueueMap, 20, nohash::BuildNoHashHasher<u64>> =
            ShardedHashMap::new();
        prev.insert(board, queuemap);
        for (_stage, (bag, i)) in bags
            .iter()
            .flat_map(|b| (0..b.count).into_iter().map(move |i| (b, i)))
            .skip(start_queue.len())
            .take(4)
            .enumerate()
        {
            let next = ShardedHashMap::new();

            prev.into_par_iter().for_each(|(old_board, old_queues)| {
                for (shape, new_boards) in gigapan
                    .get(&old_board)
                    .unwrap()
                    .into_iter()
                    .enumerate()
                {
                    let shape = Shape::try_from(shape as u8).unwrap();

                    let new_queues = bag.take_with_history(&old_queues, shape, i == 0, true);

                    if new_queues.is_empty() {
                        continue;
                    }

                    for &new_board in new_boards {
                        if !culled.contains(&new_board) {
                            continue;
                        }
                        let mut lock = next.get_shard_guard(&new_board);

                        let next_queues: &mut QueueMap = lock.entry(new_board).or_default();
                        for (&state, queues) in &new_queues {
                            next_queues.entry(state).or_default().extend(queues);
                        }
                    }
                }
            });
            prev = next;
        }

        if prev.len() == 0 {
            continue;
        }
        let mut map = HashSet::new();
        for (_b, q) in prev.into_iter().collect::<Vec<_>>() {
            for q in q.into_values() {
                map.extend(q);
            }
        }
        if map.len() == 24 {
            println!("ff?{board}");
            for m in map.iter() {
                println!("{m}");
            }
            return 24;
        }
        if map.len() > max {
            max = map.len()
        }
    }
    max
}

fn build_path(gigapan: &FrozenGigapan, start: Board, bags: &[Bag]) -> HashSet<Board> {
    let mut stages = Vec::new();
    let mut prev: ScanStage = HashMap::new();
    let first_queues = bags.first().unwrap().init_hold();

    prev.insert(start, (first_queues, Vec::new()));
    for (_stage, (bag, i)) in bags
        .iter()
        .flat_map(|b| (0..b.count).into_iter().map(move |i| (b, i)))
        .skip(1)
        .enumerate()
    {
        let mut next: ScanStage = HashMap::new();

        for (&old_board, (old_queues, _)) in prev.iter() {
            for (shape, new_boards) in gigapan.get(&old_board).unwrap().into_iter().enumerate() {
                let shape = Shape::try_from(shape as u8).unwrap();
                let new_queues = bag.take(old_queues, shape, i == 0, true);

                if new_queues.is_empty() {
                    continue;
                }

                for &new_board in new_boards {
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
    assert!(prev.len() == 1);
    stages.push(prev);

    let mut culled = HashSet::new();
    let mut iter = stages.iter().rev();

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
    culled
}
