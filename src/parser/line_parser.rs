use super::Line;
use heapless::Vec;

enum LineParseState {
    AtStart, // From the start of a new line up to the first non-whitespace character.
    LabelOrContent, // We have seen non-whitespace, but, until we see a ':', not sure if it's a label or content yet.
    AfterLabel, // We have seen a label and the ':', but not yet any content. Skip whitespace here.
    Content, // We have seen content, but not yet a newline. Everything should be recorded as content.
    Comment, // We have seen a '#' and are now ignoring everything until the next newline.
}
use LineParseState::*;
#[derive(Debug)]
pub enum ParseError {
    CapacityFull,
}

// have a vec of iterators per line, with the already trimmed lines.
pub fn string_to_iters<'a, const N:usize>(string: &'a str) -> Result<Vec<Line<'a>, N>, ParseError>
{
    let mut state = AtStart;
    let mut start = 0;
    let mut end = 0;
    let mut eol = 0;
    let mut label = "";
    let mut iters = Vec::<Line, N>::new();
    let mut chars = string.char_indices().peekable();
    while let Some((idx,c)) = chars.next() {
        let w = c.len_utf8();
        eol = idx + w;
        // Handle Windows-style newlines (\r\n) by igonring the '\r' IF it is followed by a '\n'.
        // If not, just treat the \r as a newline, which is what some old Mac-style text files use.              
        if c == '\r' && chars.peek().map(|(_, next_c)| *next_c == '\n')
            .unwrap_or(false) {
            continue;
        }
        // Handle all special characters, sometimes for a specific state.
        match c {
            '\n' | '\r' => {
                handle_end(&mut iters, string, start, end, label, eol)?;
                start = eol;
                end = start;
                label = "";
                state = AtStart;
                continue;
            },
            '#' => {
                // If we see a comment char, we ignore the rest of the line.
                state = Comment;
                continue;
            },
            ' ' | '\t' => {
                // Skip whitespace for AtStart and AfterLabel.
                if matches!(state, AtStart | AfterLabel) {
                    start = eol;
                    end = start;
                    continue;
                }
            },
            ':' => {
                if matches!(state, AtStart | LabelOrContent) {
                    label = &string[start..idx];
                    start = eol;
                    end = start;
                    state = AfterLabel;
                    continue;
                }
            },
            _ => {}
        }
        // On any other character, we might need to update the state and end index.
        match state {
            AtStart => {
                state = LabelOrContent;
                end = eol;
            },
            AfterLabel => {
                state = Content;
                end = eol;
            }
            LabelOrContent | Content => {
                end = eol;
            },
            _ => {}
        }
    }
    // If the string ended without a newline, we might still have content to process.
    if matches!(state, LabelOrContent | Content | Comment) {
        handle_end(&mut iters, string, start, end, label, eol)?;
    }
    Ok(iters)
}

pub fn handle_end<'a, const N:usize>(iters: &mut Vec<Line<'a>, N>, string: &'a str,
    start: usize, end: usize, label: &'a str, eol: usize) -> Result<(), ParseError>{
    if start < end {
        // Only process if we actually have a string
        if !label.is_empty() {
            // We have a label. Add it only if it is not already known.
            if let None = iters.iter().find(|line| line.label == label) {
                // We have a new label, so we can add it. For remain, we need the string after this line.
                let remain = &string[eol..].trim_start_matches(|c| c == '\n' || c == '\r');
                iters.push(Line::new(label, &string[start..end], remain))                        
                    .map_err(|_| ParseError::CapacityFull)?;
            }
        } else {
            // No label, simple case
            iters.push(Line::new("", &string[start..end], ""))
                .map_err(|_| ParseError::CapacityFull)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct LineParserTc {
        description: &'static str,
        case: &'static str,
        exp: [(&'static str, &'static str, &'static str); 3],
        exp_len: usize,
    }

    const LINE_PARSER_TCS: [LineParserTc; 26] = [
        LineParserTc {
            description: "Only whitespace",
            case: "  \t  \n\n\n  \t",
            exp: [("", "", ""), ("", "", ""), ("", "", "")],
            exp_len: 0
        },
        LineParserTc {
            description: "One line no with whitespace and no label",
            case: "__‾‾\n",
            exp: [("", "__‾‾", ""), ("", "", ""), ("", "", "")],
            exp_len: 1
        },
        LineParserTc {
            description: "Two lines with whitespace, empty line and no label",
            case: "\n  \t__XX\n\nXX__",
            exp: [("", "__XX", ""), ("", "XX__", ""), ("", "", "")],
            exp_len: 2
        },
        LineParserTc {
            description: "Three lines with whitespace, empty line and no label",
            case: "\n  \t__‾X \n\nXX__\nX",
            exp: [("", "__‾X ", ""), ("", "XX__", ""), ("", "X", "")],
            exp_len: 3
        },
        LineParserTc {
            description: "One line with whitespace, empty line and a label",
            case: "\n\t Lab 1: \t__‾‾\n\n",
            exp: [("Lab 1", "__‾‾", ""), ("", "", ""), ("", "", "")],
            exp_len: 1
        },
        LineParserTc {
            description: "Two line with whitespace, empty line and two labels",
            case: "\n\t Lab 1: \t__XX\n\nLab 2:XX",
            exp: [("Lab 1", "__XX", "Lab 2:XX"), ("Lab 2", "XX", ""), ("", "", "")],
            exp_len: 2
        },
        LineParserTc {
            description: "Two lines with no whitespace and two labels, one repeating next line",
            case: "Lab1:__‾‾ \nLab1:_XX_\nLab2:XX",
            exp: [("Lab1", "__‾‾ ", "Lab1:_XX_\nLab2:XX"), ("Lab2", "XX", ""), ("", "", "")],
            exp_len: 2
        },
        LineParserTc {
            description: "Two lines with whitespace and two labels, one repeating other line",
            case: "\n\t Lab 1: \t__XX\n\nLab 2:XX\nLab 1: \t_XX_",
            exp: [("Lab 1", "__XX", "Lab 2:XX\nLab 1: \t_XX_"),
                  ("Lab 2", "XX", "Lab 1: \t_XX_"), ("", "", "")],
            exp_len: 2
        },
        LineParserTc {
            description: "Three lines with whitespace and two labels, both repeating next line",
            case: " XXX\n\n\t Lab 1: \t__XX\n\nLab 1: X_\nLab 2:XX\nLab 2: \t_XX_",
            exp: [("", "XXX", ""),
                  ("Lab 1", "__XX", "Lab 1: X_\nLab 2:XX\nLab 2: \t_XX_"),
                  ("Lab 2", "XX", "Lab 2: \t_XX_")],
            exp_len: 3
        },
        LineParserTc {
            description: "Three lines no whitespace and two labels, both repeating other line",
            case: "Lab1:__XX\nLab2:XX\nX_X_\nLab1:_XX_\nLab2:__",
            exp: [("Lab1", "__XX", "Lab2:XX\nX_X_\nLab1:_XX_\nLab2:__"),
                  ("Lab2", "XX", "X_X_\nLab1:_XX_\nLab2:__"), ("", "X_X_", "")],
            exp_len: 3
        },
        LineParserTc {
            description: "Content with leading/trailing spaces preserved",
            case: "Label1:  content with spaces  \nLabel2:value",
            exp: [("Label1", "content with spaces  ", "Label2:value"),
                  ("Label2", "value", ""), ("", "", "")],
            exp_len: 2
        },
        LineParserTc {
            description: "Multiple colons (only first is label separator)",
            case: "Label1:value:with:colons\nLabel2:another:value",
            exp: [("Label1", "value:with:colons", "Label2:another:value"),
                  ("Label2", "another:value", ""), ("", "", "")],
            exp_len: 2
        },
        LineParserTc {
            description: "Three lines with comments",
            case: "
# This is comment 1
Lab1:__XX # This is comment 2
Lab2:XX
X_X_
Lab1:_XX_ # This is comment 3
Lab2:__",
            exp: [("Lab1", "__XX ", "Lab2:XX\nX_X_\nLab1:_XX_ # This is comment 3\nLab2:__"),
                  ("Lab2", "XX", "X_X_\nLab1:_XX_ # This is comment 3\nLab2:__"),
                  ("", "X_X_", "")],
            exp_len: 3
        },
        LineParserTc {
            description: "Comment-only lines (should be skipped)",
            case: "# This is just a comment\nLabel1:value1\n# Another comment",
            exp: [("Label1", "value1", "# Another comment"), ("", "", ""), ("", "", "")],
            exp_len: 1
        },
        LineParserTc {
            description: "Content with inline comment (comment removed)",
            case: "Label1:value1 # inline comment\nLabel2:value2",
            exp: [("Label1", "value1 ", "Label2:value2"),
                  ("Label2", "value2", ""), ("", "", "")],
            exp_len: 2
        },
        LineParserTc {
            description: "Only whitespace and comments",
            case: "   \n\t\n# comment\n  # another",
            exp: [("", "", ""), ("", "", ""), ("", "", "")],
            exp_len: 0
        },
        LineParserTc {
            description: "Empty comment line",
            case: "#\nLabel:value",
            exp: [("Label", "value", ""), ("", "", ""), ("", "", "")],
            exp_len: 1
        },
        LineParserTc {
            description: "Colon at start (empty label), then content",
            case: ":content_after_colon\nLabel:normal_content",
            exp: [("", "content_after_colon", ""),
                  ("Label", "normal_content", ""), ("", "", "")],
            exp_len: 2
        },
        LineParserTc {
            description: "Colon at start, no content",
            case: ":\nLabel:value",
            exp: [("Label", "value", ""),("", "", ""), ("", "", "")],
            exp_len: 1
        },
        LineParserTc {
            description: "Empty content after label",
            case: "Label:\nOther:value",
            exp: [("Other", "value", ""),("", "", ""), ("", "", "")],
            exp_len: 1
        },
        LineParserTc {
            description: "Empty content after label",
            case: "Label: \t \t\nOther:value",
            exp: [("Other", "value", ""),("", "", ""), ("", "", "")],
            exp_len: 1
        },
        LineParserTc {
            description: "Windows line endings (\\r\\n)",
            case: "Label1:content1\r\nLabel2:content2\r\n",
            exp: [("Label1", "content1", "Label2:content2\r\n"),
                ("Label2", "content2", ""), ("", "", "")],
            exp_len: 2
        },
        LineParserTc {
            description: "Mix of \\n and \\r\\n line endings",
            case: "Label1:content1\nLabel2:content2\r\nLabel3:content3\n",
            exp: [("Label1", "content1", "Label2:content2\r\nLabel3:content3\n"),
                  ("Label2", "content2", "Label3:content3\n"),
                  ("Label3", "content3", "")],
            exp_len: 3
        },
        LineParserTc {
            description: "Only one line with Windows line ending at the end",
            case: "Label:value\r\n",
            exp: [("Label", "value", ""), ("", "", ""), ("", "", "")],
            exp_len: 1
        },
        LineParserTc {
            description: "Old Mac line endings (\\r only)",
            case: "Label1:content1\rLabel2:content2\r",
            exp: [("Label1", "content1", "Label2:content2\r"),
                  ("Label2", "content2", ""), ("", "", "")],
            exp_len: 2
        },
         LineParserTc {
            description: "Test single character label and content",
            case: "A:B\nC:D\nE",
            exp: [("A", "B", "C:D\nE"), ("C", "D", "E"), ("", "E", "")],
            exp_len: 3
        },
    ];

    #[test]
    fn test_string_to_iters() {
        let cases = &LINE_PARSER_TCS;
        for tc in cases {
            let iters: Vec<Line, 3> = string_to_iters(tc.case).unwrap();

            assert_eq!(tc.exp_len, iters.len(), "unexpected number of values for case '{}'", tc.description);
            for (line, iter) in iters.iter().enumerate() {
                assert_eq!(tc.exp[line].0, iter.label, "Label mismatch at line {} for case '{}'", line, tc.description);
                assert_eq!(tc.exp[line].1, iter.content, "Content mismatch at line {} for case '{}'", line, tc.description);
                assert_eq!(tc.exp[line].2, iter.remain, "Remain mismatch at line {} for case '{}'", line, tc.description);
            }
        }
    }

    #[test]
    fn test_capacity_exceeded() {
          // Vec has max 3 entries, 4 lines requested.
        let input = "L1:c1\nL2:c2\nL3:c3\nL4:c4";
        let result: Result<Vec<Line, 3>, ParseError> = string_to_iters(input);
        
        assert!(matches!(result, Err(ParseError::CapacityFull)));
    }
}