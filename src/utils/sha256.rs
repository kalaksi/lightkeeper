use sha2::{Digest, Sha256};

pub fn hash(contents: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(contents);
    let hash = &hasher.finalize();

    hex::encode(hash)
}
