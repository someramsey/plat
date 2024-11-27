use crate::task::position::Position;

#[derive(Debug)]
pub enum FragmentData<'a> {
    AlphaNumeric(&'a str),
    Numeric(&'a str),
    Symbol(char),
}

#[derive(Debug)]
pub struct Fragment<'a> {
    pub data: FragmentData<'a>,
    pub position: Position,
}

enum State {
    None,
    Whitespace,
    Alphanumeric,
    Numeric,
}

fn capture_symbol(fragments: &mut Vec<Fragment>, mut position: &mut Position, ch: char) {
    fragments.push(Fragment {
        data: FragmentData::Symbol(ch),
        position: position.clone(),
    });
}

pub fn fragmentize(data: &str) -> Vec<Fragment> {
    let mut fragments: Vec<Fragment> = Vec::new();

    let mut head = 0;
    let mut tail = 0;

    let mut position = Position::new();

    let mut state = State::None;
    let mut chars = data.chars();

    for ch in chars.by_ref() {
        match state {
            State::None => {
                if ch.is_whitespace() {
                    state = State::Whitespace;
                } else if ch.is_numeric() {
                    state = State::Numeric;
                } else if ch.is_alphanumeric() {
                    state = State::Alphanumeric;
                } else {
                    capture_symbol(&mut fragments, &mut position, ch);
                    tail = head;
                }
            }

            State::Whitespace => {
                if ch.is_numeric() {
                    state = State::Numeric;
                } else if ch.is_alphanumeric() {
                    state = State::Alphanumeric;
                } else if !ch.is_whitespace() {
                    capture_symbol(&mut fragments, &mut position, ch);
                }

                tail = head;
            }

            State::Alphanumeric => {
                if !ch.is_alphanumeric() {
                    fragments.push(Fragment {
                        data: FragmentData::AlphaNumeric(&data[tail..head]),
                        position: position.clone(),
                    });

                    tail = head;

                    if ch.is_whitespace() {
                        state = State::Whitespace;
                    } else {
                        capture_symbol(&mut fragments, &mut position, ch);
                        state = State::None;
                    }
                }
            }

            State::Numeric => {
                if !ch.is_numeric() {
                    fragments.push(Fragment {
                        data: FragmentData::Numeric(&data[tail..head]),
                        position: position.clone(),
                    });

                    tail = head;

                    if ch.is_alphabetic() {
                        state = State::Alphanumeric;
                    } else if ch.is_whitespace() {
                        state = State::Whitespace;
                    } else {
                        capture_symbol(&mut fragments, &mut position, ch);
                    }
                }
            }
        }

        head += 1;
    }

    match state {
        State::Alphanumeric => {
            fragments.push(Fragment {
                data: FragmentData::AlphaNumeric(&data[tail..head]),
                position: position.clone(),
            });
        }

        State::Numeric => {
            fragments.push(Fragment {
                data: FragmentData::Numeric(&data[tail..head]),
                position: position.clone(),
            });
        }

        _ => {}
    }

    return fragments;
}