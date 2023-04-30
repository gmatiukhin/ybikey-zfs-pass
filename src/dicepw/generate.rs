use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;

use super::{
    params::DicewareParams,
    passphrase::{DicewarePassword, ExtraCharacter, EXTRA_CHARS, WORDLIST_SPLIT},
};

/// Seed for generating random passwords.
///
/// This matches the type returned by YubiKey's challenge-response process.
type Seed = [u8; 20];

/// Change a seed value to return the value that's greater by one.
pub fn advance(seed: &mut Seed) {
    if seed == &mut [255; 20] {
        for i in seed.iter_mut() {
            *i = 0;
        }
        return;
    }

    let last_not_full = seed
        .iter()
        .enumerate()
        .rev()
        .find(|(_i, v)| **v != 255)
        .unwrap()
        .0;
    seed[last_not_full] += 1;
    for i in seed.iter_mut().skip(last_not_full + 1) {
        *i = 0;
    }
}

/// Expand a 20-byte seed into a 32-byte seed by prepending a constant.
/// The 32-byte seed can then be used in the ChaCha20 RNG.
fn expand_seed(s: &Seed) -> [u8; 32] {
    let expand = b"............";
    #[allow(clippy::drop_non_drop)]
    {
        concat_arrays::concat_arrays!(*expand, *s)
    }
}

/// Use the given seed and parameters to generate a passphrase.
pub fn generate_pw(s: &Seed, params: &DicewareParams) -> DicewarePassword {
    let mut rng = ChaCha20Rng::from_seed(expand_seed(s));

    let words = rng.gen_range(params.words.low..=params.words.high);
    let mut word_items = Vec::with_capacity(words);
    for _ in 0..words {
        word_items.push(rng.gen_range(0..WORDLIST_SPLIT.len()))
    }

    let extras = rng.gen_range(params.extras.low..=params.extras.high);
    let mut extra_items = Vec::with_capacity(extras);
    for _ in 0..extras {
        let position = rng.gen_range(0..=words); // inclusive because maybe after all words
        let extra = if rng.gen() {
            ExtraCharacter::Special {
                which: rng.gen_range(0..EXTRA_CHARS.len()),
                position,
            }
        } else {
            let word_idx = rng.gen_range(0..words);
            let word_len = WORDLIST_SPLIT[word_idx].chars().count();
            ExtraCharacter::Indexed {
                from_word: word_idx,
                inside_word: rng.gen_range(0..word_len),
                position,
            }
        };
        extra_items.push(extra);
    }

    DicewarePassword {
        words: word_items,
        extras: extra_items,
    }
}

#[derive(PartialEq, Debug)]
pub enum CheckPasswordError {
    WrongLength {
        expected: usize,
        got: usize,
    },
    WrongWord {
        at_idx: usize,
        expected: usize,
        got: usize,
    },
    Unknown,
}

/// Use the given seed, parameters, and passphrase to check if this seed generates the same
/// passphrase.
/// Stop early if a critical parameter doesn't match.
pub fn check_pw(
    s: &Seed,
    params: &DicewareParams,
    pw: &DicewarePassword,
) -> Result<(), CheckPasswordError> {
    let mut rng = ChaCha20Rng::from_seed(expand_seed(s));

    let words = rng.gen_range(params.words.low..=params.words.high);
    if words != pw.words.len() {
        return Err(CheckPasswordError::WrongLength {
            expected: pw.words.len(),
            got: words,
        });
    }
    let mut word_items = Vec::with_capacity(words);
    for i in 0..words {
        let new_word_idx = rng.gen_range(0..WORDLIST_SPLIT.len());
        if new_word_idx != pw.words[i] {
            return Err(CheckPasswordError::WrongWord {
                at_idx: i,
                expected: pw.words[i],
                got: new_word_idx,
            });
        }
        word_items.push(new_word_idx);
    }

    let extras = rng.gen_range(params.extras.low..=params.extras.high);
    let mut extra_items = Vec::with_capacity(extras);
    for _ in 0..extras {
        let position = rng.gen_range(0..=words); // inclusive because maybe after all words
        let extra = if rng.gen() {
            ExtraCharacter::Special {
                which: rng.gen_range(0..EXTRA_CHARS.len()),
                position,
            }
        } else {
            let word_idx = rng.gen_range(0..words);
            let word_len = WORDLIST_SPLIT[word_idx].chars().count();
            ExtraCharacter::Indexed {
                from_word: word_idx,
                inside_word: rng.gen_range(0..word_len),
                position,
            }
        };
        extra_items.push(extra);
    }

    if (&DicewarePassword {
        words: word_items,
        extras: extra_items,
    } == pw)
    {
        Ok(())
    } else {
        Err(CheckPasswordError::Unknown)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_advance() {
        let mut seed = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        advance(&mut seed);
        assert_eq!(
            seed,
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]
        );

        let mut seed = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 10, 255, 255, 255, 254,
        ];
        advance(&mut seed);
        assert_eq!(
            seed,
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 10, 255, 255, 255, 255]
        );
        advance(&mut seed);
        assert_eq!(
            seed,
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 11, 0, 0, 0, 0]
        );

        let mut seed = [255; 20];
        advance(&mut seed);
        assert_eq!(seed, [0; 20]);
    }

    #[test]
    fn test_gen_is_deterministic() {
        let seed: [u8; 20] = [10; 20];
        let params = DicewareParams {
            words: crate::dicepw::params::Range { low: 1, high: 10 },
            extras: crate::dicepw::params::Range { low: 0, high: 5 },
        };
        let pw_a = generate_pw(&seed, &params);
        let pw_b = generate_pw(&seed, &params);
        assert_eq!(pw_a, pw_b);
    }

    #[test]
    fn test_check_works() {
        let seed: [u8; 20] = [10; 20];
        let params = DicewareParams {
            words: crate::dicepw::params::Range { low: 1, high: 10 },
            extras: crate::dicepw::params::Range { low: 0, high: 5 },
        };
        let pw = generate_pw(&seed, &params);
        assert_eq!(check_pw(&seed, &params, &pw), Ok(()));
    }
}
