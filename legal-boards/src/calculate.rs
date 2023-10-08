use std::fmt::Write;

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
    
    use Shape::*;
    let test_queue = vec![I,Z,J,S,S,J,L];
     
    let res = best_moves(gigapan, &culled, board, counted_bags, &test_queue);
    println!("res {res}");
    /* 
    let bar = indicatif::ProgressBar::new(permutations.len() as u64);
    bar.set_style(indicatif::ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{bar:.cyan/blue}] {pos}/{human_len} ({eta})")
    .unwrap()
    .with_key("eta", |state: &indicatif::ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
    .progress_chars("#>-"));

    let fails : Vec<_>= permutations.into_par_iter().filter_map(|queue|{
        let passed = best_moves(gigapan, &culled, board, counted_bags, &queue);
        bar.inc(1);

        if queue == test_queue{
            println!("test_queue passed? : {passed}==24");
        }
        let passed = best_moves(gigapan, &culled, board, counted_bags, &queue);
        bar.inc(1);
        if passed==24{
            None
        }else{
            println!("fail {:?}", queue);
            Some((queue, passed))
        }
    }).collect();
    println!("failed len: {}", fails.len());
    for (fail, covered) in fails{
        println!("{:?} {}", fail, covered);
    }
    println!("computed in: {}",bar.elapsed().as_secs_f64());
    */
}

pub fn best_moves(gigapan: &FrozenGigapan, culled: &HashSet<Board>, board: Board, counted_bags: &[(u8, Bag)], queue: &[Shape])->usize{
    let mut start_queue_state = QueueState(counted_bags.first().unwrap().1.full);
    for ((i, bag), shape) in counted_bags.iter().zip(queue.iter()){
        let s = if i == &0 { start_queue_state.next(bag) } else { start_queue_state };
        start_queue_state = s.take(bag, *shape).unwrap();

    }
    let permutations = get_queue_permutations(counted_bags, Some((queue.len(), start_queue_state)), None);


    let mut stages = Vec::new();
    let mut prev = HashMap::new();

    let first_hold = queue.first().unwrap().clone();

    prev.insert((board, first_hold), (Vec::new(), HashSet::new()));

    for &shape in queue.iter().skip(1){
        let mut next = HashMap::new();
        for (&(old_board, old_hold), _) in prev.iter(){
            let edges: &[Vec<Board>; 7] = gigapan.get(&old_board).unwrap();

            for &new_board in &edges[old_hold as usize] {
                if !culled.contains(&new_board){continue;}
                let (preds, _): &mut (Vec<(Board, Shape)>, HashSet<Vec<Shape>>) = next.entry((new_board, shape)).or_default();
                if !preds.contains(&(old_board, old_hold)){
                    preds.push((old_board, old_hold));
                }
            }
            if old_hold!=shape{
                for &new_board in &edges[shape as usize] {
                    if !culled.contains(&new_board){continue;}
                    let (preds, _): &mut (Vec<(Board, Shape)>, HashSet<Vec<Shape>>) = next.entry((new_board, old_hold)).or_default();
                    if !preds.contains(&(old_board, old_hold)){
                        preds.push((old_board, old_hold));
                    }
                }
            }
        }        
        stages.push(prev);
        prev = next;
    }

    for (&(old_board, old_hold), (_, queues)) in prev.iter_mut(){
        let mut holds = HashSet::new();
        holds.insert(old_hold);
        for permutation in &permutations{
            let res = test_set_queue(gigapan, culled, old_board, permutation, holds.clone());
            if res {
                queues.insert(permutation.clone());
            }
        }
    }
    stages.push(prev);



    for idx in (1..stages.len()).rev(){
        let (prev_stage, this_stage) = match &mut stages[(idx - 1)..] {
            [prev, this, ..] => (prev, this),
            _ => unreachable!(),
        };
        for ((_b, _), (pred_boards, queues)) in this_stage{
            for state in pred_boards{
                let (_, pred_queues) = prev_stage.get_mut(state).unwrap();
                pred_queues.extend(queues.iter().cloned());


                if idx > 3 && pred_queues.len()==permutations.len(){
                    return permutations.len()
                }
            }
        }
    }
    //6 2 1
    for permutation in permutations{
        let mut new_stages = Vec::new();
        let mut prev = HashSet::new();


        let cuttoffs = vec![24,6,2,1];
        let cuttoffs = cuttoffs.iter().zip(permutation.iter());

        let first_hold = queue.first().unwrap().clone();
    
        prev.insert((board, first_hold));

        let mut revealed_shapes = Vec::new();
    
        for ((i, &shape), (&cutoff, &revealed_shape)) in queue.iter().enumerate().skip(1).zip(cuttoffs){
            let mut next = HashSet::new();
            for &(old_board, old_hold) in prev.iter(){
                let edges: &[Vec<Board>; 7] = gigapan.get(&old_board).unwrap();
    
                for &new_board in &edges[old_hold as usize] {
                    match stages[i].get(&(new_board, shape)){
                        Some((_, queues)) => {
                            let mut count = 0;
                            for queue in queues{
                                if queue.iter().zip(revealed_shapes.iter()).all(|(a,b)|{
                                    a==b
                                }){count+=1};
                            }
                            if count == cutoff{
                                next.insert((new_board, shape));
                            }
                        },
                        None => continue,
                    }
                }
                if old_hold!=shape{
                    for &new_board in &edges[shape as usize] {
                        match stages[i].get(&(new_board, shape)){
                            Some((_, queues)) => {
                                let mut count = 0;
                                for queue in queues{
                                    if queue.iter().zip(revealed_shapes.iter()).all(|(a,b)|{
                                        a==b
                                    }){count+=1};
                                }
                                if count == cutoff{
                                    next.insert((new_board, shape));
                                }
                            },
                            None => continue,
                        }
                    }
                }
            }
            new_stages.push(prev);
            revealed_shapes.push(revealed_shape);
            prev = next;

            for (board, hold) in prev.iter(){
                println!("{} {:?}", board, hold);
            }
        }
        println!("{:?} {:?}", prev.len(), permutation);
    }
    0
}

fn test_set_queue(
    gigapan: &FrozenGigapan,
    culled: &HashSet<Board>,
    start_board: Board,
    start_queue: &[Shape],
    start_holds: HashSet<Shape>
)->bool{
    if start_board == Board::full(){
        return true;
    }
    let use_shape = start_queue[0];

    let edges = gigapan.get(&start_board).unwrap();

    if !start_holds.contains(&use_shape){
        for &new_board in &edges[use_shape as usize] {
            if !culled.contains(&new_board) {
                continue;
            }

            let mut next_holds: HashSet<Shape> = HashSet::with_capacity(7);
            next_holds.extend(&start_holds);

            if test_set_queue(gigapan, culled, new_board, &start_queue[1..], next_holds){
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

            if test_set_queue(gigapan, culled, new_board, &start_queue[1..], next_holds){
                return true;
            }
        }
    }

    false
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
