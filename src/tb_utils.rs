use itertools::Itertools;
use tigerbeetle_unofficial as tb;
pub mod u128 {
    pub fn to_hex_string(n: u128) -> String {
        format!("{:x}", n)
    }

    pub fn from_hex_string(s: &str) -> u128 {
        u128::from_str_radix(s, 16).expect("string can not be converted to u128 as hexadecimal")
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

pub fn create_transfers_error_name(err: tb::core::error::CreateTransfersError) -> String {
    match err {
        tigerbeetle_unofficial::error::CreateTransfersError::Send(err) => {
            err.kind().into_snake_case_str().to_string()
        }

        tigerbeetle_unofficial::error::CreateTransfersError::Api(err) => {
            let errs = err.as_slice();
            let err = errs
                .iter()
                .map(|err| err.kind().into_snake_case_str())
                .join(", ");
            err
        }
        _ => String::from("unknown error"),
    }
}
