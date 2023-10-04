use std::collections::VecDeque;
use std::fmt::Write;
use std::time::Instant;

use crate::boardgraph::FrozenGigapan;
use crate::queue::{Bag, QueueMap, QueueState};

use hashbrown::{HashMap, HashSet};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator, IntoParallelIterator};
use srs_4l::gameplay::{Board, Shape};
use srs_4l::queue::Queue;

type ScanStage = HashMap<Board, (Vec<QueueState>, Vec<Board>)>;

pub fn chance(gigapan: &FrozenGigapan, board: Board, bags: &[Bag]) {
    let piece_count: usize = bags.iter().map(|b| b.count as usize).sum();
    let new_mino_count = piece_count as u32 * 4;
    if board.0.count_ones() + new_mino_count - 4 != 40 {
        println!("bad queue len");
        return;
    }

    let culled = build_path(&gigapan, board, bags);
    println!("culled generated");
    see_x_chance(
        gigapan,
        &culled,
        bags,
        board,
        7
    );
}

fn see_x_chance(
    gigapan: &FrozenGigapan,
    culled: &HashSet<Board>,
    bags: &[Bag],
    start_board: Board,
    see: usize
){
        let next_bags :Vec<_> = bags
        .iter()
        .flat_map(|b| (0..b.count).into_iter().map(move |i| (i, b)))
        .take(see).collect();

    let mut queue = VecDeque::new();
    let mut permutations = Vec::new();

    permute_bags(&next_bags, &mut permutations, 0, QueueState(next_bags.first().unwrap().1.full), &mut queue);


    let test_queue = vec![Shape::I, Shape::Z, Shape::J, Shape::S,Shape::S,Shape::J,Shape::L];
    test_set_queue(gigapan, culled, bags, start_board, &test_queue);
    /*let bar = indicatif::ProgressBar::new(permutations.len() as u64);

    bar.set_style(indicatif::ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{bar:.cyan/blue}] {pos}/{human_len} ({eta})")
    .unwrap()
    .with_key("eta", |state: &indicatif::ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
    .progress_chars("#>-"));

    let fails : Vec<_>= permutations.into_iter().filter_map(|queue|{
        let passed = test_set_queue(gigapan, culled, bags, start_board, &queue);
        bar.inc(1);
        if !passed{
            Some(queue)
        }else{
            None
        }
    }).collect();

    for fail in fails{
        println!("{:?}", fail);
    }*/
}

fn set_queue_path(
    gigapan: &FrozenGigapan,
    culled: &HashSet<Board>,
    start_board: Board,
    start_queue: &[Shape],
    start_holds: HashSet<Shape>
) -> HashMap<Board, HashSet<Shape>>{
    let mut prev = HashMap::new();

    prev.insert(start_board, start_holds);
    for shape in start_queue.iter() {
        let mut next = HashMap::new();

        prev.into_iter().for_each(|(old_board, old_queues)| {
            let edges: &[Vec<Board>; 7] = gigapan.get(&old_board).unwrap();

            if !old_queues.contains(shape) {
                for &new_board in &edges[*shape as usize] {
                    if !culled.contains(&new_board) {
                        continue;
                    }

                    let next_queues: &mut HashSet<Shape> = next.entry(new_board).or_default();
                    next_queues.extend(&old_queues)
                }
            }

            for hold_piece in old_queues {
                for &new_board in &edges[hold_piece as usize] {
                    if !culled.contains(&new_board) {
                        continue;
                    }

                    let next_queues: &mut HashSet<Shape> = next.entry(new_board).or_default();
                    next_queues.insert(*shape);
                }
            }
        });
        prev = next;
    }
    prev
}

fn set_queue_graph(
    gigapan: &FrozenGigapan,
    culled: &HashSet<Board>,
    start_board: Board,
    start_queue: &[Shape],
    start_holds: HashSet<Shape>
) -> HashMap<(Board, Shape), (Vec<Board>, HashSet<Vec<Shape>>)>{
    let mut prev = HashMap::new();

    for hold in start_holds{
        prev.insert((start_board, hold), (Vec::new(), HashSet::new()));
    }
    for &shape in start_queue.iter() {
        let mut next = HashMap::new();

        prev.into_iter().for_each(|((old_board, old_hold), _preds)| {
            let edges: &[Vec<Board>; 7] = gigapan.get(&old_board).unwrap();

            for &new_board in &edges[old_hold as usize] {
                if !culled.contains(&new_board) {
                    continue;
                }
                let (preds, _): &mut (Vec<Board>, HashSet<Vec<Shape>>) = next.entry((new_board, shape)).or_default();
                if !preds.contains(&old_board){
                    preds.push(old_board);
                }
            }
            if old_hold!=shape{
                for &new_board in &edges[shape as usize] {
                    if !culled.contains(&new_board) {
                        continue;
                    }
                    let (preds, _): &mut (Vec<Board>, HashSet<Vec<Shape>>) = next.entry((new_board, shape)).or_default();
                    if !preds.contains(&old_board){
                        preds.push(old_board);
                    }
                }
            }
        });
        prev = next;
    }
    prev
}

fn test_set_queue_path(
    gigapan: &FrozenGigapan,
    culled: &HashSet<Board>,
    start_board: Board,
    start_queue: &[Shape],
    start_holds: HashSet<Shape>
)->bool{
    if start_queue.len()==0{return true}
    let use_shape = start_queue[0];

    let edges = gigapan.get(&start_board).unwrap();

    if !start_holds.contains(&use_shape){
        for &new_board in &edges[use_shape as usize] {
            if !culled.contains(&new_board) {
                continue;
            }

            let mut next_holds: HashSet<Shape> = HashSet::with_capacity(7);
            next_holds.extend(&start_holds);

            if test_set_queue_path(gigapan, culled, new_board, &start_queue[1..], next_holds){
                return true;
            }
        }
    }
    for hold_piece in start_holds {
        for &new_board in &edges[hold_piece as usize] {
            if !culled.contains(&new_board) {
                continue;
            }

            let mut next_holds = HashSet::with_capacity(1);
            next_holds.insert(use_shape);

            if test_set_queue_path(gigapan, culled, new_board, &start_queue[1..], next_holds){
                return true;
            }
        }
    }

    false
}

fn test_set_queue(
    gigapan: &FrozenGigapan,
    culled: &HashSet<Board>,
    bags: &[Bag],
    start_board: Board,
    start_queue: &[Shape],
) -> bool {


    let mut first_holds = HashSet::with_capacity(1);
    first_holds.insert(*start_queue.first().unwrap());


    let edge_boards = set_queue_path(gigapan, culled, start_board, &start_queue[1..], first_holds);

    if edge_boards.len() == 0 {
        return false
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

    let next_bags :Vec<_> = bags
        .iter()
        .flat_map(|b| (0..b.count).into_iter().map(move |i| (i, b)))
        .skip(start_queue.len()).collect();

    let mut queue = VecDeque::new();
    let mut permutations = Vec::new();

    permute_bags(&next_bags, &mut permutations, 0, blank_queue_state, &mut queue);

    let mut max = 0;
    for (_i, (board, holds)) in edge_boards.into_iter().enumerate() {
        let mut count = 0;
        for permutation in &permutations{
            let res = test_set_queue_path(gigapan, culled, board, permutation, holds.clone());
            if res {count+=1}
        }
        if count > max{
            max = count;
            println!("new max: {board} {:?} {}", holds, count);
        }
    }
    false
    
}

fn permute_bags(bags: &[(u8, &Bag)], permutations: &mut Vec<Vec<Shape>>, depth: usize, state: QueueState, queue: &mut VecDeque<Shape>){
    if depth == bags.len(){
        permutations.push(queue.iter().cloned().collect());
        return
    }
    let (bag_placement, bag) = &bags[depth];
    let state = if bag_placement == &0{state.next(&bag)}else{state};
    for shape in Shape::ALL{
        if let Some(state) = state.take(&bag, shape){
            queue.push_back(shape);
            permute_bags(bags, permutations, depth+1, state, queue);
            queue.pop_back();
        }
    }
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
