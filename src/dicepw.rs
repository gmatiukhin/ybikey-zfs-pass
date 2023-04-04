use std::fmt::Display;

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

/// Module for representing Diceware passwords in a low-entropy form.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct DicewarePassword {
    /// Wordlist indices, in order
    pub words: Vec<usize>,

    /// Special characters added to the phrase
    pub extras: Vec<ExtraCharacter>,
}

/// Represents a special symbol added to the password.
/// In this rule, one or more special symbols can be placed
/// in every word gap, and is separated from the words by a space on both sides.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum ExtraCharacter {
    /// The extra character comes from a table of special characters.
    Special {
        /// The index of the character in the table
        which: usize,

        /// The position of the character in the password.
        /// If zero, is before the first word;
        /// if one, after the first word.
        /// If greater than the number of words, after the last word.
        position: usize,
    },

    /// The extra character comes from one of the words in the passphrase.
    Indexed {
        /// Index of passphrase word to use, zero-based, modulo the length of the passphrase.
        from_word: usize,
        /// Index of the letter to use inside the picked word, zero-based and modulo the length of
        /// the word.
        inside_word: usize,

        /// The position of the character in the password.
        /// If zero, is before the first word;
        /// if one, after the first word.
        /// If greater than the number of words, after the last word.
        position: usize,
    },
}

const EXTRA_CHARS: &[char] = &[
    '~', '!', '#', '$', '%', '^', '&', '*', '(', ')', '-', '=', '+', '[', ']', '\\', '{', '}', ':',
    ';', '"', '\'', '<', '>', '?', '/', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
];

impl ExtraCharacter {
    pub fn get_char(&self, phrase: &DicewarePassword) -> char {
        match self {
            ExtraCharacter::Special { which, .. } => EXTRA_CHARS[*which],
            ExtraCharacter::Indexed {
                from_word,
                inside_word,
                ..
            } => {
                let word_id = phrase.words[from_word % phrase.words.len()];
                let word_text = WORDLIST_SPLIT[word_id];
                let char = word_text
                    .chars()
                    .nth(inside_word % word_text.chars().count())
                    .unwrap();
                char
            }
        }
    }

    pub fn position(&self) -> usize {
        match self {
            ExtraCharacter::Special { position, .. } => *position,
            ExtraCharacter::Indexed { position, .. } => *position,
        }
    }
}

const WORDLIST: &str = include_str!("diceware_wordlist.txt");

lazy_static! {
    static ref WORDLIST_SPLIT: Vec<&'static str> = WORDLIST.split_whitespace().collect();
}

impl Display for DicewarePassword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // NOTE: If more than one extra character is in the same position, they are printed without
        // spaces:
        // "!@ hello @#$ world %^&".
        let mut first_extras = false;
        for chr in self.extras.iter().filter(|x| x.position() == 0) {
            write!(f, "{}", chr.get_char(self))?;
            first_extras = true;
        }
        if first_extras {
            write!(f, " ")?;
        }

        let late_extras = self
            .extras
            .iter()
            .filter(|x| x.position() >= self.words.len())
            .count()
            > 0;

        for (i, word) in self.words.iter().enumerate() {
            let word_text = WORDLIST_SPLIT[*word];
            let space_after_word = if i == self.words.len() - 1 {
                // If last word, then space after it is before late extras
                late_extras
            } else {
                true
            };
            write!(f, "{word_text}{}", if space_after_word { " " } else { "" })?;
            let mut extras = false;
            for chr in self.extras.iter().filter(|x| x.position() == i + 1) {
                write!(f, "{}", chr.get_char(self))?;
                extras = true;
            }

            let space_after_extras = i != self.words.len() - 1; // only put space after extras if
                                                                // it's not the last word
            if extras && space_after_extras {
                write!(f, " ")?;
            }
        }

        for chr in self
            .extras
            .iter()
            .filter(|x| x.position() > self.words.len())
        {
            write!(f, "{}", chr.get_char(self))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::dicepw::ExtraCharacter;

    use super::DicewarePassword;

    #[test]
    pub fn test_simple_password() {
        let pw = DicewarePassword {
            words: vec![1000, 1001, 1002, 1003, 1004, 1005],
            extras: vec![],
        };
        assert_eq!(format!("{pw}"), "busch bush bushel bushy buss bust");
    }

    #[test]
    pub fn test_password_with_extra() {
        let pw = DicewarePassword {
            words: vec![1000, 1001, 1002, 1003, 1004, 1005],
            extras: vec![
                ExtraCharacter::Special {
                    which: 0,
                    position: 0,
                },
                ExtraCharacter::Special {
                    which: 1,
                    position: 0,
                },
                ExtraCharacter::Special {
                    which: 2,
                    position: 1,
                },
                ExtraCharacter::Special {
                    which: 3,
                    position: 1,
                },
                ExtraCharacter::Special {
                    which: 4,
                    position: 2,
                },
                ExtraCharacter::Special {
                    which: 5,
                    position: 2,
                },
                ExtraCharacter::Special {
                    which: 6,
                    position: 3,
                },
                ExtraCharacter::Special {
                    which: 7,
                    position: 3,
                },
                ExtraCharacter::Special {
                    which: 8,
                    position: 4,
                },
                ExtraCharacter::Special {
                    which: 9,
                    position: 4,
                },
                ExtraCharacter::Special {
                    which: 10,
                    position: 5,
                },
                ExtraCharacter::Special {
                    which: 11,
                    position: 5,
                },
                ExtraCharacter::Special {
                    which: 16,
                    position: 6,
                },
                ExtraCharacter::Special {
                    which: 17,
                    position: 6,
                },
            ],
        };
        assert_eq!(
            format!("{pw}"),
            "~! busch #$ bush %^ bushel &* bushy () buss -= bust {}"
        );
    }

    #[test]
    pub fn test_password_with_indexed_extra() {
        let pw = DicewarePassword {
            words: vec![1000, 1001, 1002],
            extras: vec![
                ExtraCharacter::Indexed {
                    from_word: 0,
                    inside_word: 0,
                    position: 0,
                },
                ExtraCharacter::Indexed {
                    from_word: 0,
                    inside_word: 7,
                    position: 0,
                },
                ExtraCharacter::Special {
                    which: 0,
                    position: 0,
                },
            ],
        };
        assert_eq!(format!("{pw}"), "bs~ busch bush bushel");
    }
}
