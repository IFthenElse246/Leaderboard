use std::env;

use super::*;

#[test]
fn test_simple() {
    unsafe {
        env::set_var("RUST_BACKTRACE", "1");
    }
    
    let mut tree: Tree<u32> = Tree::new();

    assert!(tree.is_empty());
    assert!(!tree.contains(&10));
    assert_eq!(tree.len(), 0);

    assert!(tree.insert(10));
    assert!(!tree.insert(10));
    assert!(tree.contains(&10));
    assert!(!tree.is_empty());
    assert_eq!(tree.len(), 1);

    assert!(tree.remove(&10).is_some());
    assert!(!tree.remove(&10).is_some());
    assert!(!tree.contains(&10));
    assert!(tree.is_empty());
}

#[test]
fn test_100() {
    unsafe {
        env::set_var("RUST_BACKTRACE", "1");
    }

    let mut tree: Tree<u32> = Tree::new();
    
    for i in 0..100 {
        assert!(tree.insert(i));
    };

    tree.validate();
    assert_eq!(tree.len(), 100);
    assert!(tree.contains(&72));
    assert!(!tree.contains(&100));

    for i in 0..100 {
        // print!("{}: {}\n", i, tree.index_of(i).0)
        assert!(tree.index_of(&i).0 == (99 - i as usize));
        assert!(tree.at_index((99 - i) as usize).is_some_and(|v| *v == i))
    };

    let mut cursor = tree.cursor();

    for i in 0..100 {
        cursor.move_next();
        assert!(!cursor.is_at_end());
        assert!(cursor.get_value().unwrap() == &i);
        assert!(cursor.get_index().unwrap() == (99 - i as usize));
    }
    cursor.move_next();
    assert!(cursor.is_at_end());

    for i in 0..100 {
        cursor.move_prev();
        assert!(!cursor.is_at_end());
        assert!(cursor.get_value().unwrap() == &(99 - i));
        assert!(cursor.get_index().unwrap() == (i as usize));
    }
    cursor.move_prev();
    assert!(cursor.is_at_end());

    for i in 0..25 {
        assert!(tree.remove(&i).is_some());
    };


    // for i in 0..25 {
    //     tree.validate();
    //     assert!(tree.remove(&(49 - i)).is_some());
    // };
    let mut cursor_mut = tree.seek_val_mut(&50).unwrap();
    for i in 0..25 {
        assert!(cursor_mut.delete_prev() == Some(49-i));
    }


    cursor_mut.get_tree().validate();
    assert_eq!(cursor_mut.get_tree().len(), 50);
    assert!(!cursor_mut.get_tree().contains(&32));
    assert!(cursor_mut.get_tree().contains(&72));

    assert_eq!(cursor_mut.replace(49), Some(50));

    tree.validate();
    assert!(tree.contains(&49));
    assert!(!tree.contains(&50));

    tree.remove(&52);

    cursor_mut = tree.seek_val_mut(&49).unwrap();
    cursor_mut.replace(52);

    tree.validate();
    assert!(!tree.contains(&49));
    assert!(tree.contains(&52));

    tree.remove(&4);

    cursor_mut = tree.seek_val_mut(&52).unwrap();
    cursor_mut.replace(4);

    tree.validate();
    assert!(!tree.contains(&52));
    assert!(tree.contains(&4));
    
    tree.clear();
    assert!(tree.is_empty());

}

// #[test]
// pub fn test_io() {
//     unsafe {
//         env::set_var("RUST_BACKTRACE", "1");
//         env::set_var("MIRIFLAGS", "-Zmiri-disable-isolation");
//     }

//     let mut tree: Tree<u32> = Tree::new();

//     for i in 1..10000000 {
//         tree.insert(i);
//     }

//     for i in 1..10000000 {
//         if i % 3 == 0 {
//             tree.remove(&i);
//         }
//     }

//     let mut file = File::create("test.tree").expect("Failed to create file");
//     let mut bufWriter = BufWriter::new(file);

//     if let Err(v) = bincode::encode_into_std_write(&tree, &mut bufWriter, bincode::config::standard()) {
//         panic!("Write Error: {:?}", v);
//     }

//     bufWriter.flush();
//     drop(bufWriter);

//     file = File::open("test.tree").expect("Failed to open file");
//     let mut bufReader = BufReader::new(file);

//     let tree2: Tree<u32> = match bincode::decode_from_std_read(&mut bufReader, bincode::config::standard()) {
//         Err(v) => {
//             panic!("Read Error: {:?}", v);
//         },
//         Ok(tree) => tree
//     };

//     tree2.validate();

//     // tree.print_pretty();
//     // tree2.print_pretty();

//     let mut cursor1 = tree.cursor();
//     let mut cursor2 = tree2.cursor();

//     cursor1.move_next();
//     cursor2.move_next();

//     while !cursor1.is_at_end() {
//         assert_eq!(cursor1.get_value(), cursor2.get_value(), "Values not equal, {:?} and {:?}", cursor1.get_value(), cursor2.get_value());
//         assert_eq!(cursor1.get_index(), cursor2.get_index(), "Indicies not equal, {:?} and {:?}", cursor1.get_index(), cursor2.get_index());
//         assert_eq!(cursor1.get_height(), cursor2.get_height(), "Heights not equal, {:?} and {:?}", cursor1.get_height(), cursor2.get_height());

//         cursor1.move_next();
//         cursor2.move_next();
//     }

//     if !cursor2.is_at_end() {
//         panic!("Tree2 was longer than Tree1!");
//     }

//     fs::remove_file("test.tree");
// }