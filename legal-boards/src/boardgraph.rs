use std::{io::Write, time::Duration};

use rayon::{
    iter::{IntoParallelRefMutIterator, ParallelIterator},
    prelude::*,
};
use smallvec::SmallVec;

use compute::{Counter, ShardedHashMap, FrozenMap};
use srs_4l::{
    gameplay::{Board, Shape},
    vector::Placements,
};


type NoHashBuilder = nohash::BuildNoHashHasher<u64>;
type Map = ShardedHashMap<Board, SmallVec<[Board; 6]>, 20, NoHashBuilder>;
type GraphMap = ShardedHashMap<Board, SmallVec<[(Board, Shape); 6]>, 20, NoHashBuilder>;

pub type Gigapan = ShardedHashMap<Board, [Vec<Board>;7], 20, NoHashBuilder>;
pub type FrozenGigapan = FrozenMap<Board, [Vec<Board>;7], 20, NoHashBuilder>;

type Set = ShardedHashMap<Board, (), 20, NoHashBuilder>;

pub fn compute() -> Vec<Board> {
    let mut stages: Vec<Map> = Vec::new();
    stages.resize_with(11, Map::new);

    stages[0].insert(Board::empty(), SmallVec::new());

    for iter in 1..=10 {
        let (prev_stage, this_stage) = match &mut stages[iter - 1..] {
            [prev, this, ..] => (prev, this),
            _ => unreachable!(),
        };

        let counter = Counter::zero();
        let total = prev_stage.len();

        crossbeam::scope(|s| {
            s.spawn(|_| loop {
                let counted = counter.get();
                eprint!(
                    "\r{:>12} / {:>12} ({:>6.2}%)",
                    counted,
                    total,
                    (counted as f64) / (total as f64) * 100.
                );
                std::io::stdout().flush().unwrap();
                if counted == total as u64 {
                    return;
                }
                std::thread::sleep(Duration::from_millis(100));
            });

            prev_stage.par_iter_mut().for_each(|(&board, _preds)| {
                for shape in Shape::ALL {
                    for (_piece, new_board) in Placements::place(board, shape).canonical() {
                        if new_board.has_isolated_cell() || new_board.has_imbalanced_split() {
                            continue;
                        }

                        let mut guard = this_stage.get_shard_guard(&new_board);
                        let preds = guard.entry(new_board).or_default();
                        if !preds.contains(&board) {
                            preds.push(board);
                        }
                    }
                }
                counter.increment();
            });
        })
        .unwrap();

        eprintln!();
    }

    let stages: Vec<_> = stages.drain(..).map(ShardedHashMap::freeze).collect();

    const FULL: Board = Board(0xFFFFF_FFFFF);
    let mut work = {
        let work = Set::new();
        work.insert(FULL, ());
        work.freeze()
    };
    let mut all_boards = vec![FULL];

    for (i, stage) in stages.iter().enumerate().rev() {
        println!("{:>4}-piece boards: {:>9}", i, work.len());

        work = work
            .par_iter()
            .flat_map_iter(|(&board, ())| stage.get(&board).unwrap())
            .map(|&board| (board, ()))
            .collect();

        all_boards.extend(work.iter().map(|(&board, ())| board));
    }

    // Dropping the stages takes a long time.  We're almost done anyway.
    std::mem::forget(stages);

    println!("sorting...");
    all_boards.par_sort_unstable();
    println!("sorted.");
    all_boards
}


pub fn compute_gigapan() -> (Gigapan, Gigapan){
    let mut stages: Vec<GraphMap> = Vec::new();
    stages.resize_with(11, GraphMap::new);

    stages[0].insert(Board::empty(), SmallVec::new());

    for iter in 1..=10 {
        let (prev_stage, this_stage) = match &mut stages[iter - 1..] {
            [prev, this, ..] => (prev, this),
            _ => unreachable!(),
        };

        let counter = Counter::zero();
        let total = prev_stage.len();

        crossbeam::scope(|s| {
            s.spawn(|_| loop {
                let counted = counter.get();
                eprint!(
                    "\r{:>12} / {:>12} ({:>6.2}%)",
                    counted,
                    total,
                    (counted as f64) / (total as f64) * 100.
                );
                std::io::stdout().flush().unwrap();
                if counted == total as u64 {
                    return;
                }
                std::thread::sleep(Duration::from_millis(100));
            });

            prev_stage.par_iter_mut().for_each(|(&board, _preds)| {
                for shape in Shape::ALL {
                    for (_piece, new_board) in Placements::place(board, shape).canonical() {
                        if new_board.has_isolated_cell() || new_board.has_imbalanced_split() {
                            continue;
                        }

                        let mut guard = this_stage.get_shard_guard(&new_board);
                        let preds = guard.entry(new_board).or_default();
                        if !preds.contains(&(board,shape)) {
                            preds.push((board,shape));
                        }
                    }
                }
                counter.increment();
            });
        })
        .unwrap();

        eprintln!();
    }

    // progressively drop the stages when we are done with them
    let stages = stages.drain(..).map(ShardedHashMap::freeze);

    const FULL: Board = Board(0xFFFFF_FFFFF);

    let graphmap = Gigapan::new();
    let reversemap = Gigapan::new();

    let mut work = {
        let work = Set::new();
        work.insert(FULL, ());
        work.freeze()
    };

    for (i, stage) in stages.enumerate().rev() {
        println!("{:>4}-piece boards: {:>9}", i, work.len());

        work = work
            .par_iter()
            .flat_map_iter(|(&board, ())| {
                let preds = stage.get(&board).unwrap();

                // let mut shard = reversemap.get_shard_guard(&board);
                // let entry = shard.entry(board).or_insert_with(Default::default);
                // preds.iter().for_each(|&(parent, shape)|{
                //     entry[shape as usize].push(parent);
                // });

                preds.iter().for_each(|&(parent, shape)|{
                    let mut shard = graphmap.get_shard_guard(&parent);
                    let entry = shard.entry(parent).or_insert_with(Default::default);
                    entry[shape as usize].push(board);
                });

                preds.iter().map(|&(parent, _shape)|{
                    (parent, ())
                })
            })
            .collect();
    }
    // std::mem::forget(stages);
    (graphmap,reversemap)
}