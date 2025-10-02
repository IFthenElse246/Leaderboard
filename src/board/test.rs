use std::{env, time::Instant};
use rand::distr::{Distribution, Uniform};

use crate::board::Board;

#[test]
fn test_massive() {
    // unsafe {
    //     // env::set_var("RUST_BACKTRACE", "1");
    // }

    let mut board: Board<i64, f64> = Board::new();

    let size = 100000;
    let usize_size: usize = 100000;

    println!("Capping entries...");
    board.set_size_cap(usize_size);

    println!("Populating with dummy entries...");
    for i in 0..size {
        let _ = board.update_entry(i, i as f64);
    }

    println!("Performing update test...");
    let mut num_writes = 0;
    let mut rng = rand::rng();
    let id_range = Uniform::try_from(i64::MIN..=i64::MAX).unwrap();
    let val_range = Uniform::try_from(0..size).unwrap();

    loop {
        if num_writes >= 100000 {
            break;
        }

        let target_id = id_range.sample(&mut rng);
        let target_value = val_range.sample(&mut rng);
        let _ = board.update_entry(target_id, target_value as f64);

        num_writes += 1;
    }

    println!(
        "Made {num_writes} update operations.",
    );

    println!("Preparing...");

    let ids: Vec<i64> = board.get_ids();
    let ind_range = Uniform::try_from(0..ids.len()).unwrap();

    println!("Performing retrieval around test...");

    let mut num_reads = 0;
    loop {
        if num_reads >= 100000 {
            break;
        }

        let target_id = ids.get(ind_range.sample(&mut rng)).unwrap();

        board.get_around(target_id, 25, 25);

        num_reads += 1;
    }

    println!(
        "Made {num_reads} retrieval around operations."
    );

    println!("Performing rank retrieval test...");

    let mut num_ranks = 0;

    loop {
        if num_ranks >= 100000 {
            break;
        }

        let target_id = ids.get(ind_range.sample(&mut rng)).unwrap();

        board.get_rank(target_id);

        num_ranks += 1;
    }

    println!(
        "Made {num_ranks} rank retrieval operations."
    );

    println!("Getting top 50 entries...");
    board.get_top(50);


    println!("Getting bottom 50 entries...");

    board.get_bottom(50);


    println!("Preparing to write to file...");

    let tree = board.get_tree_copy();
    tree.validate();

    // println!("Writing to file...");
    // start = Instant::now();

    // let temp_path = cmd_arc.saves_path.join(format!("{board_name}_saving.test"));

    // match File::create(&temp_path) {
    //     Ok(handle) => {
    //         let mut buf_writer = BufWriter::new(handle);

    //         if let Err(e) = bincode::encode_into_std_write(
    //             &tree,
    //             &mut buf_writer,
    //             bincode::config::standard(),
    //         ) {
    //             let _ = writeln!(
    //                 &mut io::stderr().lock(),
    //                 "Failed to serialize leaderboard:\n{e}"
    //             );
    //         }
    //     }
    //     Err(err) => {
    //         let _ = writeln!(
    //             &mut io::stderr().lock(),
    //             "Failed to open temp file to save board.\n{}",
    //             err
    //         );
    //     }
    // };

    // let write_file_time = start.elapsed().as_secs_f64();

    // println!("Reading from file...");
    // start = Instant::now();

    // match File::open(temp_path.clone()) {
    //     Err(e) => {
    //         let _ = writeln!(
    //             &mut io::stderr().lock(),
    //             "Failed to read file for leaderboard:\n{e}",
    //         );
    //     }
    //     Ok(save_file) => {
    //         let mut buf_reader = BufReader::new(save_file);

    //         match bincode::decode_from_std_read(
    //             &mut buf_reader,
    //             bincode::config::standard(),
    //         ) {
    //             Err(e) => {
    //                 let _ = writeln!(
    //                     &mut io::stderr().lock(),
    //                     "Failed to parse file for leaderboard:\n{e}",
    //                 );
    //             }
    //             Ok(tree) => {
    //                 let t: crate::board::Tree<crate::board::Entry<i64, f64>> = tree;
    //                 crate::board::Board::from_tree(t);
    //             }
    //         };
    //     }
    // };

    // let read_file_time = start.elapsed().as_secs_f64();

    // println!("Cleaning up...");

    // if temp_path.exists() {
    //     let _ = std::fs::remove_file(temp_path);
    // }

    // let mut binding = cmd_arc.boards.lock().unwrap();
    // let board = binding.get_mut(&interaction.user.board).unwrap();
    // board.clear();

    // let _ = drop(binding);
}