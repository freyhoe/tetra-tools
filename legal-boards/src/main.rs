use std::{time::Instant, str::FromStr};
use legal_boards::create_gigapan;
use srs_4l::gameplay::Board;

fn main() -> std::io::Result<()> {

    let args : Vec<_>= std::env::args().collect();

    let board = Board(decode_fumen(&args[1]).expect("valid fumen"));
    let queue = legal_boards::queue::QueueGenerator::from_str(&args[2]).expect("valid queue");
    
    let giga = legal_boards::read_gigapan("./gigapan_shards").unwrap_or_else(|_|{
        println!("you do not have gigapan already build, creating gigapan...");
        create_gigapan().unwrap();
        legal_boards::read_gigapan("./gigapan_shards").unwrap()
    }).freeze();

    println!("giga loaded: {}",giga.len());



    println!("running:{board} {},\n total queues: {}", queue.string.trim_end_matches(','), queue.total_queues);
    let instant = Instant::now();
    
    legal_boards::calculate::chance(giga, board, &queue.bags, queue.total_queues);
    
    println!("{}ms", instant.elapsed().as_millis());
    Ok(())
}


pub fn decode_fumen(encoded: &str) -> Option<u64> {

        use fumen::{CellColor, Fumen, Page};

        let fumen = Fumen::decode(encoded).ok()?;
        let page: &Page = fumen.pages.get(0)?;

        if page.field[4..] != [[CellColor::Empty; 10]; 19]
            || page.garbage_row != [CellColor::Empty; 10]
        {
            return None;
        }

        let mut field = 0;
        for idx in 0..40 {
            let cell: CellColor = page.field[idx / 10][idx % 10];
            let filled = cell != CellColor::Empty;
            field |= (filled as u64) << idx;
        }

        Some(field)

}