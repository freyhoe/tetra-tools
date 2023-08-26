use legal_boards::create_gigapan;
fn main() -> std::io::Result<()> {
    create_gigapan()?;
    /*let giga = legal_boards::read_gigapan().unwrap();
    println!("giga loaded: {}",giga.len());

    let board = srs_4l::gameplay::Board::from_str(
        "
        GGG__GGGGG
        GGG__GGGGG
        GGG__GGGGG
        GGG__GGGGG
        "
    );
    println!("board: {board}, contained: {:?}", giga.get(&board));

    legal_boards::calculate::chance(giga, board);*/
    Ok(())
}
