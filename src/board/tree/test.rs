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


    tree.validate();
    assert_eq!(tree.len(), 50);
    assert!(!tree.contains(&32));
    assert!(tree.contains(&72));
    
    tree.clear();
    assert!(tree.is_empty());

}