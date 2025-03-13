ALTER TABLE accounts
DROP CONSTRAINT fk_commodities;

ALTER TABLE accounts
DROP CONSTRAINT uk_accounts_name_commodities;

ALTER TABLE accounts
DROP COLUMN commodities_id;

DROP TABLE commodities;
