//! Default Line_Break test.

use std::char;
use std::fs::File;
use std::io::{self, prelude::*, BufReader};
use std::iter::from_fn;
use std::u32;
use unicode_linebreak::*;

const TEST_FILE: &'static str = "tests/LineBreakTest.txt";

#[test]
fn test_lb_default() -> io::Result<()> {
    for line in lines_test_file()?
    {
        let TestLine { string, comment, spots } = match extract_test_from_line(&line) {
            Some(value) => value,
            None => continue,
        };

        let actual: Vec<_> = linebreaks(&string).map(|(i, _)| i).collect();
        let expected: Vec<_> = spots
            .into_iter()
            .filter_map(|(i, is_break)| if is_break { Some(i) } else { None })
            .collect();

        assert_eq!(
            actual, expected,
            "String: ‘{}’, comment: {}",
            string, comment
        );
    }

    Ok(())
}

#[test]
fn test_lb_iter() -> io::Result<()> {
    for line in lines_test_file()?
    {
        let TestLine { string, comment, spots } = match extract_test_from_line(&line) {
            Some(value) => value,
            None => continue,
        };

        let mut break_iter = LineBreakOpportunityIter::new(&string);
        let actual: Vec<_> = break_iter.map(|(i, _)| i).collect();
        let mut expected: Vec<_> = spots
            .into_iter()
            .filter_map(|(i, is_break)| if is_break { Some(i) } else { None })
            .collect();
        expected.pop(); // we don't consider the last index of a string a line break;
        // expected.pop(); // we don't consider the last index of a string a line break;
        // expected.pop(); // we don't consider the last index of a string a line break;

        assert_eq!(
            actual, expected,
            "String: ‘{}’, comment: {}",
            string, comment
        );
    }

    Ok(())
}

#[test]
fn test_lb_iter_multi_feed() -> io::Result<()> {
    for TestLine { string, comment, spots } in lines_test_file()?.filter_map(|line| extract_test_from_line(&line)) {
        // split string
        let char_indices : Vec<usize> = string.char_indices().map(|(i, _)| i).collect();
        let (string1, string2) = string.split_at(char_indices[char_indices.len() >> 1]);

        // feed both chunks to iterator
        let mut break_iter = LineBreakOpportunityIter::new(string1);
        let mut actual = Vec::new();
        for (i, _) in &mut break_iter {
            actual.push(i);
        }
        break_iter.feed(string2);
        for (i, _) in &mut break_iter {
            actual.push(i);
        }


        let mut expected: Vec<_> = spots
            .into_iter()
            .filter_map(|(i, is_break)| if is_break { Some(i) } else { None })
            .collect();
        expected.pop();

        assert_eq!(
            actual, expected,
            "String: ‘{}’, comment: {}",
            string, comment
        );
    }
    Ok(())
}

fn lines_test_file() -> Result<impl Iterator<Item = String>, io::Error> {
    let file = File::open(TEST_FILE)?;
    Ok(BufReader::new(file)
        .lines()
        .map(|l| l.unwrap())
        .filter(|l| !l.starts_with('#'))
    )
}

pub struct TestLine {
    string  : String,
    comment : String,
    spots   : Vec<(usize, bool)>,
}


fn extract_test_from_line(line: &str) -> Option<TestLine> {
    let (line, comment) = line.split_once("# ").expect("Missing comment");
    if comment.contains("[30.22]") || comment.contains("[999.0]") {
        return None;
    }
    let mut items = line.split_whitespace();
    items.next().unwrap();
    let mut byte_idx = 0;
    let (spots, string): (Vec<_>, String) = from_fn(|| {
        if let Some(hex) = items.next() {
            let codepoint = u32::from_str_radix(hex, 16)
                .ok()
                .and_then(char::from_u32)
                .expect("Invalid codepoint");
            byte_idx += codepoint.len_utf8();

            let is_break = match items.next() {
                Some("÷") => true,
                Some("×") => false,
                _ => unreachable!(),
            };

            Some(((byte_idx, is_break), codepoint))
        } else {
            None
        }
    })
    .unzip();
    Some(TestLine {
        string,
        comment: comment.to_string(),
        spots,
    })
}
