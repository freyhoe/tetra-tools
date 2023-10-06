use crate::boardgraph::FrozenGigapan;
use crate::queue::{Bag, QueueState, get_queue_permutations};

use hashbrown::{HashMap, HashSet};
use srs_4l::gameplay::{Board, Shape};

type ScanStage = HashMap<Board, (Vec<QueueState>, Vec<Board>)>;

pub fn limited_see_chance(gigapan: &FrozenGigapan, board: Board, bags: &[(u8, Bag)]) {
    let piece_count: usize = bags.len()-1;
    let new_mino_count = piece_count as u32 * 4;
    if board.0.count_ones() + new_mino_count != 40 {
        println!("bad queue len");
        return;
    }

    let culled = build_path(&gigapan, board, bags);

//IZJSSJL
    best_moves(gigapan, &culled, board, bags, &[ Shape::I, Shape::Z, Shape::J, Shape::S, Shape::S, Shape::J, Shape::L])
}

pub fn best_moves(gigapan: &FrozenGigapan, culled: &HashSet<Board>, board: Board, counted_bags: &[(u8, Bag)], queue: &[Shape]){
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
            }
        }
    }

    let mut max = 0;

    for ((board,hold), (_,queues)) in &stages[1]{
        if queues.len()>= max{
            max = queues.len();
            println!("{} {:?} {}", board, hold, max);
        }
    }



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


fn build_path(gigapan: &FrozenGigapan, start: Board, bags: &[(u8, Bag)]) -> HashSet<Board> {
    let mut stages = Vec::new();
    let mut prev: ScanStage = HashMap::new();
    let first_queues = bags.first().unwrap().1.init_hold();

    prev.insert(start, (first_queues, Vec::new()));
    for (_stage, (i, bag)) in bags.iter()
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
