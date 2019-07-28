use chessground::DrawBrush;
use shakmaty::Square;

#[derive(Debug)]
pub enum Annotation {
    Arrow {
        color: DrawBrush,
        from: Square,
        to: Square,
    },
    Square {
        color: DrawBrush,
        square: Square,
    },
}

pub fn parse_annotations(mut bytes: &[u8]) -> Vec<Annotation> {
    let mut result = vec![];

    loop {
        if bytes.is_empty() {
            return result;
        }

        if bytes[0] != b'[' {
            bytes = &bytes[1..];
        }
        else {
            let (new_bytes, new_annotations) = annotations(&bytes[1..]);
            bytes = new_bytes;
            result.extend(new_annotations);
        }
    }
}

fn annotations(mut bytes: &[u8]) -> (&[u8], Vec<Annotation>) {
    if bytes[0] == b'%' {
        bytes = &bytes[1..];
        if bytes[0] == b'c' {
            bytes = &bytes[1..];
            if bytes[0] == b'a' {
                return arrows(&bytes[1..]);
            }
            else if bytes[0] == b's' {
                return squares(&bytes[1..]);
            }
        }
    }
    eprintln!("Expecting %cs or %ca");
    (skip_until_annotation_end(bytes), vec![])
}

fn arrow(mut bytes: &[u8]) -> Option<(&[u8], Annotation)> {
    let (new_bytes, color) = color(bytes)?;
    bytes = new_bytes;
    let from = Square::from_ascii(&bytes[..2]).ok()?;
    bytes = &bytes[2..];
    let to = Square::from_ascii(&bytes[..2]).ok()?;
    bytes = &bytes[2..];
    Some((bytes, Annotation::Arrow {
        color,
        from,
        to,
    }))
}

fn arrows(mut bytes: &[u8]) -> (&[u8], Vec<Annotation>) {
    let mut annotations = vec![];
    if bytes[0] == b'l' {
        bytes = &bytes[1..];
        if bytes[0] == b' ' || bytes[0] == b'\n' {
            bytes = &bytes[1..];
            loop {
                if let Some((new_bytes, arrow)) = arrow(bytes) {
                    annotations.push(arrow);
                    bytes = new_bytes;
                    if bytes[0] == b']' {
                        bytes = &bytes[1..];
                        break;
                    }
                    else if bytes[0] == b',' {
                        bytes = &bytes[1..];
                    }
                    else {
                        eprintln!("Unexpected byte {}", bytes[0]);
                        return (skip_until_annotation_end(bytes), annotations);
                    }
                }
                else {
                    return (skip_until_annotation_end(bytes), annotations);
                }
            }
        }
    }
    else {
        eprintln!("Expecting `l `");
        return (skip_until_annotation_end(bytes), annotations);
    }
    (bytes, annotations)
}

fn color(bytes: &[u8]) -> Option<(&[u8], DrawBrush)> {
    let color =
        match bytes[0] {
            b'Y' => DrawBrush::Yellow,
            b'G' => DrawBrush::Green,
            b'R' => DrawBrush::Red,
            _ => return None,
        };
    Some((&bytes[1..], color))
}

fn square(mut bytes: &[u8]) -> Option<(&[u8], Annotation)> {
    let (new_bytes, color) = color(bytes)?;
    bytes = new_bytes;
    let square = Square::from_ascii(&bytes[..2]).ok()?;
    bytes = &bytes[2..];
    Some((bytes, Annotation::Square {
        color,
        square,
    }))
}

fn squares(mut bytes: &[u8]) -> (&[u8], Vec<Annotation>) {
    let mut annotations = vec![];
    if bytes[0] == b'l' {
        bytes = &bytes[1..];
        if bytes[0] == b' ' || bytes[0] == b'\n' {
            loop {
                if let Some((new_bytes, square)) = square(bytes) {
                    annotations.push(square);
                    bytes = new_bytes;
                    if bytes[0] == b']' {
                        bytes = &bytes[1..];
                        break;
                    }
                    else if bytes[0] == b',' {
                        bytes = &bytes[1..];
                    }
                    else {
                        eprintln!("Unexpected byte {}", bytes[0]);
                        return (skip_until_annotation_end(bytes), annotations);
                    }
                }
                else {
                    return (skip_until_annotation_end(bytes), annotations);
                }
            }
        }
    }
    else {
        eprintln!("Expecting `l `");
        return (skip_until_annotation_end(bytes), annotations);
    }
    (bytes, annotations)
}

fn skip_until_annotation_end(mut bytes: &[u8]) -> &[u8] {
    loop {
        if bytes.is_empty() {
            return bytes;
        }

        if bytes[0] == b']' {
            bytes = &bytes[1..];
            return bytes;
        }
        else {
            bytes = &bytes[1..];
        }
    }
}
