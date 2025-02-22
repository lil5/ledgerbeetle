// @generated automatically by Diesel CLI.

diesel::table! {
    accounts (id) {
        id -> Int8,
        name -> Varchar,
        tb_id -> Int8,
        currencies_id -> Nullable<Int4>,
    }
}

diesel::table! {
    currencies (id) {
        id -> Int4,
        tb_ledger -> Int4,
        unit -> Text,
    }
}

diesel::table! {
    transfer_details (id) {
        id -> Int8,
        tb_id -> Int8,
        description -> Text,
    }
}

diesel::joinable!(accounts -> currencies (currencies_id));

diesel::allow_tables_to_appear_in_same_query!(
    accounts,
    currencies,
    transfer_details,
);
