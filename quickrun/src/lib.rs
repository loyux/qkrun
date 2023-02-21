pub mod buildimg;
pub mod cli;
pub mod dockerapi;
pub mod k8sapi;
pub mod templates;
mod tools;
// use rand::distributions::Alphanumeric;
// use rand::{thread_rng, Rng};

// fn genstring() -> String {
//     let rand_string: String = thread_rng()
//         .sample_iter(&Alphanumeric)
//         .take(6)
//         .map(char::from)
//         .collect();
//     rand_string.as_str().to_ascii_lowercase().to_string()
// }

// #[test]
// fn test_genstring() {
//     genstring();
// }
