use std::{time::Instant, str::FromStr};

//use legal_boards::create_gigapan;
fn main() -> std::io::Result<()> {
    //create_gigapan()?;
    let giga = legal_boards::read_gigapan("./gigapan_shards").unwrap().freeze();

    println!("giga loaded: {}",giga.len());

    /*let board = srs_4l::gameplay::Board::from_str(
        "
        
        "
    );*/
    let board = srs_4l::gameplay::Board(0);

    let queue = legal_boards::queue::QueueGenerator::from_str("[^JIT],*p7").unwrap();

    println!("benching:{board} {}", queue.string);
    let instant = Instant::now();
        legal_boards::calculate::chance(&giga, board, &queue.bags);
    println!("{}ms", instant.elapsed().as_millis());
    Ok(())
}
