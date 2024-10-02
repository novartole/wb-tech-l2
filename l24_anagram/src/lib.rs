use std::collections::{BTreeSet, HashMap};

pub fn get_anagrams<'a>(words: &[&'a str]) -> HashMap<&'a str, Vec<String>> {
    let mut anagrams = HashMap::new();
    let mut chars = Vec::default();

    for &word in words {
        let lc_word = word.to_lowercase();

        chars.clear();
        chars.extend(lc_word.chars());
        chars.sort_unstable();

        match anagrams.get_mut(&chars) {
            None => {
                anagrams.insert(chars.clone(), (BTreeSet::from([lc_word]), word));
            }
            Some((elmnts, _)) => {
                elmnts.insert(lc_word);
            }
        }
    }

    anagrams
        .into_values()
        .filter(|(elmnts, _)| elmnts.len() > 1)
        .map(|(elmnts, first_w)| (first_w, Vec::from_iter(elmnts)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let words = [
            "ПЯТКА",
            "пятак",
            "листок",
            "Пятак",
            "столик",
            "пятка",
            "хворост",
            "тяпка",
            "слиток",
        ];

        let anagrams = get_anagrams(&words);

        assert_eq!(anagrams.len(), 2);

        assert!(anagrams
            .get("ПЯТКА")
            .is_some_and(|elmnts| *elmnts == vec!["пятак", "пятка", "тяпка"]));

        assert!(anagrams
            .get("листок")
            .is_some_and(|elmnts| *elmnts == vec!["листок", "слиток", "столик"]));
    }
}
