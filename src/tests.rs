use std::collections::HashMap;

use rand::{
    rngs::StdRng,
    SeedableRng,
};

use super::*;

/// Tests basic StreamingWswor functionality - feeds 4 items to a sampler
/// with capacity 3, verifies exactly 3 items are returned.
#[test]
fn test_streaming_wswor_basic() {
    let mut rng = StdRng::seed_from_u64(42);
    let mut sampler: StreamingWswor<f64, i32> = StreamingWswor::new(3);

    sampler.feed(1, 1.0, &mut rng).unwrap();
    sampler.feed(2, 2.0, &mut rng).unwrap();
    sampler.feed(3, 3.0, &mut rng).unwrap();
    sampler.feed(4, 4.0, &mut rng).unwrap();

    let results: Vec<_> = sampler.take().collect();
    assert_eq!(results.len(), 3);
}

/// Tests edge case where count=0 - feeds items but expects 0 results back.
/// Verifies the sampler respects the capacity limit even when items are
/// fed.
#[test]
fn test_streaming_wswor_zero_count() {
    let mut rng = StdRng::seed_from_u64(42);
    let mut sampler: StreamingWswor<f64, i32> = StreamingWswor::new(0);

    sampler.feed(1, 1.0, &mut rng).unwrap();
    sampler.feed(2, 2.0, &mut rng).unwrap();

    let results: Vec<_> = sampler.take().collect();
    assert_eq!(results.len(), 0);
}

/// Tests when fewer items are fed than the capacity - feeds 2 items to
/// capacity 5, expects all 2 back. Verifies the sampler doesn't require
/// full capacity to work.
#[test]
fn test_streaming_wswor_fewer_items_than_count() {
    let mut rng = StdRng::seed_from_u64(42);
    let mut sampler: StreamingWswor<f64, i32> = StreamingWswor::new(5);

    sampler.feed(1, 1.0, &mut rng).unwrap();
    sampler.feed(2, 2.0, &mut rng).unwrap();

    let results: Vec<_> = sampler.take().collect();
    assert_eq!(results.len(), 2);
    assert!(results.contains(&1));
    assert!(results.contains(&2));
}

/// Tests handling of zero weights - feeds item with weight 0.0 and weight
/// 1.0, both should be included. Verifies zero weights are handled without
/// errors.
#[test]
fn test_streaming_wswor_zero_weight() {
    let mut rng = StdRng::seed_from_u64(42);
    let mut sampler: StreamingWswor<f64, i32> = StreamingWswor::new(2);

    sampler.feed(1, 0.0, &mut rng).unwrap();
    sampler.feed(2, 1.0, &mut rng).unwrap();

    let results: Vec<_> = sampler.take().collect();
    assert_eq!(results.len(), 2);
}

/// Tests error handling for negative weights, NaN, and infinity - all
/// should return errors. Verifies input validation works correctly.
#[test]
fn test_streaming_wswor_invalid_weights() {
    let mut rng = StdRng::seed_from_u64(42);
    let mut sampler: StreamingWswor<f64, i32> = StreamingWswor::new(2);

    assert!(sampler.feed(1, -1.0, &mut rng).is_err());
    assert!(sampler.feed(2, f64::NAN, &mut rng).is_err());
    assert!(sampler.feed(3, f64::INFINITY, &mut rng).is_err());
}

/// Tests bulk feeding via iterator - feeds 5 items at once, expects 3 back.
/// Verifies the feed_iter convenience method works correctly.
#[test]
fn test_streaming_wswor_feed_iter() {
    let mut rng = StdRng::seed_from_u64(42);
    let mut sampler: StreamingWswor<f64, i32> = StreamingWswor::new(3);

    let items = vec![(1.0, 1), (2.0, 2), (3.0, 3), (4.0, 4), (5.0, 5)];
    sampler.feed_iter(items.into_iter(), &mut rng).unwrap();

    let results: Vec<_> = sampler.take().collect();
    assert_eq!(results.len(), 3);
}

/// Tests that feed_iter properly propagates weight validation errors.
/// Verifies error handling works in batch feeding scenarios.
#[test]
fn test_streaming_wswor_feed_iter_invalid_weight() {
    let mut rng = StdRng::seed_from_u64(42);
    let mut sampler: StreamingWswor<f64, i32> = StreamingWswor::new(3);

    let items = vec![(1.0, 1), (-1.0, 2), (3.0, 3)];
    assert!(sampler.feed_iter(items.into_iter(), &mut rng).is_err());
}

/// Tests the non-consuming iterator method vs the consuming take method.
/// Verifies both access patterns work correctly.
#[test]
fn test_streaming_wswor_iter() {
    let mut rng = StdRng::seed_from_u64(42);
    let mut sampler: StreamingWswor<f64, i32> = StreamingWswor::new(2);

    sampler.feed(1, 1.0, &mut rng).unwrap();
    sampler.feed(2, 2.0, &mut rng).unwrap();

    let iter_count = sampler.iter().count();
    assert_eq!(iter_count, 2);

    let results: Vec<_> = sampler.take().collect();
    assert_eq!(results.len(), 2);
}

/// Statistical test running 1000 trials to verify items with higher weights
/// (9.0 vs 1.0) are selected more frequently.
#[test]
fn test_streaming_wswor_weighted_distribution() {
    let mut rng = StdRng::seed_from_u64(42);
    let mut counts = HashMap::new();

    for _ in 0 .. 1000 {
        let mut sampler: StreamingWswor<f64, i32> = StreamingWswor::new(1);
        sampler.feed(1, 1.0, &mut rng).unwrap();
        sampler.feed(2, 9.0, &mut rng).unwrap(); // 9x more likely

        let result = sampler.take().next().unwrap();
        *counts.entry(result).or_insert(0) += 1;
    }

    let count_1 = counts.get(&1).unwrap_or(&0);
    let count_2 = counts.get(&2).unwrap_or(&0);

    assert!(*count_2 > *count_1 * 3); // Should be roughly 9:1 ratio
}

/// Tests basic single-item sampling - feeds 3 items, verifies one result
/// via both get() and take().
#[test]
fn test_single_streaming_ws_basic() {
    let mut rng = StdRng::seed_from_u64(42);
    let mut sampler: SingleStreamingWs<f64, i32> = SingleStreamingWs::new();

    sampler.feed(1, 1.0, &mut rng).unwrap();
    sampler.feed(2, 2.0, &mut rng).unwrap();
    sampler.feed(3, 3.0, &mut rng).unwrap();

    let result = sampler.get();
    assert!(result.is_some());

    let taken = sampler.take();
    assert!(taken.is_some());
}

/// Tests empty sampler behavior - both get() and take() should return None.
/// Verifies proper handling of uninitialized state.
#[test]
fn test_single_streaming_ws_empty() {
    let sampler: SingleStreamingWs<f64, i32> = SingleStreamingWs::new();

    assert!(sampler.get().is_none());
    assert!(sampler.take().is_none());
}

/// Tests error handling for invalid weights in single sampler. Verifies
/// input validation works for single-item sampling.
#[test]
fn test_single_streaming_ws_invalid_weights() {
    let mut rng = StdRng::seed_from_u64(42);
    let mut sampler: SingleStreamingWs<f64, i32> = SingleStreamingWs::new();

    assert!(sampler.feed(1, -1.0, &mut rng).is_err());
    assert!(sampler.feed(2, f64::NAN, &mut rng).is_err());
    assert!(sampler.feed(3, f64::INFINITY, &mut rng).is_err());
}

/// Tests bulk feeding for single sampler. Verifies feed_iter works
/// correctly for single-item sampling scenarios.
#[test]
fn test_single_streaming_ws_feed_iter() {
    let mut rng = StdRng::seed_from_u64(42);
    let mut sampler: SingleStreamingWs<f64, i32> = SingleStreamingWs::new();

    let items = vec![(1.0, 1), (2.0, 2), (3.0, 3)];
    sampler.feed_iter(items.into_iter(), &mut rng).unwrap();

    assert!(sampler.get().is_some());
}

/// Tests error propagation in single sampler's feed_iter. Verifies batch
/// feeding error handling for single-item sampling.
#[test]
fn test_single_streaming_ws_feed_iter_invalid_weight() {
    let mut rng = StdRng::seed_from_u64(42);
    let mut sampler: SingleStreamingWs<f64, i32> = SingleStreamingWs::new();

    let items = vec![(1.0, 1), (-1.0, 2)];
    assert!(sampler.feed_iter(items.into_iter(), &mut rng).is_err());
}

/// Statistical test for single sampler - verifies higher-weighted items are
/// selected more often over 1000 trials. Basic weighted selection
/// verification.
#[test]
fn test_single_streaming_ws_weighted_distribution() {
    let mut rng = StdRng::seed_from_u64(42);
    let mut counts = HashMap::new();

    for _ in 0 .. 1000 {
        let mut sampler: SingleStreamingWs<f64, i32> =
            SingleStreamingWs::new();
        sampler.feed(1, 1.0, &mut rng).unwrap();
        sampler.feed(2, 9.0, &mut rng).unwrap(); // 9x more likely

        let result = sampler.take().unwrap();
        *counts.entry(result).or_insert(0) += 1;
    }

    let count_1 = counts.get(&1).unwrap_or(&0);
    let count_2 = counts.get(&2).unwrap_or(&0);

    dbg!(*count_2);
    dbg!(*count_1 * 3);
    assert!(*count_2 > *count_1 * 3); // Should be roughly 9:1 ratio
}

/// Tests the standalone wswor convenience function that wraps
/// StreamingWswor. Verifies the high-level API works correctly.
#[test]
fn test_wswor_convenience_function() {
    let mut rng = StdRng::seed_from_u64(42);
    let items = vec![(1.0, 1), (2.0, 2), (3.0, 3), (4.0, 4)];

    let results: Result<Vec<_>, _> =
        wswor(items.into_iter(), &mut rng, 2).map(|iter| iter.collect());

    assert!(results.is_ok());
    let results = results.unwrap();
    assert_eq!(results.len(), 2);
}

/// Tests error handling in the convenience function. Verifies error
/// propagation through the high-level API.
#[test]
fn test_wswor_convenience_function_invalid_weights() {
    let mut rng = StdRng::seed_from_u64(42);
    let items = vec![(1.0, 1), (-1.0, 2)];

    let results = wswor(items.into_iter(), &mut rng, 1);
    assert!(results.is_err());
}

/// Tests the weight validation logic directly. Verifies that positive
/// weights (including zero) are accepted while negative, NaN, and infinite
/// weights are rejected.
#[test]
fn test_has_invalid_weights_check() {
    assert!(HasInvalidWeights::check_weight(&1.0f64).is_ok());
    assert!(HasInvalidWeights::check_weight(&0.0f64).is_ok());
    assert!(HasInvalidWeights::check_weight(&(-1.0f64)).is_err());
    assert!(HasInvalidWeights::check_weight(&f64::NAN).is_err());
    assert!(HasInvalidWeights::check_weight(&f64::INFINITY).is_err());
    assert!(HasInvalidWeights::check_weight(&f64::NEG_INFINITY).is_err());
}

/// Tests that items are selected proportionally to their weights (1:2:3:4
/// ratio) over 10,000 trials. Uses a formal chi-square statistical test to
/// verify observed frequencies match expected probabilities within
/// acceptable bounds. This is the core correctness test for weighted
/// sampling.
#[test]
fn test_single_streaming_proportionality() {
    let mut rng = StdRng::seed_from_u64(12345);
    let mut counts = HashMap::new();
    let trials = 10000;

    // Items with weights 1:2:3:4 (total weight = 10)
    let items = vec![(1.0, 'A'), (2.0, 'B'), (3.0, 'C'), (4.0, 'D')];
    let expected_probs = vec![0.1, 0.2, 0.3, 0.4];

    for _ in 0 .. trials {
        let mut sampler: SingleStreamingWs<f64, char> =
            SingleStreamingWs::new();
        for (weight, value) in items.iter() {
            sampler.feed(*value, *weight, &mut rng).unwrap();
        }
        let result = sampler.take().unwrap();
        *counts.entry(result).or_insert(0) += 1;
    }

    // Chi-square test
    let mut chi_square = 0.0;
    for (i, &item) in ['A', 'B', 'C', 'D'].iter().enumerate() {
        let observed = *counts.get(&item).unwrap_or(&0) as f64;
        let expected = trials as f64 * expected_probs[i];
        chi_square += (observed - expected).powi(2) / expected;
    }

    // Critical value for 3 degrees of freedom at 0.01 significance level
    assert!(
        chi_square < 11.345,
        "Chi-square value {} exceeds critical value",
        chi_square
    );
}

/// Tests the algorithm's handling of very unequal weights (1 vs 1000).
/// Verifies that massively higher-weighted items dominate selection (>90%
/// of the time) and maintain extreme ratios (>100:1). This ensures the
/// algorithm doesn't break down with large weight differences.
#[test]
fn test_single_streaming_extreme_weights() {
    let mut rng = StdRng::seed_from_u64(54321);
    let mut counts = HashMap::new();
    let trials = 5000;

    for _ in 0 .. trials {
        let mut sampler: SingleStreamingWs<f64, char> =
            SingleStreamingWs::new();
        sampler.feed('A', 1.0, &mut rng).unwrap();
        sampler.feed('B', 1000.0, &mut rng).unwrap(); // 1000x more likely

        let result = sampler.take().unwrap();
        *counts.entry(result).or_insert(0) += 1;
    }

    let count_a = *counts.get(&'A').unwrap_or(&0);
    let count_b = *counts.get(&'B').unwrap_or(&0);

    // B should be selected much more often (at least 90% of the time)
    assert!(count_b as f64 / trials as f64 > 0.9);
    assert!(count_b > count_a * 100);
}

/// Tests that when all items have equal weights, they're selected roughly
/// equally. Verifies the algorithm doesn't introduce bias when there's no
/// weight difference and acts as a baseline uniformity check.
#[test]
fn test_single_streaming_uniform_distribution() {
    let mut rng = StdRng::seed_from_u64(98765);
    let mut counts = HashMap::new();
    let trials = 8000;

    for _ in 0 .. trials {
        let mut sampler: SingleStreamingWs<f64, i32> =
            SingleStreamingWs::new();
        for i in 1 ..= 8 {
            sampler.feed(i, 1.0, &mut rng).unwrap(); // All equal weights
        }

        let result = sampler.take().unwrap();
        *counts.entry(result).or_insert(0) += 1;
    }

    // Each item should appear roughly 1/8 of the time (1000 ± tolerance)
    let expected = trials / 8;
    for i in 1 ..= 8 {
        let count = *counts.get(&i).unwrap_or(&0);
        let deviation = (count as i32 - expected as i32).abs();
        assert!(
            deviation < 200,
            "Item {} count {} deviates too much from expected {}",
            i,
            count,
            expected
        );
    }
}

/// Tests mixed scenarios with zero-weight and positive-weight items.
/// Verifies zero-weight items are almost never selected (<1% each), while
/// positive-weight items maintain their proper 2:1 ratio.
#[test]
fn test_single_streaming_zero_weight_mixed() {
    let mut rng = StdRng::seed_from_u64(13579);
    let mut counts = HashMap::new();
    let trials = 3000;

    for _ in 0 .. trials {
        let mut sampler: SingleStreamingWs<f64, char> =
            SingleStreamingWs::new();
        sampler.feed('A', 0.0, &mut rng).unwrap(); // Zero weight
        sampler.feed('B', 0.0, &mut rng).unwrap(); // Zero weight
        sampler.feed('C', 1.0, &mut rng).unwrap(); // Positive weight
        sampler.feed('D', 2.0, &mut rng).unwrap(); // Positive weight

        let result = sampler.take().unwrap();
        *counts.entry(result).or_insert(0) += 1;
    }

    // Zero-weight items should almost never be selected
    let count_a = *counts.get(&'A').unwrap_or(&0);
    let count_b = *counts.get(&'B').unwrap_or(&0);
    let count_c = *counts.get(&'C').unwrap_or(&0);
    let count_d = *counts.get(&'D').unwrap_or(&0);

    // Zero-weight items should be selected very rarely (less than 1% each)
    assert!(count_a < trials / 100);
    assert!(count_b < trials / 100);

    // D should be selected twice as often as C
    assert!((count_d as f64) / (count_c as f64) > 1.5);
    assert!((count_d as f64) / (count_c as f64) < 2.5);
}

/// Tests that in reservoir sampling (k=3 from 5 items with weights
/// 1:2:3:4:5), higher-weight items appear more frequently than lower-weight
/// ones. Verifies the ordering E>D>C>B>A and that the highest-weight item
/// appears in most samples.
#[test]
fn test_streaming_wswor_proportionality() {
    let mut rng = StdRng::seed_from_u64(24680);
    let mut counts = HashMap::new();
    let trials = 5000;

    // Items with weights 1:2:3:4:5 (total weight = 15)
    let items =
        vec![(1.0, 'A'), (2.0, 'B'), (3.0, 'C'), (4.0, 'D'), (5.0, 'E')];

    for _ in 0 .. trials {
        let mut sampler: StreamingWswor<f64, char> = StreamingWswor::new(3);
        for (weight, value) in items.iter() {
            sampler.feed(*value, *weight, &mut rng).unwrap();
        }

        for result in sampler.take() {
            *counts.entry(result).or_insert(0) += 1;
        }
    }

    // Higher weight items should appear more frequently
    let count_a = *counts.get(&'A').unwrap_or(&0);
    let count_b = *counts.get(&'B').unwrap_or(&0);
    let count_c = *counts.get(&'C').unwrap_or(&0);
    let count_d = *counts.get(&'D').unwrap_or(&0);
    let count_e = *counts.get(&'E').unwrap_or(&0);

    // E should be most frequent, A should be least frequent
    assert!(count_e > count_d);
    assert!(count_d > count_c);
    assert!(count_c > count_b);
    assert!(count_b > count_a);

    // E should appear in most samples (weight 5/15 ≈ 0.33, and we sample
    // 3/5 items)
    assert!(count_e as f64 / trials as f64 > 0.7);
}

/// Tests that weight proportions are maintained regardless of sample size
/// (k=1,2,3). For items with 2:1 weight ratio, verifies the selection ratio
/// stays around 2:1 across different sample sizes. This ensures the
/// algorithm's proportionality doesn't depend on how many items you're
/// sampling.
#[test]
fn test_streaming_wswor_sample_size_independence() {
    let mut rng = StdRng::seed_from_u64(11111);
    let items = vec![(1.0, 'A'), (2.0, 'B'), (3.0, 'C'), (4.0, 'D')];
    let trials = 2000;

    // Test different sample sizes
    for &sample_size in &[1, 2, 3] {
        let mut counts = HashMap::new();

        for _ in 0 .. trials {
            let mut sampler: StreamingWswor<f64, char> =
                StreamingWswor::new(sample_size);
            for (weight, value) in items.iter() {
                sampler.feed(*value, *weight, &mut rng).unwrap();
            }

            for result in sampler.take() {
                *counts.entry(result).or_insert(0) += 1;
            }
        }

        // Proportions should be maintained regardless of sample size
        let count_b = *counts.get(&'B').unwrap_or(&0) as f64;
        let count_a = *counts.get(&'A').unwrap_or(&0) as f64;
        let ratio = count_b / count_a;

        // B has 2x weight of A, so ratio should be around 2
        assert!(
            ratio > 1.5,
            "Sample size {} - ratio B/A = {} should be > 1.5",
            sample_size,
            ratio
        );
        assert!(
            ratio < 3.0,
            "Sample size {} - ratio B/A = {} should be < 3.0",
            sample_size,
            ratio
        );
    }
}

/// Tests reservoir sampling with equal weights - each item should appear in
/// roughly 50% of samples (3 out of 6 items selected). Verifies no bias is
/// introduced in uniform scenarios and that sampling frequency matches
/// mathematical expectation.
#[test]
fn test_streaming_wswor_uniform_distribution() {
    let mut rng = StdRng::seed_from_u64(22222);
    let mut counts = HashMap::new();
    let trials = 3000;
    let num_items = 6;
    let sample_size = 3;

    for _ in 0 .. trials {
        let mut sampler: StreamingWswor<f64, i32> =
            StreamingWswor::new(sample_size);
        for i in 1 ..= num_items {
            sampler.feed(i, 1.0, &mut rng).unwrap(); // All equal weights
        }

        for result in sampler.take() {
            *counts.entry(result).or_insert(0) += 1;
        }
    }

    // Each item should appear in roughly half the samples (3/6 = 0.5)
    let expected = (trials * sample_size) / (num_items as usize);
    for i in 1 ..= num_items {
        let count = *counts.get(&i).unwrap_or(&0);
        let deviation = (count as i32 - expected as i32).abs();
        assert!(
            deviation < 300,
            "Item {} count {} deviates too much from expected {}",
            i,
            count,
            expected
        );
    }
}

/// Tests with 50 items having linearly increasing weights (1,2,3...50),
/// sampling 10 items over 1000 trials. Verifies: (1) top 10 items selected
/// 2x more than bottom 10, (2) highest-weight items exceed average
/// frequency, (3) item 50 has maximum selection count. This tests
/// scalability and maintains proportionality in large populations.
#[test]
fn test_streaming_wswor_large_population() {
    let mut rng = StdRng::seed_from_u64(12345);
    let mut counts = HashMap::new();
    let trials = 1000;
    let population_size = 50;
    let sample_size = 10;

    for _ in 0 .. trials {
        let mut sampler: StreamingWswor<f64, i32> =
            StreamingWswor::new(sample_size);

        // Create items with linearly increasing weights (1, 2, 3, ..., 50)
        for i in 1 ..= population_size {
            sampler.feed(i, i as f64, &mut rng).unwrap();
        }

        for result in sampler.take() {
            *counts.entry(result).or_insert(0) += 1;
        }
    }

    // Higher-numbered (higher-weighted) items should appear more frequently
    let low_items_sum: usize =
        (1 ..= 10).map(|i| *counts.get(&i).unwrap_or(&0)).sum();
    let high_items_sum: usize =
        (41 ..= 50).map(|i| *counts.get(&i).unwrap_or(&0)).sum();

    // Top 10 items should be selected much more often than bottom 10 items
    assert!(
        high_items_sum > low_items_sum * 2,
        "High items sum: {}, Low items sum: {}",
        high_items_sum,
        low_items_sum
    );

    // Item 50 (highest weight) should appear frequently, but not
    // necessarily 80% because there are other high-weight items competing
    let count_50 = *counts.get(&50).unwrap_or(&0);
    let count_49 = *counts.get(&49).unwrap_or(&0);
    let count_48 = *counts.get(&48).unwrap_or(&0);

    // Top items should appear more often than average
    let average_count = (trials * sample_size) / (population_size as usize);
    assert!(
        count_50 > average_count,
        "Item 50 count {} should be > average = {}",
        count_50,
        average_count
    );
    assert!(
        count_49 > average_count / 2,
        "Item 49 count {} should be > average/2 = {}",
        count_49,
        average_count / 2
    );
    assert!(
        count_48 > average_count / 2,
        "Item 48 count {} should be > average/2 = {}",
        count_48,
        average_count / 2
    );

    // Item 50 should be selected most frequently among all items
    let max_count = (1 ..= population_size)
        .map(|i| *counts.get(&i).unwrap_or(&0))
        .max()
        .unwrap_or(0);
    assert_eq!(
        count_50, max_count,
        "Item 50 should have the highest count"
    );
}

/// Tests reservoir sampling with mixed zero and positive weights. Verifies
/// zero-weight items are rarely selected (<2% each), positive-weight items
/// maintain proper ordering (5>3>1), and high-weight items appear in most
/// samples. This tests edge case handling in reservoir sampling.
#[test]
fn test_streaming_wswor_zero_weight_mixed() {
    let mut rng = StdRng::seed_from_u64(44444);
    let mut counts = HashMap::new();
    let trials = 2000;

    for _ in 0 .. trials {
        let mut sampler: StreamingWswor<f64, char> = StreamingWswor::new(2);
        sampler.feed('A', 0.0, &mut rng).unwrap(); // Zero weight
        sampler.feed('B', 0.0, &mut rng).unwrap(); // Zero weight
        sampler.feed('C', 1.0, &mut rng).unwrap(); // Low weight
        sampler.feed('D', 5.0, &mut rng).unwrap(); // High weight
        sampler.feed('E', 3.0, &mut rng).unwrap(); // Medium weight

        for result in sampler.take() {
            *counts.entry(result).or_insert(0) += 1;
        }
    }

    let count_a = *counts.get(&'A').unwrap_or(&0);
    let count_b = *counts.get(&'B').unwrap_or(&0);
    let count_c = *counts.get(&'C').unwrap_or(&0);
    let count_d = *counts.get(&'D').unwrap_or(&0);
    let count_e = *counts.get(&'E').unwrap_or(&0);

    // Zero-weight items should be very rarely selected
    assert!(count_a < trials / 50);
    assert!(count_b < trials / 50);

    // D should be selected most often, then E, then C
    assert!(count_d > count_e);
    assert!(count_e > count_c);

    // D should appear in most samples (high weight + sample size 2)
    assert!(count_d as f64 / trials as f64 > 0.8);
}
