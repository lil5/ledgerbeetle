pub mod u128 {
    pub fn to_hex_string(n: u128) -> String {
        format!("{:X}", n)
    }

    pub fn from_hex_string<'a>(s: &'a str) -> u128 {
        u128::from_str_radix(s, 16)
            .expect(format!("string can not be converted to u128 '{}'", s).as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::u128;

    #[test]
    fn to_hex_string() {
        let result = u128::to_hex_string(16u128);
        assert_eq!(result, "f")
    }

    #[test]
    fn from_hex_string() {
        let result = u128::from_hex_string("f");
        assert_eq!(result, 16u128)
    }
}

pub mod account_flags {
    pub static LINKED: u16 = 1 << 0;
    pub static DEBITS_MUST_NOT_EXCEED_CREDITS: u16 = 1 << 1;
    pub static CREDITS_MUST_NOT_EXCEED_DEBITS: u16 = 1 << 2;
    pub static HISTORY: u16 = 1 << 3;
    pub static IMPORTED: u16 = 1 << 4;
    pub static CLOSED: u16 = 1 << 5;
}
pub mod transfer_flags {
    pub static LINKED: u16 = 1 << 0;
    pub static PENDING: u16 = 1 << 1;
    pub static POST_PENDING_TRANSFER: u16 = 1 << 2;
    pub static VOID_PENDING_TRANSFER: u16 = 1 << 3;
    pub static BALANCING_DEBIT: u16 = 1 << 4;
    pub static BALANCING_CREDIT: u16 = 1 << 5;
    pub static CLOSING_DEBIT: u16 = 1 << 6;
    pub static CLOSING_CREDIT: u16 = 1 << 7;
    pub static IMPORTED: u16 = 1 << 8;
}

pub mod account_filter_flags {
    pub static DEBITS: u16 = 1 << 0;
    pub static CREDITS: u16 = 1 << 1;
    pub static REVERSED: u16 = 1 << 2;
}

pub mod query_filter_flags {
    pub static REVERSED: u16 = 1 << 0;
}
