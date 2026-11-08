//! Cryptographic and arithmetic checksum validation algorithms.

/// Pure Rust standard SHA-256 implementation (FIPS 180-4) for zero-dependency checksum verification.
pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut state = [
        0x6a09e667u32, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
        0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ];

    let k = [
        0x428a2f98u32, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
        0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
        0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
        0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
        0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ];

    let bit_len = (data.len() as u64) * 8;
    let mut msg = data.to_vec();
    msg.push(0x80);
    while (msg.len() % 64) != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in msg.chunks_exact(64) {
        let mut w = [0u32; 64];
        for (i, part) in chunk.chunks_exact(4).enumerate() {
            w[i] = u32::from_be_bytes([part[0], part[1], part[2], part[3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
        }

        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = state;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = h.wrapping_add(s1).wrapping_add(ch).wrapping_add(k[i]).wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        state[0] = state[0].wrapping_add(a);
        state[1] = state[1].wrapping_add(b);
        state[2] = state[2].wrapping_add(c);
        state[3] = state[3].wrapping_add(d);
        state[4] = state[4].wrapping_add(e);
        state[5] = state[5].wrapping_add(f);
        state[6] = state[6].wrapping_add(g);
        state[7] = state[7].wrapping_add(h);
    }

    let mut out = [0u8; 32];
    for (i, val) in state.iter().enumerate() {
        out[i * 4..(i + 1) * 4].copy_from_slice(&val.to_be_bytes());
    }
    out
}

/// Double SHA-256 (`SHA256(SHA256(data))`) used in Bitcoin and TRON checksums.
pub fn double_sha256(data: &[u8]) -> [u8; 32] {
    sha256(&sha256(data))
}

const BASE58_ALPHABET: &[u8; 58] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

/// Decode Base58 string to raw bytes.
pub fn decode_base58(s: &str) -> Option<Vec<u8>> {
    let mut indexes = [0xFFu8; 256];
    for (i, &b) in BASE58_ALPHABET.iter().enumerate() {
        indexes[b as usize] = i as u8;
    }

    let mut zeroes = 0;
    for b in s.bytes() {
        if b == b'1' {
            zeroes += 1;
        } else {
            break;
        }
    }

    let mut bytes = Vec::new();
    for b in s.bytes() {
        let val = indexes[b as usize];
        if val == 0xFF {
            return None;
        }
        let mut carry = val as u32;
        for byte in bytes.iter_mut().rev() {
            let acc = (*byte as u32) * 58 + carry;
            *byte = (acc & 0xFF) as u8;
            carry = acc >> 8;
        }
        while carry > 0 {
            bytes.insert(0, (carry & 0xFF) as u8);
            carry >>= 8;
        }
    }

    let mut result = vec![0u8; zeroes];
    result.extend(bytes);
    Some(result)
}

/// Cryptographic validation for TRON Base58Check addresses.
///
/// TRON addresses decode to 25 bytes:
/// - 1 byte prefix (`0x41` for mainnet)
/// - 20 bytes RIPEMD-160/SHA-256 address payload
/// - 4 bytes double-SHA256 checksum
pub fn validate_tron_base58check(address: &str) -> (bool, String) {
    if address.len() != 34 || !address.starts_with('T') {
        return (false, "TRON address must be exactly 34 chars and start with 'T'".into());
    }

    let decoded = match decode_base58(address) {
        Some(d) => d,
        None => return (false, "Contains invalid Base58 characters".into()),
    };

    if decoded.len() != 25 {
        return (false, format!("Invalid decoded length: {} bytes (expected 25)", decoded.len()));
    }

    if decoded[0] != 0x41 {
        return (false, format!("Invalid TRON prefix: 0x{:02x} (expected 0x41)", decoded[0]));
    }

    let payload = &decoded[..21];
    let checksum = &decoded[21..25];
    let computed = double_sha256(payload);

    if &computed[..4] == checksum {
        (true, "Cryptographic Base58Check double-SHA256 checksum verified (Prefix 0x41)".into())
    } else {
        (false, "Double-SHA256 checksum mismatch (synthesized or corrupted address)".into())
    }
}

/// Validate ISO 7064 Mod 97-10 checksum for International Bank Account Numbers (IBAN).
pub fn validate_iban(iban: &str) -> (bool, String) {
    let clean: String = iban.chars().filter(|c| !c.is_whitespace()).collect();
    let upper = clean.to_uppercase();

    if upper.len() < 15 || upper.len() > 34 {
        return (false, format!("IBAN length {} outside 15-34 range", upper.len()));
    }

    let country = &upper[..2];
    if !country.chars().all(|c| c.is_ascii_alphabetic()) {
        return (false, "Invalid ISO country code in IBAN".into());
    }

    // Rearrange: Move first 4 characters to the back
    let rearranged = format!("{}{}", &upper[4..], &upper[..4]);

    // Convert letters to two digits (A=10, ..., Z=35) and compute Mod 97 progressively
    let mut remainder = 0u32;
    for ch in rearranged.chars() {
        if ch.is_ascii_digit() {
            let digit = ch.to_digit(10).unwrap();
            remainder = (remainder * 10 + digit) % 97;
        } else if ch.is_ascii_alphabetic() {
            let val = (ch as u32) - ('A' as u32) + 10;
            remainder = (remainder * 100 + val) % 97;
        } else {
            return (false, format!("Invalid character '{}' in IBAN", ch));
        }
    }

    if remainder == 1 {
        (true, format!("ISO 7064 Mod 97-10 validated (Country: {})", country))
    } else {
        (false, format!("ISO 7064 Mod 97-10 checksum failed (remainder {})", remainder))
    }
}

/// Validate ISBN-13 checksum (EAN-13 format).
pub fn validate_isbn13(isbn: &str) -> bool {
    let digits: Vec<u32> = isbn.chars().filter_map(|c| c.to_digit(10)).collect();
    if digits.len() != 13 {
        return false;
    }
    let sum: u32 = digits[..12]
        .iter()
        .enumerate()
        .map(|(i, &d)| if i % 2 == 0 { d } else { d * 3 })
        .sum();
    let check = (10 - (sum % 10)) % 10;
    check == digits[12]
}

/// Validate ISBN-10 checksum.
pub fn validate_isbn10(isbn: &str) -> bool {
    let clean: String = isbn.chars().filter(|c| c.is_ascii_digit() || *c == 'X' || *c == 'x').collect();
    if clean.len() != 10 {
        return false;
    }
    let mut sum = 0u32;
    for (i, c) in clean.chars().enumerate() {
        let weight = 10 - i as u32;
        let val = if i == 9 && (c == 'X' || c == 'x') {
            10
        } else if let Some(d) = c.to_digit(10) {
            d
        } else {
            return false;
        };
        sum += val * weight;
    }
    sum % 11 == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_known_vector() {
        // SHA-256("abc") = ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad
        let digest = sha256(b"abc");
        assert_eq!(
            format!("{:02x?}", &digest).replace(", ", "").replace('[', "").replace(']', ""),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn test_tron_real_mainnet_address() {
        // TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t is the official Tether USDT contract on TRON
        let (valid, reason) = validate_tron_base58check("TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t");
        assert!(valid, "Real TRON USDT address must validate: {}", reason);
    }

    #[test]
    fn test_tron_corrupted_checksum() {
        // Corrupted checksum: changed last character from 'f' to '9'
        let (valid, reason) = validate_tron_base58check("TXYZopYRdj2D9XRtbG411XZZ3kM5VkAeB9");
        assert!(!valid, "Synthesized address with bad checksum must fail cryptographic check: {}", reason);
    }

    #[test]
    fn test_iban_validation() {
        // Valid German IBAN
        let (valid, reason) = validate_iban("DE89370400440532013000");
        assert!(valid, "Valid German IBAN should pass: {}", reason);

        // Invalid IBAN (modified check digit)
        let (invalid, _) = validate_iban("DE89370400440532013001");
        assert!(!invalid, "Corrupted IBAN must fail");
    }

    #[test]
    fn test_isbn13_validation() {
        assert!(validate_isbn13("978-0-306-40615-7"));
        assert!(!validate_isbn13("978-0-306-40615-9"));
    }
}
