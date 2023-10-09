
mod queue;
mod calculate;
use std::str::FromStr;
use srs_4l::gameplay::Board;

use clap::Parser;


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// SFinder queue input
    #[arg(short, long, default_value = "", required=false)]
    queue: String,

    /// How many pieces gigapan can be aware for certain of at any given state
    #[arg(short, long, default_value_t = 7)]
    see: u8,

    /// Fumen input of the board
    #[arg(short, long, default_value = "v115@vhAAgH")]
    fumen: String,

}

fn main() -> std::io::Result<()> {

    let args = Args::parse();
    if args.queue == ""{
        return legal_boards::create_gigapan();
    }

    let board = Board(decode_fumen(&args.fumen).expect("valid fumen"));
    let queue = queue::CombinatoricQueue::from_str(&args.queue).expect("valid queue");
    
    let giga = legal_boards::read_gigapan("./gigapan_shards").expect("unable to find gigapan shards! try with --generate").freeze();

    println!("giga loaded: {}",giga.len());

    println!("running:{board} {}", queue);
    calculate::limited_see_chance(&giga, board, 7, &queue, false);

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