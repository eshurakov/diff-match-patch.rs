use std::fmt;

#[derive(Debug, Clone)]
pub struct DecodeError;
impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Malformed string")
    }
}
impl std::error::Error for DecodeError {}

#[allow(dead_code)]
pub fn percent_decode_u16(input: &[u8]) -> Result<Vec<u16>, Box<dyn std::error::Error>> {
    let mut input_iter = input.iter();
    let mut result: Vec<u16> = Vec::new();

    while let Some(&byte) = input_iter.next() {
        if byte != b'%' {
            result.push(byte as u16);
            continue;
        }

        let byte1 = next_percent_encoded_byte(&mut input_iter, true);
        if (byte1 & 0x80) == 0 {
            result.push(byte1);
            continue;
        }

        let mut byte2 = next_percent_encoded_byte(&mut input_iter, false);
        // continuation bytes have bitmask 10xx xxxx
        if (byte2 & 0xC0) != 0x80 {
            return Err(DecodeError.into());
        }

        // continuation bytes thus only contribute six bits each
        // these data bits are found with the bit mask xx11 1111
        byte2 = byte2 & 0x3F;

        // in two-byte sequences the first byte has bitmask 110x xxxx
        if (byte1 & 0xE0) == 0xC0 {
            // byte1 ___x xxxx << 6
            // byte2        __yy yyyy
            // value    x xxxxyy yyyy -> 11 bits
            result.push(((byte1 & 0x1F) << 6) | byte2);
            continue;
        }

        let mut byte3 = next_percent_encoded_byte(&mut input_iter, false);
        if (byte3 & 0xC0) != 0x80 {
            return Err(DecodeError.into());
        }

        byte3 = byte3 & 0x3F;

        // in three-byte sequences the first byte has bitmask 1110 xxxx
        if (byte1 & 0xF0) == 0xE0 {
            // byte1 ____ xxxx << 12
            // byte2        __yy yyyy << 6
            // byte3               __zz zzzz
            // value      xxxxyy yyyyzz zzzz -> 16 bits
            result.push(((byte1 & 0x0F) << 12) | (byte2 << 6) | byte3);
            continue;
        }

        let mut byte4 = next_percent_encoded_byte(&mut input_iter, false);
        if (byte4 & 0xC0) != 0x80 {
            return Err(DecodeError.into());
        }

        byte4 = byte4 & 0x3F;

        // in four-byte sequences the first byte has bitmask 1111 0xxx
        if (byte1 & 0xF8) == 0xF0 {
            // byte1 ____ _xxx << 18
            // byte2        __yy yyyy << 12
            // byte3               __zz zzzz << 6
            // byte4                      __tt tttt
            // value       xxxyy yyyyzz zzzztt tttt -> 21 bits
            let mut code_point = ((byte1 as u32 & 0x07) << 0x12) | ((byte2 as u32) << 0x0C) | ((byte3 as u32) << 0x06) | byte4 as u32;
            if code_point >= 0x010000 && code_point <= 0x10FFFF {
                code_point -= 0x010000;

                result.push((((code_point >> 10) & 0x3FF) | 0xD800) as u16);
                result.push((0xDC00 | (code_point & 0x3FF)) as u16);
                continue;
            }
        }

        return Err(DecodeError.into());
    }

    Ok(result)
}

fn next_percent_encoded_byte(iter: &mut std::slice::Iter<u8>, skip_percent: bool) -> u16 {
    if !skip_percent && iter.next() != Some(&b'%') {
        panic!("URI malformed");
    }

    let h = iter.next().and_then(|&b| (b as char).to_digit(16)).unwrap();
    let l = iter.next().and_then(|&b| (b as char).to_digit(16)).unwrap();

    (h as u8 * 0x10 + l as u8) as u16
}
