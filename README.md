# WSWoR

An implementation of a streaming sampler based on
[Müller's (2016)](https://raw.githubusercontent.com/krlmlr/wrswoR/master/vignettes/internal/out/wrswoR.pdf)
paper on weighted random sampling without replacement.

This sampler can process arbitrarily large datasets through iterators without 
storing all items in memory - only the sample results are kept in memory.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
wswor = { git = "https://github.com/f5xs-0000a/wswor.git" }
```

### Quick Start

For simple one-shot sampling:

```rust
use wswor::wswor;
use rand::thread_rng;

let mut rng = thread_rng();
let items = vec![(1.0, "apple"), (2.0, "banana"), (3.0, "cherry")];

// sample two items without replacement
let results: Vec<_> = wswor(items.into_iter(), &mut rng, 2)?.collect();
println!("{:?}", results); // e.g., ["cherry", "banana"]
```

### Sampling from Manually Fed Values

```rust
use wswor::StreamingWswor;
use rand::thread_rng;

let mut rng = thread_rng();
let mut sampler = StreamingWswor::<f64, &str>::new(3); // Sample 3 items

// feed items one by one
sampler.feed("item1", 1.0, &mut rng)?;
sampler.feed("item2", 2.0, &mut rng)?;
sampler.feed("item3", 5.0, &mut rng)?; // Higher weight = more likely to be selected
sampler.feed("item4", 1.5, &mut rng)?;

// get results
let results: Vec<_> = sampler.take().collect();
println!("{:?}", results);

// or iterate without consuming
let sampler = StreamingWswor::new(2);
for item in sampler.iter() {
    println!("{}", item);
}
```

### Sampling a Single Item

```rust
use wswor::SingleStreamingWs;
use rand::thread_rng;

let mut rng = thread_rng();
let mut sampler = SingleStreamingWs::<f64, char>::new();

sampler.feed('A', 1.0, &mut rng)?;
sampler.feed('B', 3.0, &mut rng)?; // 3x more likely than A
sampler.feed('C', 2.0, &mut rng)?;

if let Some(result) = sampler.take() {
    println!("Selected: {}", result);
}
```

### Sampling from an Iterator

```rust
let mut sampler = StreamingWswor::new(2);
let items = vec![(1.0, "red"), (2.0, "green"), (3.0, "blue")];
sampler.feed_iter(items.into_iter(), &mut rng)?;
```

### Features

- Memory efficient -- Processes arbitrarily large datasets without storing all items in memory
- Streaming -- Works with any iterator; no need to collect data upfront
- Supports any numeric type implementing `Float` trait as a weight
- Zero weights are handled correctly (items with zero weight have minimal selection probability)
- Proper error handling for invalid weights (negative, NaN, infinite)
