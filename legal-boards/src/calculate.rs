use srs_4l::gameplay::{Shape, Board};
use crate::boardgraph::FrozenGigapan;
use crate::queue::{Bag, QueueState};
use hashbrown::{HashMap, HashSet};
use smallvec::SmallVec;

type ScanStage = HashMap<Board, (SmallVec<[QueueState; 7]>, SmallVec< [Board; 7]>)>;


pub fn chance(gigapan: FrozenGigapan, board: Board, bags: &[Bag], total_queues: usize){
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

    transverse_queues(&gigapan, &culled, board, bags, total_queues)
}

fn transverse_queues(gigapan: &FrozenGigapan, culled: &HashSet<Board>, board: Board, bags: &[Bag], total_queues: usize){
    println!("culled boards grabbed {}", culled.len());
    print!("chance start: ");

    let mut prev = HashMap::new();
    let first_queues = bags.first().unwrap().init_hold_with_history();

    prev.insert(board, first_queues);
    for (_stage, (bag, i)) in bags
        .iter()
        .flat_map(|b| (0..b.count).into_iter().map(move |i| (b, i)))
        .skip(1)
        .enumerate()
    {
        let mut next = HashMap::new();

        for (&old_board, old_queues) in prev.iter() {

            for (shape, new_boards) in gigapan.get(&old_board).unwrap().into_iter().enumerate(){
                let shape = Shape::try_from(shape as u8).unwrap();

                let new_queues = bag.take_with_history(old_queues, shape, i==0, true);
                
                if new_queues.is_empty() {
                    continue;
                }

                for &new_board in new_boards{
                    if !culled.contains(&new_board){continue;}
                    
                    let next_queues: &mut HashMap<QueueState, HashSet<srs_4l::queue::Queue>> = next.entry(new_board).or_default();
                    for (&state, queues) in &new_queues {
                        next_queues.entry(state).or_default().extend(queues);
                    }

                }
            }
        }
        print!("b: {}", next.len());

        prev = next;
    }
    println!();
    let queues = prev.into_iter().collect::<Vec<_>>();
    assert!(queues.len()==1);
    let mut map = HashSet::new();
    for (_b, q) in queues{
        for q in q.into_values(){
            map.extend(q);
        }
    }
    println!("chance queues: {}/{total_queues}", map.len());
    println!("{}%", map.len() as f32/total_queues as f32 * 100.0);
    
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