use std::io::{self, Read, Write};
use smallvec::SmallVec;
fn to_io_error(err: leb128::read::Error) -> io::Error {
    use leb128::read::Error;

    match err {
        Error::IoError(err) => err,
        Error::Overflow => io::Error::new(io::ErrorKind::InvalidData, err),
    }
}
use crate::gameplay::Board;

pub fn write(boards: &[Board], mut w: impl Write) -> io::Result<()> {
    leb128::write::unsigned(&mut w, boards.len() as u64)?;

    let mut current = 0;

    for &board in boards {
        let diff = board.0 - current;
        current = board.0;

        leb128::write::unsigned(&mut w, diff)?;
    }

    Ok(())
}

pub fn read(mut r: impl Read) -> io::Result<Vec<Board>> {

    let len = leb128::read::unsigned(&mut r).map_err(to_io_error)? as usize;

    let mut boards = Vec::new();
    let mut current = 0;

    for _ in 0..len {
        let diff = leb128::read::unsigned(&mut r).map_err(to_io_error)?;
        current += diff;
        boards.push(Board(current));
    }

    Ok(boards)
}


pub fn write_graph(nodes: &[(Board, [SmallVec<[Board;6]>;7])], mut w: impl Write) -> io::Result<()>{
    leb128::write::unsigned(&mut w, nodes.len() as u64)?;
    for (board, children) in nodes{
        leb128::write::unsigned(&mut w, board.0)?;
        assert!(children.len()==7);
        for boards in children{
            leb128::write::unsigned(&mut w, boards.len() as u64)?;
            for child in boards{
                leb128::write::unsigned(&mut w, child.0-board.0)?;
            }
        }
    }
    Ok(())
}

pub fn read_graph(mut r: impl Read)->io::Result<Vec<(Board, [SmallVec<[Board;6]>;7])>>{
    let len = leb128::read::unsigned(&mut r).map_err(to_io_error)? as usize;
    let mut gigavec = Vec::new();
    for _ in 0..len{
        let current = leb128::read::unsigned(&mut r).map_err(to_io_error).unwrap();
        let mut pieces: [SmallVec<[Board;6]>;7] = Default::default();
        for i in 0..7{
            let mut boards : SmallVec<[Board;6]> = SmallVec::new();
            let len = leb128::read::unsigned(&mut r).map_err(to_io_error).unwrap();
            for _ in 0..len{
                let new = current + leb128::read::unsigned(&mut r).map_err(to_io_error).unwrap();
                boards.push(Board(new));
            }
            pieces[i]=boards;
        }
        gigavec.push((Board(current), pieces));
    }
    Ok(gigavec)
}