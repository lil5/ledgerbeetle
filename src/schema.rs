// @generated automatically by Diesel CLI.

diesel::table! {
    accounts (id) {
        id -> Int8,
        name -> Varchar,
        #[max_length = 31]
        tb_id -> Varchar,
        commodities_id -> Int4,
    }
}

diesel::table! {
    commodities (id) {
        id -> Int4,
        tb_ledger -> Int4,
        unit -> Text,
        decimal_place -> Int4,
    }
}

diesel::joinable!(accounts -> commodities (commodities_id));

diesel::allow_tables_to_appear_in_same_query!(
    accounts,
    commodities,
);
