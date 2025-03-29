use micromap::Map;
use rand::rngs::SmallRng;
use rand::Rng;
use rand::SeedableRng;
use uuid::Builder;
use uuid::Uuid;

fn main() {
    let mut rng = SmallRng::seed_from_u64(1);

    let mut uuids = vec![];
    for _ in 0..64 {
        let random_bytes = rng.random();
        let uuid = Builder::from_bytes(random_bytes).into_uuid();
        uuids.push(uuid);
    }

    let key_index_list: Vec<usize> = (0..10000).map(|_| rng.random_range(0..64)).collect();
    let value_list: Vec<[u8; 16]> = (0..10000).map(|_| rng.random()).collect();

    // get average of key_index_list
    let mut sum = 0;
    for i in 0..key_index_list.len() {
        sum += key_index_list[i];
    }
    let avg = sum as f64 / key_index_list.len() as f64;
    println!("key_index_list avg is {:?}", avg);

    let mut m: Map<Uuid, [u8; 16], 64> = Map::new();

    let mut before_full = vec![];
    let mut not_full = true;
    for i in 0..10000 {
        let index = key_index_list[i];
        let key = uuids[index];
        let value = value_list[i];

        m.insert(key, value);

        if not_full {
            before_full.push(m.len());
            if m.len() == m.capacity() {
                not_full = false;
                println!(
                    "before_full(len={}) is {:?}",
                    before_full.len(),
                    before_full
                );
            }
        }
    }
}
