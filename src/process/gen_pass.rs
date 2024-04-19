use rand::seq::SliceRandom;

const UPPER: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ";
const LOWER: &[u8] = b"abcdefghijkmnopqrstuvwxyz";
const NUM: &[u8] = b"123456789";
const SYM: &[u8] = b"!@#$%^&*_";

pub fn process_gen_pass(
    length: u8,
    upper: bool,
    lower: bool,
    num: bool,
    sym: bool,
) -> anyhow::Result<String> {
    let mut rng = rand::thread_rng();
    let mut password = Vec::new();
    let mut chars = Vec::new();

    if upper {
        chars.extend_from_slice(UPPER);
        password.push(*UPPER.choose(&mut rng).expect("UPPER is not empty"));
    }
    if lower {
        chars.extend_from_slice(LOWER);
        password.push(*LOWER.choose(&mut rng).expect("LOWER is not empty"));
    }
    if num {
        chars.extend_from_slice(NUM);
        password.push(*NUM.choose(&mut rng).expect("NUM is not empty"));
    }
    if sym {
        chars.extend_from_slice(SYM);
        password.push(*SYM.choose(&mut rng).expect("SYM is not empty"));
    };

    for _ in 0..length - password.len() as u8 {
        password.push(*chars.choose(&mut rng).expect("chars is not empty"));
    }

    // TODO: make sure that the password contains at least one of each type of character

    password.shuffle(&mut rng);

    Ok(String::from_utf8(password).expect("password is valid utf-8"))
}
