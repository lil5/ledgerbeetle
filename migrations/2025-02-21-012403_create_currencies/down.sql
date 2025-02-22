ALTER TABLE accounts
DROP CONSTRAINT fk_currencies;

ALTER TABLE accounts
DROP COLUMN currencies_id;

DROP TABLE currencies;