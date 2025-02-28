use std::fmt::{self, Debug};

use tigerbeetle_unofficial as tb;
pub mod u128 {
    pub fn to_hex_string(n: u128) -> String {
        format!("{:x}", n)
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
        let result = u128::to_hex_string(15u128);
        assert_eq!(result, "f")
    }

    #[test]
    fn from_hex_string() {
        let result = u128::from_hex_string("f");
        assert_eq!(result, 15u128)
    }
}

pub fn create_transfers_error_name<'a>(err: tb::core::error::CreateTransfersError) -> &'a str {
    match err {
        tigerbeetle_unofficial::error::CreateTransfersError::Send(err) => {
            err.kind().into_snake_case_str()
        }

        tigerbeetle_unofficial::error::CreateTransfersError::Api(err) => {
            let errs = err.as_slice();
            let err = errs.first();
            match err {
                Some(err) => err.kind().into_snake_case_str(),
                None => "unknown create transfer error",
            }
        }
        _ => "unknown error",
    }
}
