use std::time::{SystemTime, UNIX_EPOCH};

use rand::Rng;

fn format_radix(mut x: u128, radix: u32) -> String {
    let mut result = vec![];

    loop {
        let m = x % radix as u128;
        x = x / radix as u128;

        // will panic if you use a bad radix (< 2 or > 36).
        result.push(std::char::from_digit(m as u32, radix).unwrap());
        if x == 0 {
            break;
        }
    }
    result.into_iter().rev().collect()
}

fn date_now() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwords on this machine, please check machines clock.")
        .as_millis()
}

pub fn random() -> f32 {
    rand::thread_rng().gen_range(0.0..1.0)
}

pub fn get_random_int(a: i32, b: i32) -> i32 {
    rand::thread_rng().gen_range(a..b)
}

pub fn generate_verify_fp() -> String {
    let t: Vec<_> = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz"
        .chars()
        .collect();

    let e = t.len();
    let n = format_radix(date_now(), 36);
    let mut r: [Option<char>; 36] = [None; 36];

    r[8] = Some('_');
    r[13] = Some('_');
    r[18] = Some('_');
    r[23] = Some('_');
    r[14] = Some('4');

    for o in 0..r.len() {
        if r[o].is_some() {
            continue;
        }

        let i = 0 | (random() * e as f32).floor() as usize;
        let k = if 19 == o { 3 & i | 8 } else { i };
        r[o] = Some(t[k]);
    }

    let r: String = r.iter().map(|c| c.unwrap()).collect();

    format!("verify_{}_{}", n, r)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn radix_formats_number() {
        let a: u128 = 1675719784841;
        let result = format_radix(a, 36);
        assert_eq!(result, "ldtcal55");
    }

    #[test]
    fn generate_verify_fp_token() {
        let fp = generate_verify_fp();
        assert!(fp.len() > 0)
    }
}
