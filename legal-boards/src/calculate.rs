use std::collections::VecDeque;
use std::fmt::Write;
use std::iter::FromIterator;

use crate::boardgraph::FrozenGigapan;
use crate::queue::{Bag, QueueState, get_queue_permutations};

use hashbrown::{HashMap, HashSet};
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use srs_4l::gameplay::{Board, Shape};

type ScanStage = HashMap<Board, (Vec<QueueState>, Vec<Board>)>;

pub fn limited_see_chance(gigapan: &FrozenGigapan, board: Board, counted_bags: &[(u8, Bag)]) {
    let piece_count: usize = counted_bags.len()-1;
    let new_mino_count = piece_count as u32 * 4;
    if board.0.count_ones() + new_mino_count != 40 {
        println!("bad queue len");
        return;
    }

    let culled = build_path(&gigapan, board, counted_bags);

    println!("found {} total possible boards", culled.len());

    let permutations = get_queue_permutations(counted_bags, None, Some(7));
    
    /*use Shape::*;
    let test_queue = vec![I,Z,J,S,S,J,L];
     
    let res = best_moves(gigapan, &culled, board, counted_bags, &test_queue);
    println!("res {res}");*/
     
    let bar = indicatif::ProgressBar::new(permutations.len() as u64);
    bar.set_style(indicatif::ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{bar:.cyan/blue}] {pos}/{human_len} ({eta})")
    .unwrap()
    .with_key("eta", |state: &indicatif::ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
    .progress_chars("#>-"));

    let fails : Vec<_>= permutations.into_par_iter().map(|mut queue|{
        let mut start_queue_state = QueueState(counted_bags.first().unwrap().1.full);
        for ((i, bag), shape) in counted_bags.iter().zip(queue.iter()){
            let s = if i == &0 { start_queue_state.next(bag) } else { start_queue_state };
            start_queue_state = s.take(bag, *shape).unwrap();
        }
    
        let hold = queue.pop_front().unwrap();
        let passed = solve(gigapan, &culled, board, hold, &counted_bags[(1+queue.len())..], start_queue_state, &mut queue, [24, 6, 2, 1], 0);

        bar.inc(1);
        (queue, passed)
    }).collect();

    bar.finish_and_clear();
    let mut total = 0;
    for (fail, covered) in fails{
        total += covered;
        println!("{:?} {}", fail, covered);
    }
    println!("total: {total}");
    println!("computed in: {}",bar.elapsed().as_secs_f64());
    
}

fn solve(gigapan: &FrozenGigapan, culled: &HashSet<Board>, board: Board, hold: Shape, counted_bags: &[(u8, Bag)], state: QueueState, queue: &mut VecDeque<Shape>, cutoffs: [usize; 4], depth: usize)-> usize{
    if depth >= 4{
        return test_set_queue(gigapan, culled, board, queue, hold) as usize;
    }

    let (bag_placement, bag) = &counted_bags[depth];
    let state = if bag_placement == &0{state.next(&bag)}else{state};

    let use_shape = queue.pop_front().unwrap();
    let mut max = 0;

    let edges = gigapan.get(&board).unwrap();
    for &new_board in &edges[use_shape as usize] {
        if !culled.contains(&new_board) {
            continue;
        }
        let mut count = 0;

        for shape in Shape::ALL{
            if let Some(state) = state.take(&bag, shape){
                queue.push_back(shape);
                count += solve(gigapan, culled, new_board, hold, counted_bags, state, queue, cutoffs, depth+1);
                queue.pop_back();
            }
            max = max.max(count);
            if max==cutoffs[depth]{break};
        }
    }
    if use_shape != hold{
        for &new_board in &edges[hold as usize] {
            if !culled.contains(&new_board) {
                continue;
            }
            let mut count = 0;
    
            for shape in Shape::ALL{
                if let Some(state) = state.take(&bag, shape){
                    queue.push_back(shape);
                    count += solve(gigapan, culled, new_board, use_shape, counted_bags, state, queue, cutoffs, depth+1);
                    queue.pop_back();
                }
                max = max.max(count);
                if max==cutoffs[depth]{break};
            }
        }
    }
    queue.push_front(use_shape);
    max
}


fn test_set_queue(
    gigapan: &FrozenGigapan,
    culled: &HashSet<Board>,
    start_board: Board,
    start_queue: &mut VecDeque<Shape>,
    start_hold: Shape
)->bool{
    if start_board == Board::full(){
        return true;
    }
    let use_shape = start_queue.pop_front().unwrap();
    let mut result = false;

    let edges = gigapan.get(&start_board).unwrap();
    for &new_board in &edges[use_shape as usize] {
        if !culled.contains(&new_board) {
            continue;
        }
        if test_set_queue(gigapan, culled, new_board, start_queue, start_hold){
            result = true;break;
        }
    }

    if start_hold != use_shape{
        for &new_board in &edges[start_hold as usize] {
            if !culled.contains(&new_board) {
                continue;
            }
            if test_set_queue(gigapan, culled, new_board, start_queue, use_shape){
                result = true;break;
            }
        }
    }

    start_queue.push_front(use_shape);
    return result;
}


fn build_path(gigapan: &FrozenGigapan, start: Board, counted_bags: &[(u8, Bag)]) -> HashSet<Board> {
    let mut stages = Vec::new();
    let mut prev: ScanStage = HashMap::new();
    let first_queues = counted_bags.first().unwrap().1.init_hold();

    prev.insert(start, (first_queues, Vec::new()));
    for (_stage, (i, bag)) in counted_bags.iter()
        .skip(1)
        .enumerate()
    {
        let mut next: ScanStage = HashMap::new();

        for (&old_board, (old_queues, _)) in prev.iter() {
            for (shape, new_boards) in gigapan.get(&old_board).unwrap().into_iter().enumerate() {
                let shape = Shape::try_from(shape as u8).unwrap();
                let new_queues = bag.take(old_queues, shape, i == &0, true);

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
