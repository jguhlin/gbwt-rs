use super::*;

use simple_sds::serialize;

use std::collections::HashSet;

//-----------------------------------------------------------------------------

// GBWT example from the paper: (edges, runs, invalid_node)
fn get_edges_runs() -> (Vec<Vec<Pos>>, Vec<Vec<Run>>, usize) {
    let edges = vec![
        vec![Pos::new(1, 0)],
        vec![Pos::new(2, 0), Pos::new(3, 0)],
        vec![Pos::new(4, 0), Pos::new(5, 0)],
        vec![Pos::new(4, 1)],
        vec![Pos::new(5, 1), Pos::new(6, 0)],
        vec![Pos::new(7, 0)],
        vec![Pos::new(7, 2)],
        vec![Pos::new(0, 0)],
    ];
    let runs = vec![
        vec![Run::new(0, 3)],
        vec![Run::new(0, 2), Run::new(1, 1)],
        vec![Run::new(0, 1), Run::new(1, 1)],
        vec![Run::new(0, 1)],
        vec![Run::new(1, 1), Run::new(0, 1)],
        vec![Run::new(0, 2)],
        vec![Run::new(0, 1)],
        vec![Run::new(0, 3)],
    ];
    (edges, runs, 8)
}

// Bidirectional version of the example: (edges, runs, invalid_node)
fn get_bidirectional() -> (Vec<Vec<Pos>>, Vec<Vec<Run>>, usize) {
    let edges = vec![
        // ENDMARKER
        vec![Pos::new(2, 0), Pos::new(15, 0)],
        // 1
        vec![Pos::new(4, 0), Pos::new(6, 0)],
        vec![Pos::new(0, 0)],
        // 2
        vec![Pos::new(8, 0), Pos::new(10, 0)],
        vec![Pos::new(3, 0)],
        // 3
        vec![Pos::new(8, 1)],
        vec![Pos::new(3, 2)],
        // 4
        vec![Pos::new(10, 1), Pos::new(12, 0)],
        vec![Pos::new(5, 0), Pos::new(7, 0)],
        // 5
        vec![Pos::new(14, 0)],
        vec![Pos::new(5, 1), Pos::new(9, 0)],
        // 6
        vec![Pos::new(14, 2)],
        vec![Pos::new(9, 1)],
        // 7
        vec![Pos::new(0, 0)],
        vec![Pos::new(11, 0), Pos::new(13, 0)],
    ];
    let runs = vec![
        // ENDMARKER
        vec![Run::new(0, 3), Run::new(1, 3)],
        // 1
        vec![Run::new(0, 2), Run::new(1, 1)],
        vec![Run::new(0, 3)],
        // 2
        vec![Run::new(0, 1), Run::new(1, 1)],
        vec![Run::new(0, 2)],
        // 3
        vec![Run::new(0, 1)],
        vec![Run::new(0, 1)],
        // 4
        vec![Run::new(1, 1), Run::new(0, 1)],
        vec![Run::new(1, 1), Run::new(0, 1)],
        // 5
        vec![Run::new(0, 2)],
        vec![Run::new(0, 1), Run::new(1, 1)],
        // 6
        vec![Run::new(0, 1)],
        vec![Run::new(0, 1)],
        // 7
        vec![Run::new(0, 3)],
        vec![Run::new(1, 1), Run::new(0, 2)],
    ];
    (edges, runs, 16)
}

fn create_bwt(edges: &[Vec<Pos>], runs: &[Vec<Run>]) -> BWT {
    let mut builder = BWTBuilder::new();
    assert_eq!(builder.len(), 0, "Newly created builder has non-zero length");
    assert!(builder.is_empty(), "Newly created builder is not empty");

    for i in 0..edges.len() {
        builder.append(&edges[i], &runs[i]);
    }
    assert_eq!(builder.len(), edges.len(), "Invalid number of records in the builder");
    assert_eq!(builder.is_empty(), edges.is_empty(), "Invalid builder emptiness");

    BWT::from(builder)
}

// Check records in the BWT, using the provided edges as the source of truth.
// Also checks that `id()` works correctly.
fn check_records(bwt: &BWT, edges: &[Vec<Pos>]) {
    assert_eq!(bwt.len(), edges.len(), "Invalid number of records in the BWT");
    assert_eq!(bwt.is_empty(), edges.is_empty(), "Invalid BWT emptiness");

    // Edges.
    for i in 0..bwt.len() {
        let record = bwt.record(i);
        let curr_edges = &edges[i];
        assert_eq!(record.is_none(), curr_edges.is_empty(), "Invalid record {} existence", i);
        if let Some(record) = record {
            assert_eq!(record.id(), i, "Invalid id for record {}", i);
            assert_eq!(record.outdegree(), curr_edges.len(), "Invalid outdegree in record {}", i);
            for j in 0..record.outdegree() {
                assert_eq!(record.successor(j), curr_edges[j].node, "Invalid successor {} in record {}", j, i);
                assert_eq!(record.offset(j), curr_edges[j].offset, "Invalid offset {} in record {}", j, i);
            }
        }

        // Compressed record.
        let compressed = bwt.compressed_record(i);
        assert_eq!(compressed.is_none(), curr_edges.is_empty(), "Invalid compressed record {} existence", i);
        if let Some((edge_bytes, bwt_bytes)) = compressed {
            let decompressed = Record::decompress_edges(edge_bytes);
            assert!(decompressed.is_some(), "Could not decompress edges for record {}", i);
            let (edges, offset) = decompressed.unwrap();
            assert_eq!(offset, edge_bytes.len(), "Invalid offset after edge list for record {}", i);
            assert_eq!(&edges, curr_edges, "Invalid edges in compressed record {}", i);
            let record = bwt.record(i).unwrap();
            assert_eq!(bwt_bytes, record.bwt, "Invalid BWT in compressed record {}", i);
        }
    }
}

// Check that the iterator finds the correct records and that the id iterator finds the same ids.
fn check_iter(bwt: &BWT) {
    let mut iter = bwt.iter();
    let mut id_iter = bwt.id_iter();
    for i in 0..bwt.len() {
        if let Some(truth) = bwt.record(i) {
            if let Some(record) = iter.next() {
                assert_eq!(record.id(), truth.id(), "Invalid record id from the iterator");
            } else {
                panic!("Iterator did not find record {}", i);
            }
            assert_eq!(id_iter.next(), Some(truth.id()), "Invalid id from id iterator");
        }
    }
    assert!(iter.next().is_none(), "Iterator found a record past the end");
    assert!(id_iter.next().is_none(), "Id iterator found a record past the end");
}

// Check all `lf()` results in the BWT, using the provided edges and runs as the source of truth.
// Then check that decompressing the record works correctly.
// Also checks that `offset_to()` works in positive cases and that `len()` is correct.
fn check_lf(bwt: &BWT, edges: &[Vec<Pos>], runs: &[Vec<Run>]) {
    // `lf()` at each offset of each record.
    for i in 0..bwt.len() {
        if let Some(record) = bwt.record(i) {
            let mut offset = 0;
            let mut curr_edges = edges[i].clone();
            let curr_runs = &runs[i];
            let decompressed = record.decompress();
            assert_eq!(decompressed.len(), record.len(), "Invalid decompressed record {} length", i);
            for run in curr_runs {
                for _ in 0..run.len {
                    let edge = curr_edges[run.value];
                    let expected = if edge.node == ENDMARKER { None } else { Some(edge) };
                    assert_eq!(record.lf(offset), expected, "Invalid lf({}) in record {}", offset, i);
                    assert_eq!(decompressed[offset], edge, "Invalid decompressed lf({}) in record {}", offset, i);
                    let expected = if edge.node == ENDMARKER { None } else { Some(offset) };
                    assert_eq!(record.offset_to(edge), expected, "Invalid offset_to(({}, {})) in record {}", edge.node, edge.offset, i);
                    offset += 1;
                    curr_edges[run.value].offset += 1;
                }
            }
            assert_eq!(record.len(), offset, "Invalid record {} length", i);
            assert_eq!(record.lf(offset), None, "Got an lf() result past the end in record {}", i);
        }
    }
}

// Check all `follow()` results in the BWT, using `lf()` as the source of truth.
// Also checks that `bd_follow()` returns the same ranges.
// The tests for bidirectional search in `GBWT` make sure that the second return values are correct.
fn check_follow(bwt: &BWT, invalid_node: usize) {
    for record in bwt.iter() {
        let i = record.id();
        // Check all ranges, including empty and past-the-end ones.
        let len = record.len();
        for start in 0..len + 1 {
            for limit in start..len + 1 {
                // With an endmarker.
                assert_eq!(record.follow(start..limit, ENDMARKER), None, "Got a follow({}..{}, endmarker) result in record {}", start, limit, i);
                assert_eq!(record.bd_follow(start..limit, ENDMARKER), None, "Got a bd_follow({}..{}, endmarker) result in record {}", start, limit, i);

                // With each successor node.
                for rank in 0..record.outdegree() {
                    let successor = record.successor(rank);
                    if successor == ENDMARKER {
                        continue;
                    }
                    if let Some(result) = record.follow(start..limit, successor) {
                        let mut found = result.start..result.start;
                        for j in start..limit {
                            if let Some(pos) = record.lf(j) {
                                if pos.node == successor && pos.offset == found.end {
                                    found.end += 1;
                                }
                            }
                        }
                        assert_eq!(result, found, "follow({}..{}, {}) did not find the correct range in record {}", start, limit, successor, i);
                        if let Some((bd_result, _)) =  record.bd_follow(start..limit, successor) {
                            assert_eq!(bd_result, result, "bd_follow({}..{}, {}) did not find the same range as follow() in record {}", start, limit, successor, i);
                        } else {
                            panic!("bd_follow({}..{}, {}) did not find a result in record {}", start, limit, successor, i);
                        }
                    } else {
                        for j in start..limit {
                            if let Some(pos) = record.lf(j) {
                                assert_ne!(pos.node, successor, "follow({}..{}, {}) did not follow offset {} in record {}", start, limit, successor, j, i);
                            }
                            assert_eq!(record.bd_follow(start..limit, successor), None, "Got a bd_follow({}..{}, {}) result in record {}", start, limit, successor, i);
                        }
                    }
                }

                // With an invalid node.
                assert_eq!(record.follow(start..limit, invalid_node), None, "Got a follow({}..{}, invalid) result in record {}", start, limit, i);
                assert_eq!(record.bd_follow(start..limit, invalid_node), None, "Got a bd_follow({}..{}, invalid) result in record {}", start, limit, i);
            }
        }
    }
}

// Check negative cases for `offset_to()`.
fn negative_offset_to(bwt: &BWT, invalid_node: usize) {
    for record in bwt.iter() {
        assert_eq!(record.offset_to(Pos::new(ENDMARKER, 0)), None, "Got an offset to the endmarker from record {}", record.id());
        assert_eq!(record.offset_to(Pos::new(invalid_node, 0)), None, "Got an offset to an invalid node from record {}", record.id());
        for rank in 0..record.outdegree() {
            let successor = record.successor(rank);
            if successor == ENDMARKER {
                continue;
            }
            let offset = record.offset(rank);
            if offset > 0 {
                assert_eq!(record.offset_to(Pos::new(successor, offset - 1)), None, "Got an offset from record {} to a too small position in {}", record.id(), successor);
            }
            let count = record.follow(0..record.len(), successor).unwrap().len();
            assert_eq!(record.offset_to(Pos::new(successor, offset + count)), None, "Got an offset from record {} to a too large position in {}", record.id(), successor);
        }
    }
}

// Check that we can find predecessors for all positions except starting positions.
// The tests for `GBWT::backward()` will make sure that the predecessors are correct.
fn check_predecessor_at(bwt: &BWT) {
    let mut starting_positions = HashSet::<Pos>::new();
    let endmarker = bwt.record(ENDMARKER).unwrap();
    for i in 0..endmarker.len() {
        starting_positions.insert(endmarker.lf(i).unwrap());
    }

    for record in bwt.iter() {
        if record.id() == ENDMARKER {
            continue;
        }
        let reverse_id = ((record.id() + 1) ^ 1) - 1; // Record to node, flip, node to record.
        let reverse_record = bwt.record(reverse_id).unwrap();
        for i in 0..record.len() {
            if starting_positions.contains(&Pos::new(record.id() + 1, i)) {
                assert!(reverse_record.predecessor_at(i).is_none(), "Found a predecessor for a starting position ({}, {})", record.id() + 1, i);
            } else {
                assert!(reverse_record.predecessor_at(i).is_some(), "Did not find a predecessor for position ({}, {})", record.id() + 1, i);
            }
        }
        assert!(reverse_record.predecessor_at(record.len()).is_none(), "Found a predecessor for an invalid offset at node {}", record.id() + 1);
    }
}

//-----------------------------------------------------------------------------

#[test]
fn empty_bwt() {
    let edges = Vec::new();
    let runs = Vec::new();
    let invalid_node = 0;
    let bwt = create_bwt(&edges, &runs);
    check_records(&bwt, &edges);
    check_iter(&bwt);
    check_lf(&bwt, &edges, &runs);
    check_follow(&bwt, invalid_node);
    negative_offset_to(&bwt, invalid_node);
    serialize::test(&bwt, "empty-bwt", None, true);
}

#[test]
fn non_empty_bwt() {
    let (edges, runs, invalid_node) = get_edges_runs();
    let bwt = create_bwt(&edges, &runs);
    check_records(&bwt, &edges);
    check_iter(&bwt);
    check_lf(&bwt, &edges, &runs);
    check_follow(&bwt, invalid_node);
    negative_offset_to(&bwt, invalid_node);
    serialize::test(&bwt, "non-empty-bwt", None, true);
}

#[test]
fn empty_records() {
    let (mut edges, mut runs, invalid_node) = get_edges_runs();
    edges[2] = Vec::new();
    edges[6] = Vec::new();
    runs[2] = Vec::new();
    runs[6] = Vec::new();
 
    let bwt = create_bwt(&edges, &runs);
    check_records(&bwt, &edges);
    check_iter(&bwt);
    check_lf(&bwt, &edges, &runs);
    check_follow(&bwt, invalid_node);
    negative_offset_to(&bwt, invalid_node);
    serialize::test(&bwt, "bwt-with-empty", None, true);
}

#[test]
fn bidirectional_bwt() {
    let (edges, runs, invalid_node) = get_bidirectional();
    let bwt = create_bwt(&edges, &runs);
    check_records(&bwt, &edges);
    check_iter(&bwt);
    check_lf(&bwt, &edges, &runs);
    check_follow(&bwt, invalid_node);
    negative_offset_to(&bwt, invalid_node);
    check_predecessor_at(&bwt);
    serialize::test(&bwt, "bidirectional-bwt", None, true);
}

//-----------------------------------------------------------------------------
