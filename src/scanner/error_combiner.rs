use miette::{NamedSource, SourceSpan};

use super::error::ScannerError::{self, UnexpectedCharacter, UnexpectedCharacters};

pub struct ErrorCombiner {
    source: String,
    filename: String,
}

impl ErrorCombiner {
    pub fn new(source: String, filename: String) -> Self { Self { source, filename } }

    fn handle_accumulated(
        &self,
        accumulated: &mut Vec<char>,
        last_offset: &mut Option<SourceSpan>,
        result: &mut Vec<ScannerError>,
    ) {
        if let Some(last_offset) = last_offset {
            match &accumulated[..] {
                [] => (),
                [char] => result.push(UnexpectedCharacter {
                    char: *char,
                    src: self.named_source(),
                    location: *last_offset,
                }),
                _ => result.push(UnexpectedCharacters {
                    chars: accumulated.iter().collect(),
                    src: self.named_source(),
                    location: *last_offset,
                }),
            };
        }
        accumulated.clear();
        *last_offset = None;
    }

    pub fn combine(
        &self,
        scanner_errors: Vec<ScannerError>,
    ) -> Vec<ScannerError> {
        let mut result = vec![];
        let mut accumulated = vec![];
        let mut last_offset: Option<SourceSpan> = None;

        for error in scanner_errors.into_iter() {
            if let UnexpectedCharacter {
                char,
                src: _,
                location,
            } = &error
            {
                match last_offset {
                    Some(offset) if offset.offset() + offset.len() == location.offset() => {
                        last_offset = Some((offset.offset(), offset.len() + location.len()).into());
                    }
                    Some(_) => {
                        self.handle_accumulated(&mut accumulated, &mut last_offset, &mut result);
                        last_offset = Some(*location);
                    }
                    None => {
                        last_offset = Some(*location);
                    }
                }
                accumulated.push(*char);
                continue;
            }
            self.handle_accumulated(&mut accumulated, &mut last_offset, &mut result);
            result.push(error)
        }
        self.handle_accumulated(&mut accumulated, &mut last_offset, &mut result);
        result
    }

    fn named_source(&self) -> NamedSource {
        NamedSource::new(self.filename.clone(), self.source.to_string())
    }
}