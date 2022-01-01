use std::fmt;

pub type MM = i32; // millimeters

const MM_PER_INCH: f64 = 25.4;


pub struct Paper {
    size: ISO216,
    w: MM,
    h: MM,
}

pub enum Orientation {
    Portrait,
    Landscape,
}
#[derive(Debug)]
pub enum ISO216 {
    A0,
    A1,
    A2,
    A3,
    A4,
    A5,
    A6,
    A7,
    A8,
}

impl fmt::Display for ISO216 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}


#[derive(Debug)]
pub enum ISO216PaperLookupErr {
    BadLenErr,
    ParseIntErr,
}

impl Paper {
    pub fn from_iso216(
        size: ISO216,
        orientation: Orientation,
    ) -> Result<Self, ISO216PaperLookupErr> {
        let size_str: Vec<char> = size.to_string().chars().collect();
        if size_str.len() != 2 {
            return Err(ISO216PaperLookupErr::BadLenErr);
        }
        let num_c = size_str[1];
        let num_i = String::from(num_c).parse::<i32>();

        if let Ok(num) = num_i {
            // Input string is OK. Now calculate size.
            let mut longest: MM = 1189;
            let mut shortest: MM = 841;
            (0..num).for_each(|_| {
                let temp = longest;
                longest = shortest;
                shortest = temp / 2;
            });

            match orientation {
                Orientation::Portrait => Ok(Paper {
                    size,
                    w: shortest,
                    h: longest,
                }),
                Orientation::Landscape => Ok(Paper {
                    size,
                    w: longest,
                    h: shortest,
                }),
            }
        } else {
            Err(ISO216PaperLookupErr::ParseIntErr)
        }
    }

    pub fn pixel_dimensions(&self, dpi: u16) -> (u32, u32) {
        let dpi_float: f64 = dpi.into();
        let w_inch = self.w as f64 / MM_PER_INCH;
        let h_inch = self.h as f64 / MM_PER_INCH;
        ((w_inch.round() * dpi_float) as u32, (h_inch.round() * dpi_float) as u32)
    }
}

#[cfg(test)]
mod tests {
    use crate::paper::*;

    #[test]
    fn test_from_iso216() {
        let portrait = Paper::from_iso216(ISO216::A0, Orientation::Portrait).unwrap();
        assert_eq!(portrait.w, 841, "{}", portrait.w);
        assert_eq!(portrait.h, 1189, "{}", portrait.h);
        let portrait = Paper::from_iso216(ISO216::A1, Orientation::Portrait).unwrap();
        assert_eq!(portrait.w, 594, "{}", portrait.w);
        assert_eq!(portrait.h, 841, "{}", portrait.h);
        let portrait = Paper::from_iso216(ISO216::A2, Orientation::Portrait).unwrap();
        assert_eq!(portrait.w, 420, "{}", portrait.w);
        assert_eq!(portrait.h, 594, "{}", portrait.h);
        let portrait = Paper::from_iso216(ISO216::A3, Orientation::Portrait).unwrap();
        assert_eq!(portrait.w, 297, "{}", portrait.w);
        assert_eq!(portrait.h, 420, "{}", portrait.h);
        let portrait = Paper::from_iso216(ISO216::A4, Orientation::Portrait).unwrap();
        assert_eq!(portrait.w, 210, "{}", portrait.w);
        assert_eq!(portrait.h, 297, "{}", portrait.h);

        let landscape = Paper::from_iso216(ISO216::A0, Orientation::Landscape).unwrap();
        assert_eq!(landscape.h, 841, "{}", portrait.h);
        assert_eq!(landscape.w, 1189, "{}", portrait.w);
    }

    #[test]
    fn test_dpi() {
        let paper = Paper::from_iso216(ISO216::A4, Orientation::Portrait).unwrap();
        let (w, h) = paper.pixel_dimensions(300);
        assert_eq!(2480, w);
        assert_eq!(3408, h);
    }
}
