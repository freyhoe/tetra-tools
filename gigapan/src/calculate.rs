use std::collections::VecDeque;
use std::fmt::Write as FmtWrite;
use std::time::Instant;

use legal_boards::boardgraph::FrozenGigapan;
use crate::queue::{Bag, QueueState, get_queue_permutations, CombinatoricQueue};
use hashbrown::HashSet;
use compute::ShardedHashMap;

use rayon::prelude::{IntoParallelIterator, ParallelIterator, IntoParallelRefMutIterator};
use srs_4l::gameplay::{Board, Shape};

type NoHashBuilder = nohash::BuildNoHashHasher<u64>;
type ScanStage = ShardedHashMap<Board, (Vec<QueueState>, Vec<Board>), 20, NoHashBuilder>;

use std::fs::File;
use std::io::{Write, LineWriter};

pub fn limited_see_chance(
    gigapan: &FrozenGigapan,
    board: Board,
    combinatoric_queue: &CombinatoricQueue,
    previews: usize,
    init_hold: bool,
    use_hold: bool,
    generate_culled: bool,
    two_line: bool
) {
    let counted_bags = &combinatoric_queue.get_counted_bags();

    let piece_count: usize = counted_bags.len()-1;
    let new_mino_count = piece_count as u32 * 4;
    if board.0.count_ones() + new_mino_count != 40 {
        eprintln!("bad queue len");
        return;
    }
    let culled = if generate_culled{
        let instant = Instant::now();
        let c = get_culled_boards(&gigapan, board, counted_bags, use_hold);
        eprintln!("found {} total possible path boards in {:?}", c.len(), instant.elapsed());
        Some(c)
    }else{
        None
    };
    let culled = culled.as_ref();

    let permutations = get_queue_permutations(counted_bags, None, Some(previews+1+init_hold as usize));
     
    let bar = indicatif::ProgressBar::new(permutations.len() as u64);
    bar.set_style(indicatif::ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{bar:.cyan/blue}] {pos}/{human_len} queues ({eta})")
    .unwrap()
    .with_key("eta", |state: &indicatif::ProgressState, w: &mut dyn FmtWrite| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
    .progress_chars("#>-"));

    let fails : Vec<_>= permutations.into_par_iter().map(|mut queue|{
        let mut start_queue_state = QueueState(counted_bags.first().unwrap().1.full);
        for ((i, bag), shape) in counted_bags.iter().zip(queue.iter()){
            let s = if i == &0 { start_queue_state.next(bag) } else { start_queue_state };
            start_queue_state = s.take(bag, *shape).unwrap();
        }
        let revealed_pieces = queue.len();
        let hold = if init_hold{
            Some(queue.pop_front().unwrap())
        }else{None};

        let passed = max_limited_see_queues(gigapan, culled, board, hold, use_hold, two_line, counted_bags, start_queue_state, &mut queue, revealed_pieces);
        if let Some(hold) = hold{queue.push_front(hold);}
        bar.inc(1);
        (queue, passed)
    }).collect();

    bar.finish_and_clear();

    let file = File::create("fail-queues.txt").unwrap();
    let mut file = LineWriter::new(file);

    let mut total = 0;
    for (fail, (covered, maximum)) in fails{
        total += covered;
        let maximum = maximum.unwrap_or(0);
        if covered != maximum{
            file.write_fmt(format_args!("{:?} {} {}\n", fail, covered, maximum)).unwrap();
        }
    }
    let total_queues = combinatoric_queue.queue_count();
    println!("passing queues: {total}/{}",total_queues);
    println!("chance: {}%", total as f64 / total_queues as f64 * 100.0);
    eprintln!("computed in: {:.3}s",bar.elapsed().as_secs_f64());


    
}

fn count_possible_queues(
    counted_bags: &[(u8, Bag)],
    queue_state: QueueState,
    revealed_pieces: usize
)->usize{
    //todo: optimize this by actually doing the math, atm its annoying with counted_bags setup

    let mut count = 0;

    let (bag_placement, bag) = &counted_bags[revealed_pieces];
    let queue_state = if bag_placement == &0{queue_state.next(&bag)}else{queue_state};

    for shape in Shape::ALL{
        if let Some(queue_state) = queue_state.take(&bag, shape){
            count += count_possible_queues(counted_bags, queue_state, revealed_pieces+1)
        }
    }
    return count
}

///DFS search to find the maximum found hidden queues that conform to limited see, and the maximum possible hidden queues
fn max_limited_see_queues(
    gigapan: &FrozenGigapan,
    culled: Option<&HashSet<Board>>,
    board: Board,
    hold: Option<Shape>,
    use_hold: bool,
    two_line: bool,
    counted_bags: &[(u8, Bag)],
    queue_state: QueueState,
    queue: &mut VecDeque<Shape>,
    revealed_pieces: usize)-> (usize, Option<usize>){

    if two_line && board == Board::half(){ // will only happen on low see i think, just in case
        let total = count_possible_queues(counted_bags, queue_state, revealed_pieces);
        return (total, Some(total))
    }

    if revealed_pieces >= counted_bags.len(){
        let res =  match hold{
            Some(hold) => {
                if use_hold{
                    test_set_queue_with_hold(gigapan, culled, board, queue, hold, two_line)
                }else{
                    test_set_queue_without_hold(gigapan, culled, board, queue, two_line)
                }
            },
            None => {
                if use_hold{
                    let new_hold = queue.pop_front().unwrap();
                    let res = test_set_queue_with_hold(gigapan, culled, board, queue, new_hold, two_line);
                    queue.push_front(new_hold);
                    res
                }else{
                    test_set_queue_without_hold(gigapan, culled, board, queue, two_line)
                }
            },
        };
        return (res as usize, Some(1))
    }

    let (bag_placement, bag) = &counted_bags[revealed_pieces];
    let queue_state = if bag_placement == &0{queue_state.next(&bag)}else{queue_state};

    let use_shape = queue.pop_front().unwrap();
    let mut max = 0;

    let edges = gigapan.get(&board).unwrap();
    let next_states: Vec<_> = Shape::ALL.iter().filter_map(|&shape|{
        if let Some(queue_state) = queue_state.take(&bag, shape){
            Some((shape, queue_state))
        }else{
            None
        }
    }).collect();


    let mut cutoffs = vec![None; next_states.len()];

    let mut oqb_skip = false;
    let mut max_found = false;
 
    if use_hold && hold.is_none(){
        if let Some(next_shape) = queue.front(){
            if next_shape == &use_shape{
                oqb_skip = true; //if current and next are equal, and theres no piece in hold, the best choice is always to hold to reveal extra piece
            }
        }

        let mut count = 0;
        let mut max_count = 0;

        for (idx, &(shape, queue_state)) in next_states.iter().enumerate(){
            queue.push_back(shape);
            let (next_count, next_possible_queues) = max_limited_see_queues(gigapan, culled, board, Some(use_shape), use_hold, two_line, counted_bags, queue_state, queue, revealed_pieces+1);
            count += next_count;
            if let Some(next_possible_queues) = next_possible_queues{
                if next_count == next_possible_queues{max_count+=1;}
                if cutoffs[idx].is_none(){cutoffs[idx] = Some(next_possible_queues)}
            }
            queue.pop_back();
        }
        if count > max{
            max = count;
        }
        if max_count==next_states.len(){max_found = true;}
    }

    if !oqb_skip && !max_found{
        for &new_board in &edges[use_shape as usize] {
            if let Some(culled) = culled{if !culled.contains(&new_board){continue;}}
            let mut count = 0;
            let mut max_count = 0;
    
            for (idx, &(shape, queue_state)) in next_states.iter().enumerate(){
                queue.push_back(shape);

                let (next_count, next_possible_queues) = max_limited_see_queues(gigapan, culled, new_board, hold, use_hold, two_line, counted_bags, queue_state, queue, revealed_pieces+1);
                count += next_count;
                if let Some(next_possible_queues) = next_possible_queues{
                    if next_count == next_possible_queues{max_count+=1;}
                    if cutoffs[idx].is_none(){cutoffs[idx] = Some(next_possible_queues)}
                }
                queue.pop_back();
            }
            if count > max{
                max = count;
            }
            if max_count==next_states.len(){max_found = true; break;}
        }
    }
    if use_hold && hold.is_some() && !max_found{
        let hold = hold.unwrap();
        if use_shape != hold{
            for &new_board in &edges[hold as usize] {
                if let Some(culled) = culled{if !culled.contains(&new_board){continue;}}
                let mut count = 0;
                let mut max_count = 0;
        
                for (idx, &(shape, queue_state)) in next_states.iter().enumerate(){
                    queue.push_back(shape);
                    let (next_count, next_possible_queues) = max_limited_see_queues(gigapan, culled, new_board, Some(use_shape), use_hold, two_line, counted_bags, queue_state, queue, revealed_pieces+1);
                    count += next_count;
                    if let Some(next_possible_queues) = next_possible_queues{
                        if next_count == next_possible_queues{max_count+=1;}
                        if cutoffs[idx].is_none(){cutoffs[idx] = Some(next_possible_queues)}
                    }
                    queue.pop_back();
                }
                if count > max{
                    max = count;
                }
                if max_count==next_states.len(){break;}
            }
        }
    }

    queue.push_front(use_shape);

    let mut next_cutoff = 0;
    for cutoff in cutoffs{
        match cutoff{
            Some(cutoff)=>{next_cutoff+=cutoff},
            None=> return (max, None)
        }
    }

    (max, Some(next_cutoff))
}

///DFS search to see if the given (board,queue,hold) state achieved PC
fn test_set_queue_with_hold(
    gigapan: &FrozenGigapan,
    culled: Option<&HashSet<Board>>,
    start_board: Board,
    start_queue: &mut VecDeque<Shape>,
    start_hold: Shape,
    two_line: bool
)->bool{
    if start_board == Board::full() || (two_line && start_board==Board::half()){
        return true;
    }
    let use_shape = start_queue.pop_front().unwrap();
    let mut result = false;

    let edges = gigapan.get(&start_board).unwrap();
    for &new_board in &edges[use_shape as usize] {
        if let Some(culled) = culled{if !culled.contains(&new_board){continue;}}
        if test_set_queue_with_hold(gigapan, culled, new_board, start_queue, start_hold, two_line){
            result = true;break;
        }
    }

    if start_hold != use_shape{
        for &new_board in &edges[start_hold as usize] {
            if let Some(culled) = culled{if !culled.contains(&new_board){continue;}}
            if test_set_queue_with_hold(gigapan, culled, new_board, start_queue, use_shape, two_line){
                result = true;break;
            }
        }
    }

    start_queue.push_front(use_shape);
    return result;
}

fn test_set_queue_without_hold(
    gigapan: &FrozenGigapan,
    culled: Option<&HashSet<Board>>,
    start_board: Board,
    start_queue: &mut VecDeque<Shape>,
    two_line: bool
)->bool{
    if start_board == Board::full() || (two_line && start_board==Board::half()){
        return true;
    }
    let use_shape = start_queue.pop_front().unwrap();
    let mut result = false;

    let edges = gigapan.get(&start_board).unwrap();
    for &new_board in &edges[use_shape as usize] {
        if let Some(culled) = culled{if !culled.contains(&new_board){continue;}}
        if test_set_queue_without_hold(gigapan, culled, new_board, start_queue, two_line){
            result = true;break;
        }
    }

    start_queue.push_front(use_shape);
    return result;
}

///This function returns a hashset of boards that will reach a perfect clear if they are achieved by the current combinatoric queue input
fn get_culled_boards(gigapan: &FrozenGigapan, start: Board, counted_bags: &[(u8, Bag)], use_hold: bool) -> HashSet<Board> {
    let mut stages = Vec::new();
    let mut prev: ScanStage = ShardedHashMap::new();
    let first_queues = counted_bags.first().unwrap().1.init_hold();

    prev.insert(start, (first_queues, Vec::new()));
    for (_stage, (i, bag)) in counted_bags.iter()
        .skip(1)
        .enumerate()
    {
        let next: ScanStage = ShardedHashMap::new();

        prev.par_iter_mut().for_each(|(&old_board, (old_queues, _))|{
            for (shape, new_boards) in gigapan.get(&old_board).unwrap().into_iter().enumerate() {
                let shape = Shape::try_from(shape as u8).unwrap();
                let new_queues = bag.take(old_queues, shape, i == &0, use_hold);

                if new_queues.is_empty() {
                    continue;
                }

                for &new_board in new_boards {
                    let mut lock = next.get_shard_guard(&new_board);
                    let (queues, preds) = lock.entry(new_board).or_default();
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
        });
        stages.push(prev);
        prev = next;
    }
    assert!(prev.len() == 1, "is a perfect clear even possible?");
    stages.push(prev);


    let mut culled = HashSet::new();
    let mut iter = stages.into_iter().rev();

    if let Some(final_stage) = iter.next() {
        final_stage.into_iter().for_each(|(board, (_queues, preds))|{
            culled.insert(board);
            for board in preds{
                culled.insert(board);
            }
        });
    }

    for stage in iter {
        stage.into_iter().for_each(|(board, (_queues, preds))|{
            if culled.contains(&board){
                for board in preds{
                    culled.insert(board);
                }
            };
        });
    }
    culled
}
