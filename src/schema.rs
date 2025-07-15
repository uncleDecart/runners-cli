diesel::table! {
    runners_saturation (id) {
        id -> Int4,
        rid -> BigInt,
        name -> Varchar,
        busy -> Bool,
        created_at -> Timestamptz,
    }
}
