use std::{io::Write, time::Duration};

use rayon::{
    iter::ParallelIterator,
    prelude::*,
};
use smallvec::SmallVec;

use compute::Counter;
use srs_4l::{
    gameplay::{Board, Shape},
    vector::Placements,
};

use dashmap::{DashMap, DashSet};


type GraphMap = DashMap<Board, SmallVec<[(Board, Shape); 6]>>;

pub type Gigapan = DashMap<Board, [Vec<Board>;7]>;

type Set = DashSet<Board>;

pub fn compute_gigapan() -> Gigapan{
    let mut stages: Vec<GraphMap> = Vec::new();
    stages.resize_with(11, ||{GraphMap::with_shard_amount(32)});

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

            prev_stage.into_par_iter().for_each(|entry| {
                let board = *entry.key();
                for shape in Shape::ALL {
                    for (_piece, new_board) in Placements::place(board, shape).canonical() {
                        if new_board.has_isolated_cell() || new_board.has_imbalanced_split() {
                            continue;
                        }
                        let mut entry = this_stage.entry(new_board).or_default();
                        let preds = entry.value_mut();

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

    let stages: Vec<_> = stages.drain(..).collect();

    const FULL: Board = Board(0xFFFFF_FFFFF);

    let graphmap = Gigapan::with_shard_amount(32);

    let mut work = {
        let work = Set::new();
        work.insert(FULL);
        work
    };

    for (i, stage) in stages.iter().enumerate().rev() {
        println!("{:>4}-piece boards: {:>9}", i, work.len());

        work = work
            .par_iter()
            .flat_map_iter(|entry| {
                let board = entry.key();
                let v = stage.get(&board).unwrap();
                let preds = v.value();

                preds.iter().for_each(|&(parent, shape)|{
                    let mut entry = graphmap.entry(parent).or_insert_with(Default::default);

                    entry.value_mut()[shape as usize].push(*board);
                });

                preds.iter().map(|&(parent, _shape)|{
                    parent
                }).collect::<Vec<_>>()
            })
            .collect();
    }
    std::mem::forget(stages);
    graphmap
}