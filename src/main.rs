use gol::{io::GoLFile, iter::IntoTransitionIter};

fn main() -> Result<(), &'static str> {
    let start = std::time::Instant::now();
    // {
    //     let file = std::fs::File::open("./big.gol").unwrap();
    //     let data = read_gol_file_good(file)?;
    //     let result = transition(&data);
    //     println!("{}", start.elapsed().as_millis());
    // }

    // let mut file = std::fs::File::open("./medium.gol").or(Err("Failed to open test file"))?;
    // let mut file = std::fs::File::open("./reader_test.gol").or(Err("Failed to open test file"))?;

    let file = std::fs::File::open("./big.gol").or(Err("Failed to open test file"))?;
    let reader = GoLFile::with_max_capacity(file, 300000)?;
    let mut output = Vec::with_capacity(reader.height);
    for _ in 0..reader.height {
        output.push(vec![false; reader.width]);
    }
    reader.into_transition_iter().for_each(|v| {
        output[v.0][v.1] = v.2;
    });
    println!("{}", start.elapsed().as_millis());

    Ok(())
}

#[cfg(test)]
mod tests {
    use gol::io::{read_gol, write_gol};

    #[test]
    fn write_and_read_return_same_data() {
        let data = vec![
            vec![true, false, true, false, true, false, true, false, true],
            vec![false, true, false, true, false, true, false, true, false],
            vec![true, false, true, false, true, false, true, false, true],
        ]
        .into();
        let mut buffer = [2; 12 + 9 * 3];
        write_gol(buffer.as_mut_slice(), &data).unwrap();
        let data2 = read_gol(buffer.as_slice()).unwrap();
        assert_eq!(data, data2);
    }

    mod reader {
        use gol::io::GoLFile;
        use gol::read::Read;
        use std::io::Cursor;

        #[test]
        fn works_with_files_smaller_than_buffer() -> Result<(), &'static str> {
            let data = b"GOFL\x00\x00\x00\x01\x00\x00\x00\x01\x00";
            let mut reader = GoLFile::new(Cursor::new(data))?;
            assert_eq!(reader.width, 1);
            assert_eq!(reader.height, 1);
            assert_eq!(reader.read_cell(0, 0)?, false);
            assert!(reader.read_cell(1, 0).is_err());
            assert!(reader.read_cell(0, 1).is_err());

            let data = b"GOFL\x00\x00\x00\x03\x00\x00\x00\x03\x00\x00\x00\x01\x01\x01\x00\x00\x00";
            let mut reader = GoLFile::new(Cursor::new(data))?;
            assert_eq!(reader.width, 3);
            assert_eq!(reader.height, 3);
            assert_eq!(reader.read_cell(0, 0)?, false);
            let expected = [false, false, false, true, true, true, false, false, false];
            for y in 0..3 {
                for x in 0..3 {
                    assert_eq!(reader.read_cell(x, y)?, expected[x + y * 3]);
                }
            }
            Ok(())
        }

        #[test]
        fn works_with_files_larger_than_buffer() -> Result<(), &'static str> {
            let data = b"GOFL\x00\x00\x00\x0b\x00\x00\x00\x06\x00\x00\x00\x00\x00\
                \x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\
                \x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\
                \x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\
                \x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01";
            assert_eq!(data.len(), 12 + 66);
            let mut reader = GoLFile::new(Cursor::new(data))?;
            assert_eq!(reader.width, 11);
            assert_eq!(reader.height, 6);
            assert_eq!(reader.read_cell(0, 0)?, false);
            assert_eq!(reader.read_cell(10, 5)?, true);
            Ok(())
        }
    }
}
