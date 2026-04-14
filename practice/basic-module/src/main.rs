mod crypto;
mod models;

use crypto::sha256;
use models::Block;

fn main() {
    let hash = sha256("hello");
    let block = Block::new();
}
