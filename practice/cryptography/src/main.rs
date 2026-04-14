use sha2::{Digest, Sha256};

fn hash_data(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

fn main() {
    let inputs = vec!["hello", "hello world", "blockchain"];

    for input in inputs {
        let hash = hash_data(input);
        println!("SHA256({:?}) = {}", input, hash);
    }
    // 눈사태 효과 확인
    println!("\n--- 눈사태 효과 ---");
    println!("SHA256(\"hello\") = {}", hash_data("hello"));
    println!("SHA256(\"hellO\") = {}", hash_data("hellO"));
    // 한 글자 차이지만 완전히 다른 해시!
}
