use std::time::Instant;

//use legal_boards::create_gigapan;
fn main() -> std::io::Result<()> {
    //create_gigapan()?;
    let giga = legal_boards::read_gigapan("./gigapan_shards").unwrap().freeze();
    println!("giga loaded: {}",giga.len());

    let board = srs_4l::gameplay::Board::from_str(
        "
        "
    );

    println!("benching:{board} *p7 *p4");
    let instant = Instant::now();
        legal_boards::calculate::chance(&giga, board);
    println!("{}ms", instant.elapsed().as_millis());
    Ok(())
}
