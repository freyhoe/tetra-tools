use std::{time::Instant, str::FromStr};

fn main() -> std::io::Result<()> {
    let giga = legal_boards::read_gigapan("./gigapan_shards").unwrap().freeze();

    println!("giga loaded: {}",giga.len());

    let board = srs_4l::gameplay::Board::from_str(
        "
        GG________
        GG________
        GG________
        GG________
        "
    );

    let queue = legal_boards::queue::QueueGenerator::from_str("*p7*p2").unwrap();

    println!("running:{board} {},\n total queues: {}", queue.string.trim_end_matches(','), queue.total_queues);
    let instant = Instant::now();
    
    legal_boards::calculate::chance(giga, board, &queue.bags, queue.total_queues);
    
    println!("{}ms", instant.elapsed().as_millis());
    Ok(())
}
