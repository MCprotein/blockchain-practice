use mini_chain::Blockchain;

fn main() {
    let mut chain = Blockchain::new(2);
    chain.add_block("alice -> bob: 10");
    chain.add_block("bob -> carol: 3");

    for block in chain.blocks() {
        println!("#{:02} {} {}", block.index, block.data, block.hash);
    }
    println!("valid: {}", chain.is_valid());
}
