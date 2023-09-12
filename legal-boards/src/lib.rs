pub mod boardgraph;
pub mod calculate;
pub mod queue;
mod big_queue;

use std::error::Error;
use std::{fs::OpenOptions, io::{BufWriter, BufReader}};
use std::time::Instant;

use rayon::prelude::*;

use boardgraph::Gigapan;

pub fn create_gigapan() -> std::io::Result<()> {
    std::fs::DirBuilder::new().recursive(true).create("./gigapan_shards")?;

    let instant = Instant::now();
    let (gigapan, reversepan) = boardgraph::compute_gigapan();
    println!("generated gigapan in {}s", instant.elapsed().as_secs());

    write_pan("gigapan_shards", gigapan)?;
    write_pan("reverse_gigapan_shards", reversepan)?;

    Ok(())
}
//impl Iterator<Item=(usize, impl Iterator<Item = (Board, SmallVec<[SmallVec<[Board;6]>;7]>)>)>
fn write_pan(path:&str, mut gigapan: Gigapan) -> std::io::Result<()>{
    let instant = Instant::now();
    let gigalen = gigapan.len();
    let shards : Vec<_> = gigapan.into_par_iter().collect();
    println!("sharded gigapan of length {} in {}ms", gigalen, instant.elapsed().as_millis());

    let instant = Instant::now();
    print!("writing shards of length: ");
    for (shard, chunk) in shards.chunks(gigalen/10).enumerate(){
        print!("{} ", chunk.len());
        let file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(format!("./{path}/{shard}.leb128")).unwrap();
        let writer = BufWriter::new(file);
        srs_4l::board_list::write_graph(chunk, writer)?;
    }
    println!();

    println!("wrote gigapan in {}s", instant.elapsed().as_secs());
    Ok(())
}

pub fn read_gigapan(path: &str) -> std::result::Result<Gigapan, Box<dyn Error>>{
    let gigapan = Gigapan::new();
    let paths : Vec<_> = std::fs::read_dir(path)?.filter_map(|entry|{
        if let Ok(entry) = entry{
            Some(entry.path())
        }else{
            None
        }
    }).collect();
    paths.par_iter().for_each(|path|{
        let file = OpenOptions::new().read(true).open(path).unwrap();
        let reader = BufReader::new(file);
        let shard = srs_4l::board_list::read_graph(reader).unwrap();
        for (k,v) in shard{
            gigapan.insert(k, v);
        }
    });
    Ok(gigapan)
}